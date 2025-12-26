use anyhow::{Result, Context};
use log::info;
use std::path::Path;
use chrono::{Utc, Duration};
use rusqlite::OptionalExtension;

use crate::cache::Cache;
use crate::config::settings::AppConfig;
use crate::config::paths;
use crate::database::{self, DbConn};
use crate::database::repositories::player_repository::PlayerRepository;
use crate::domain::ExpandedGame;
use crate::fetchers::cuescore_models::TournamentResponse;
use crate::services::{tournament_processor::TournamentProcessor, game_processor::GameProcessor, rating_processor::RatingProcessor};

pub struct ProcessingService {
    config: AppConfig,
    cache: Cache,
}

impl ProcessingService {
    pub fn new(config: AppConfig) -> Result<Self> {
        Ok(Self {
            config,
            cache: Cache::new(paths::get_cache_dir())?,
        })
    }

    pub fn run(&self) -> Result<()> {
        let db_path = paths::get_database_path();
        let temp_db_path = format!("{}.tmp", db_path);

        info!("=== Starting Data Processing (Atomic) ===\n");
        info!("Target DB: {}, Temp DB: {}", db_path, temp_db_path);

        // Clean up previous temp file if exists
        if std::path::Path::new(&temp_db_path).exists() {
            std::fs::remove_file(&temp_db_path)?;
        }

        // Process to temp DB
        self.process_to_db(&temp_db_path)?;

        // Atomic swap
        std::fs::rename(&temp_db_path, &db_path)?;
        info!("Successfully swapped database to {}", db_path);

        info!("=== Processing Complete ===");
        Ok(())
    }

    fn process_to_db(&self, db_path: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(db_path).parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let pool = database::create_pool(db_path)?;
        let mut conn = database::get_connection(&pool)?;

        // Setup database schema
        database::setup::create_avatars_table_if_missing(&mut conn)?;
        database::setup::reset_database(&mut conn)?;
        info!("  → Database schema reset\n");

        // Load tournaments from cache
        let tournaments = self.load_tournaments_from_cache()?;
        info!("  → Loaded {} tournaments from cache\n", tournaments.len());

        // Process tournaments to games
        let mut all_games = self.process_all_tournaments(&mut conn, &tournaments)?;
        info!("  → Expanded to {} individual games (total)", all_games.len());

        // Apply time decay weights
        GameProcessor::apply_time_decay(&mut all_games);

        // Update player last_played dates
        self.update_player_last_played_dates(&mut conn)?;

        // Calculate and save ratings for each period
        self.process_rating_periods(&mut conn, &all_games)?;

        Ok(())
    }

    fn load_tournaments_from_cache(&self) -> Result<Vec<TournamentResponse>> {
        self.cache
            .load_parsed("tournaments")?
            .ok_or_else(|| anyhow::anyhow!("No tournaments found in cache"))
    }

    fn process_all_tournaments(
        &self,
        conn: &mut DbConn,
        tournaments: &[TournamentResponse],
    ) -> Result<Vec<ExpandedGame>> {
        let mut all_games = Vec::new();
        let mut skipped_count = 0;

        // Create processors once and reuse (no mutable state)
        let tournament_processor = TournamentProcessor::new(self.config.tournament_processing.clone());
        let game_processor = GameProcessor::new(self.config.tournament_processing.clone());

        for (idx, tournament) in tournaments.iter().enumerate() {
            if (idx + 1) % 100 == 0 || idx + 1 == tournaments.len() {
                info!("  Processing tournament {}/{}", idx + 1, tournaments.len());
            }

            // Skip doubles/team tournaments
            if tournament_processor.is_doubles_tournament(&tournament.name) {
                skipped_count += 1;
                continue;
            }

            // Process single tournament
            let mut games = self.process_single_tournament(
                conn,
                tournament,
                &tournament_processor,
                &game_processor,
            )?;
            all_games.append(&mut games);
        }

        if skipped_count > 0 {
            info!("  Skipped {} doubles/team tournaments", skipped_count);
        }

        Ok(all_games)
    }

    fn process_single_tournament(
        &self,
        conn: &mut DbConn,
        tournament: &TournamentResponse,
        tournament_processor: &TournamentProcessor,
        game_processor: &GameProcessor,
    ) -> Result<Vec<ExpandedGame>> {
        // Extract player info and insert tournament
        let player_info_map = tournament_processor.extract_player_info(tournament);
        let tournament_db = tournament_processor.insert_tournament(conn, tournament)?;

        // Expand to games and filter out team players
        let mut games = tournament_processor.expand_to_games(tournament)?;
        game_processor.filter_team_players(&mut games, &player_info_map);

        // Insert games and players to database
        game_processor.insert_games(conn, &games, tournament_db.id, &player_info_map)?;

        Ok(games)
    }

    fn update_player_last_played_dates(&self, conn: &mut DbConn) -> Result<()> {
        info!("  Updating player last played dates...");
        let players = PlayerRepository::list_all(conn)?;

        for player in players {
            let last_played_date = conn.query_row(
                "SELECT MAX(date) FROM games WHERE first_player_id = ?1 OR second_player_id = ?1",
                rusqlite::params![player.id],
                |row| row.get(0),
            ).optional().context("Failed to query max game date")?;

            if let Some(date) = last_played_date {
                PlayerRepository::update_player_last_played(conn, player.id, date)?;
            }
        }
        Ok(())
    }

    fn process_rating_periods(
        &self,
        conn: &mut DbConn,
        all_games: &[ExpandedGame],
    ) -> Result<()> {
        for period in &self.config.rating.periods {
            info!("  Calculating ratings for period: {}", period.name);

            // Filter games by period
            let filtered_games = self.filter_games_by_period(all_games, period.years.map(|y| y as i32));
            info!("    → {} games for period {}", filtered_games.len(), period.name);

            // Calculate ratings
            let ratings = RatingProcessor::calculate_ratings(
                &filtered_games,
                &period.name,
                &self.config.rating,
            )?;
            info!("    → Calculated ratings for {} players for period {}", ratings.len(), period.name);

            // Save to database
            RatingProcessor::save_ratings(conn, &ratings, &period.name)?;
            info!("    → Saved ratings for period {} to database\n", period.name);
        }

        Ok(())
    }

    fn filter_games_by_period(&self, all_games: &[ExpandedGame], years: Option<i32>) -> Vec<ExpandedGame> {
        if let Some(years) = years {
            let cutoff_date = Utc::now().naive_utc() - Duration::days((years * 365) as i64);
            all_games.iter()
                .filter(|game| game.date >= cutoff_date)
                .cloned()
                .collect()
        } else {
            all_games.to_vec()
        }
    }
}
