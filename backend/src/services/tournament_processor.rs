use anyhow::Result;
use std::collections::HashMap;
use chrono::{NaiveDateTime, DateTime};

use crate::config::settings::TournamentProcessingSettings;
use crate::database::{DbConn, Tournament};
use crate::database::repositories::tournament_repository::TournamentRepository;
use crate::fetchers::cuescore_models::{TournamentResponse, PlayerInfo};
use crate::domain::ExpandedGame;

pub struct TournamentProcessor {
    settings: TournamentProcessingSettings,
}

impl TournamentProcessor {
    pub fn new(settings: TournamentProcessingSettings) -> Self {
        Self { settings }
    }

    /// Insert tournament to database and return the DB record
    pub fn insert_tournament(
        &self,
        conn: &mut DbConn,
        tournament: &TournamentResponse,
    ) -> Result<Tournament> {
        let start_date = Self::parse_date(&tournament.starttime)?;
        let end_date = Self::parse_optional_date(&tournament.stoptime)?;

        TournamentRepository::upsert_tournament(
            conn,
            tournament.id,
            &tournament.name,
            tournament.venue_id(),
            &tournament.venue_name(),
            start_date,
            end_date,
        )
    }

    /// Extract player info map from tournament data
    pub fn extract_player_info(&self, tournament: &TournamentResponse) -> HashMap<i64, PlayerInfo> {
        let mut players = HashMap::new();

        for match_data in &tournament.matches {
            if match_data.is_played() {
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

    /// Expand tournament to individual games
    pub fn expand_to_games(&self, tournament: &TournamentResponse) -> Result<Vec<ExpandedGame>> {
        crate::domain::games_expansion::expand_tournament_to_games(tournament)
    }

    /// Check if tournament is doubles/team format (should be skipped)
    pub fn is_doubles_tournament(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        self.settings.doubles_keywords.iter().any(|keyword| lower.contains(keyword))
    }

    /// Parse tournament date string to NaiveDateTime
    fn parse_date(date_str: &str) -> Result<NaiveDateTime> {
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Ok(dt.naive_utc());
        }

        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
            return Ok(dt);
        }

        if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.f") {
            return Ok(dt);
        }

        anyhow::bail!("Failed to parse tournament date: {}", date_str)
    }

    /// Parse optional tournament date
    fn parse_optional_date(date_str: &Option<String>) -> Result<Option<NaiveDateTime>> {
        match date_str {
            Some(s) => Ok(Some(Self::parse_date(s)?)),
            None => Ok(None),
        }
    }
}
