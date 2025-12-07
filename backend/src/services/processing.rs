use anyhow::Result;
use log::{info, error};
use std::collections::HashMap;
use chrono::{Utc, Duration, NaiveDateTime};

use crate::cache::Cache;
use crate::config::settings::AppConfig;
use crate::database::{self, DbConn};
use crate::domain::{self, ExpandedGame};
use crate::rating;

pub struct ProcessingService {
    config: AppConfig,
    cache: Cache,
}

impl ProcessingService {
    pub fn new(config: AppConfig) -> Result<Self> {
        Ok(Self {
            config,
            cache: Cache::new("cache")?,
        })
    }

    pub fn run(&self) -> Result<()> {
        let db_path = std::env::var("DATABASE_PATH")
            .unwrap_or_else(|_| "warsaw_pool_ranking.db".to_string());
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
        let pool = database::create_pool(db_path)?;
        let mut conn = database::get_connection(&pool)?;

        // Step 1: Reset database (PoC - no migrations)
        database::setup::reset_database(&mut conn)?;
        info!("  → Database schema reset\n");

        // Step 2: Load cached tournaments
        let tournaments = self.load_tournaments_from_cache()?;
        info!("  → Loaded {} tournaments from cache\n", tournaments.len());

        // Step 3: Insert tournaments and expand to games (all games, before filtering for periods)
        let all_expanded_games = self.process_tournaments(&mut conn, &tournaments)?;
        info!("  → Expanded to {} individual games (total)", all_expanded_games.len());

        // Step 4: Calculate and save ratings for each period
        for period in &self.config.rating.periods {
            info!("  Calculating ratings for period: {}", period.name);

            let filtered_games = if let Some(years) = period.years {
                let cutoff_date = Utc::now().naive_utc() - Duration::days((years * 365) as i64);
                all_expanded_games.iter()
                    .filter(|game| game.date >= cutoff_date)
                    .cloned()
                    .collect::<Vec<ExpandedGame>>()
            } else {
                all_expanded_games.clone()
            };

            info!("    → {} games for period {}", filtered_games.len(), period.name);

            let ratings = self.calculate_player_ratings(&filtered_games, &period.name)?;
            info!("    → Calculated ratings for {} players for period {}", ratings.len(), period.name);

            self.save_ratings_to_db(&mut conn, &ratings, &period.name)?;
            info!("    → Saved ratings for period {} to database\n", period.name);
        }

        Ok(())
    }

    fn load_tournaments_from_cache(&self) -> Result<Vec<crate::domain::TournamentResponse>> {
        self.cache
            .load_parsed("tournaments")?
            .ok_or_else(|| anyhow::anyhow!("No tournaments found in cache"))
    }

    fn process_tournaments(
        &self,
        conn: &mut DbConn,
        tournaments: &[crate::domain::TournamentResponse],
    ) -> Result<Vec<ExpandedGame>> {
        let mut all_games = Vec::new();
        let mut skipped_count = 0;

        for (idx, tournament) in tournaments.iter().enumerate() {
            if (idx + 1) % 100 == 0 || idx + 1 == tournaments.len() {
                info!("  Processing tournament {}/{}", idx + 1, tournaments.len());
            }

            if self.is_doubles_tournament(&tournament.name) {
                skipped_count += 1;
                continue;
            }

            let player_info_map = self.extract_player_info(tournament);

            let tournament_db = self.insert_tournament_to_db(conn, tournament)?;
            let mut games = self.expand_tournament_games(tournament)?;

            games.retain(|g| {
                let w_name = player_info_map.get(&g.winner_id).map(|p| p.name.as_str()).unwrap_or("");
                let l_name = player_info_map.get(&g.loser_id).map(|p| p.name.as_str()).unwrap_or("");
                !self.is_team_player(w_name) && !self.is_team_player(l_name)
            });

            self.insert_games_to_db(conn, &games, tournament_db.id, &player_info_map)?;
            all_games.append(&mut games);
        }

        if skipped_count > 0 {
            info!("  Skipped {} doubles/team tournaments", skipped_count);
        }

        // Apply time decay only once, on the full set of games, before filtering by period
        self.apply_time_decay_weights(&mut all_games);
        Ok(all_games)
    }

    fn is_doubles_tournament(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        lower.contains("debel") ||
        lower.contains("deblowy") ||
        lower.contains("double") ||
        lower.contains(" par") ||
        lower.contains("pary") ||
        lower.contains("team")
    }

    fn is_team_player(&self, name: &str) -> bool {
        name.contains('/') || name.contains('&') || name.contains('+') || name.to_lowercase().starts_with("team") || name.to_lowercase().starts_with("6ur")
    }

    fn insert_tournament_to_db(
        &self,
        conn: &mut DbConn,
        tournament: &crate::domain::TournamentResponse,
    ) -> Result<database::Tournament> {
        let start_date = self.parse_tournament_date(&tournament.starttime)?;
        let end_date = self.parse_optional_tournament_date(&tournament.stoptime)?;

        database::tournaments::upsert_tournament(
            conn,
            tournament.id,
            &tournament.name,
            tournament.venue_id(),
            &tournament.venue_name(),
            start_date,
            end_date,
        )
    }

