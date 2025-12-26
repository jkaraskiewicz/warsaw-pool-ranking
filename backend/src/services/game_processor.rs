use anyhow::Result;
use std::collections::HashMap;
use chrono::Utc;

use crate::config::settings::TournamentProcessingSettings;
use crate::database::DbConn;
use crate::database::repositories::{game_repository::GameRepository, player_repository::PlayerRepository};
use crate::domain::ExpandedGame;
use crate::fetchers::cuescore_models::PlayerInfo;
use crate::rating;

pub struct GameProcessor {
    settings: TournamentProcessingSettings,
}

impl GameProcessor {
    pub fn new(settings: TournamentProcessingSettings) -> Self {
        Self { settings }
    }

    /// Filter out team players from games
    pub fn filter_team_players(
        &self,
        games: &mut Vec<ExpandedGame>,
        player_info_map: &HashMap<i64, PlayerInfo>,
    ) {
        games.retain(|g| {
            let w_name = player_info_map.get(&g.winner_id).map(|p| p.name.as_str()).unwrap_or("");
            let l_name = player_info_map.get(&g.loser_id).map(|p| p.name.as_str()).unwrap_or("");
            !self.is_team_player(w_name) && !self.is_team_player(l_name)
        });
    }

    /// Insert games to database along with player records
    pub fn insert_games(
        &self,
        conn: &mut DbConn,
        games: &[ExpandedGame],
        tournament_db_id: i32,
        player_info_map: &HashMap<i64, PlayerInfo>,
    ) -> Result<()> {
        for game in games {
            let first_player_info = player_info_map.get(&game.winner_id)
                .ok_or_else(|| anyhow::anyhow!("Winner not found in player_info_map"))?;
            let second_player_info = player_info_map.get(&game.loser_id)
                .ok_or_else(|| anyhow::anyhow!("Loser not found in player_info_map"))?;

            let first_player_db = Self::upsert_player(conn, first_player_info)?;
            let second_player_db = Self::upsert_player(conn, second_player_info)?;

            GameRepository::insert_game(
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

    /// Apply time decay weights to games based on current date
    pub fn apply_time_decay(games: &mut [ExpandedGame]) {
        let current_date = Utc::now().naive_utc();
        rating::weighting::apply_weights_to_games(games, current_date);
    }

    /// Check if player name indicates team play
    fn is_team_player(&self, name: &str) -> bool {
        let lower = name.to_lowercase();

        // Check for separator characters
        let has_separator = self.settings.team_player_separators
            .iter()
            .any(|sep| name.contains(sep));

        // Check for team prefixes
        let has_prefix = self.settings.team_player_prefixes
            .iter()
            .any(|prefix| lower.starts_with(prefix));

        has_separator || has_prefix
    }

    /// Upsert player record from player info
    fn upsert_player(
        conn: &mut DbConn,
        player_info: &PlayerInfo,
    ) -> Result<crate::database::Player> {
        let cuescore_id = player_info.player_id
            .ok_or_else(|| anyhow::anyhow!("Missing cuescore_id for player: {}", player_info.name))?;
        let name = &player_info.name;
        let avatar_url = player_info.image.as_deref();
        PlayerRepository::upsert_player(conn, cuescore_id, name, avatar_url)
    }
}
