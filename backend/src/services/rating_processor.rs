use anyhow::Result;
use chrono::Utc;
use log::error;

use crate::config::settings::RatingSettings;
use crate::database::DbConn;
use crate::database::repositories::{rating_repository::RatingRepository, player_repository::PlayerRepository};
use crate::domain::ExpandedGame;
use crate::rating;

pub struct RatingProcessor;

impl RatingProcessor {
    /// Calculate player ratings from games
    pub fn calculate_ratings(
        games: &[ExpandedGame],
        rating_type: &str,
        settings: &RatingSettings,
    ) -> Result<Vec<rating::PlayerRating>> {
        let game_results = Self::convert_to_game_results(games);
        let mut ratings = rating::calculate_ratings(&game_results, settings);

        // Set rating type for all results
        for r in &mut ratings {
            r.rating_type = rating_type.to_string();
        }

        Ok(ratings)
    }

    /// Save ratings to database
    pub fn save_ratings(
        conn: &mut DbConn,
        ratings: &[rating::PlayerRating],
        rating_type: &str,
    ) -> Result<()> {
        let calculated_at = Utc::now().naive_utc();

        for player_rating in ratings {
            let cuescore_id = player_rating.player_id as i64;

            // Upsert player record (may not exist if rating calculated before player inserted)
            let player = PlayerRepository::upsert_player(conn, cuescore_id, "Unknown Player", None)?;

            if let Err(e) = RatingRepository::insert_rating(
                conn,
                player.id,
                rating_type,
                player_rating.rating,
                player_rating.games_played,
                player_rating.confidence_level.as_str(),
                calculated_at,
            ) {
                error!("Failed to insert rating for player {}: {:?}", player.id, e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Convert expanded games to rating game results
    fn convert_to_game_results(games: &[ExpandedGame]) -> Vec<rating::GameResult> {
        games
            .iter()
            .map(|g| rating::GameResult {
                winner_id: g.winner_id as i32,
                loser_id: g.loser_id as i32,
                weight: g.weight,
            })
            .collect()
    }
}