    fn parse_tournament_date(&self, date_str: &str) -> Result<NaiveDateTime> {
        use chrono::{DateTime, NaiveDateTime as ND};

        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Ok(dt.naive_utc());
        }

        if let Ok(dt) = ND::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
            return Ok(dt);
        }

        if let Ok(dt) = ND::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.f") {
            return Ok(dt);
        }

        anyhow::bail!("Failed to parse tournament date: {}", date_str)
    }

    fn parse_optional_tournament_date(
        &self,
        date_str: &Option<String>,
    ) -> Result<Option<NaiveDateTime>> {
        match date_str {
            Some(s) => Ok(Some(self.parse_tournament_date(s)?)),
            None => Ok(None),
        }
    }

    fn expand_tournament_games(
        &self,
        tournament: &crate::domain::TournamentResponse,
    ) -> Result<Vec<ExpandedGame>> {
        domain::games_expansion::expand_tournament_to_games(tournament)
    }

    fn extract_player_info(
        &self,
        tournament: &crate::domain::TournamentResponse,
    ) -> HashMap<i64, domain::PlayerInfo> {
        let mut players = HashMap::new();

        for match_data in &tournament.matches {
            if match_data.is_played() {
                // Clone to store independent PlayerInfo structs
                if let Some(player_id) = match_data.player_a.player_id {
                    players.entry(player_id).or_insert_with(|| match_data.player_a.clone());
                }
                if let Some(player_id) = match_data.player_b.player_id {
                    players.entry(player_id).or_insert_with(|| match_data.player_b.clone());
                }
            }
        }
        players
    }

    fn insert_games_to_db(
        &self,
        conn: &mut DbConn,
        games: &[ExpandedGame],
        tournament_db_id: i32,
        player_info_map: &HashMap<i64, domain::PlayerInfo>,
    ) -> Result<()> {
        for game in games {
            let first_player_info = player_info_map.get(&game.winner_id)
                .ok_or_else(|| anyhow::anyhow!("Winner not found in player_info_map"))?;
            let second_player_info = player_info_map.get(&game.loser_id)
                .ok_or_else(|| anyhow::anyhow!("Loser not found in player_info_map"))?;

            let first_player_db = self.upsert_player(conn, first_player_info)?;
            let second_player_db = self.upsert_player(conn, second_player_info)?;

            database::games::insert_game(
                conn,
                tournament_db_id,
                first_player_db.id,
                second_player_db.id,
                1,
                0,
                game.date,
                game.weight,
            )?;
        }

        Ok(())
    }

    fn upsert_player(
        &self,
        conn: &mut DbConn,
        player_info: &crate::domain::PlayerInfo,
    ) -> Result<database::Player> {
        let cuescore_id = player_info.player_id.unwrap_or(0);
        let name = &player_info.name;
        let avatar_url = player_info.image.as_deref();
        database::players::upsert_player(conn, cuescore_id, name, avatar_url)
    }

    fn apply_time_decay_weights(&self, games: &mut [ExpandedGame]) {
        let current_date = Utc::now().naive_utc();
        rating::weighting::apply_weights_to_games(games, current_date);
    }

    fn calculate_player_ratings(
        &self,
        games: &[ExpandedGame],
        rating_type: &str,
    ) -> Result<Vec<rating::PlayerRating>> {
        let game_results = self.convert_to_game_results(games);
        let mut ratings = rating::calculate_ratings(&game_results, &self.config.rating);
        for r in &mut ratings {
            r.rating_type = rating_type.to_string();
        }
        Ok(ratings)
    }

    fn convert_to_game_results(
        &self,
        games: &[ExpandedGame],
    ) -> Vec<rating::GameResult> {
        games
            .iter()
            .map(|g| rating::GameResult {
                winner_id: g.winner_id as i32,
                loser_id: g.loser_id as i32,
                weight: g.weight,
            })
            .collect()
    }

    fn save_ratings_to_db(
        &self,
        conn: &mut DbConn,
        ratings: &[rating::PlayerRating],
        rating_type: &str,
    ) -> Result<()> {
        let calculated_at = Utc::now().naive_utc();

        for player_rating in ratings {
            let cuescore_id = player_rating.player_id as i64;
            // When saving ratings, we don't have direct access to avatar_url here from PlayerInfo
            // So, for existing players, this upsert will just update rating-related fields if needed,
            // but won't overwrite avatar_url if it's already set.
            // For new players inserted here, avatar_url will be None, to be updated later by game insertion path.
            let player = database::players::upsert_player(conn, cuescore_id, "Unknown Player", None)?;

            if let Err(e) = database::ratings::insert_rating(
                conn,
                player.id,
                rating_type,
                player_rating.rating,
                player_rating.games_played,
                player_rating.confidence_level.as_str(),
                calculated_at,
            ) {
                error!("Failed to insert rating for player {}: {:?}", player.id, e);
                return Err(e.into());
            }
        }

        Ok(())
    }
}