use urlencoding::encode;
use crate::api::models::{PlayerDetail, MatchResult};
use crate::database::models::PlayerWithRating;
use crate::database::repositories::{player_repository::PlayerRepository, game_repository::GameRepository};
use crate::database::connection::DbConn;
use crate::errors::AppError;

pub fn calculate_adjusted_ratings(
    games_played: i32,
    rating: f64,
    established_games: i32,
    starter_rating: f64,
) -> (f64, f64, f64) {
    let starter_weight = if games_played >= established_games {
        0.0
    } else {
        (established_games - games_played) as f64 / established_games as f64
    };
    let ml_weight = 1.0 - starter_weight;

    let ml_rating = if ml_weight > 0.0001 {
        (rating - (starter_weight * starter_rating)) / ml_weight
    } else {
        rating
    };
    (ml_rating, starter_weight, ml_weight)
}

/// Build a complete PlayerDetail from PlayerWithRating data
/// Fetches additional data (last played, matches count) from database
pub fn build_player_detail(
    conn: &mut DbConn,
    player_data: PlayerWithRating,
    established_games: i32,
    starter_rating: f64,
    include_last_matches: bool,
) -> Result<PlayerDetail, AppError> {
    let (ml_rating, starter_weight, ml_weight) =
        calculate_adjusted_ratings(player_data.games_played, player_data.rating, established_games, starter_rating);

    let last_played = PlayerRepository::get_player_last_match_date(conn, player_data.player_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let matches_played = GameRepository::count_matches_played_for_player(conn, player_data.player_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let last_matches = if include_last_matches {
        let last_matches_rows = GameRepository::get_player_last_matches(conn, player_data.player_id, 10)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        last_matches_rows.into_iter().map(|m| MatchResult {
            date: m.date.to_string(),
            tournament_name: m.tournament_name,
            opponent_name: m.opponent_name,
            opponent_id: m.opponent_id as i64,
            player_total_score: m.player_total_score,
            opponent_total_score: m.opponent_total_score,
        }).collect()
    } else {
        vec![]
    };

    let encoded_name = encode(&player_data.name).replace(' ', "+");
    let cuescore_profile_url = format!(
        "https://cuescore.com/player/{}/{}",
        encoded_name,
        player_data.cuescore_id
    );

    Ok(PlayerDetail {
        player_id: player_data.player_id as i64,
        cuescore_id: player_data.cuescore_id,
        name: player_data.name,
        cuescore_profile_url,
        rating: player_data.rating,
        games_played: player_data.games_played,
        confidence_level: player_data.confidence_level,
        ml_rating,
        starter_weight,
        ml_weight,
        effective_games: player_data.games_played,
        last_played,
        matches_played,
        last_matches,
    })
}
