use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use urlencoding::encode;

use crate::api::models::{PlayerListItem, PlayerListResponse, PlayerDetail, HeadToHeadMatch, HeadToHeadResponse, HeadToHeadStats};
use crate::database::{self, models::{PlayerFilter, SortColumn, SortOrder}};
use super::{AppState, PlayerParams};

pub async fn get_players(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PlayerParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(100).clamp(1, 1000);
    let offset = (page - 1) * page_size;
    let rating_type = params.rating_type.unwrap_or_else(|| "all".to_string());

    let sort_by = match params.sort_by.as_deref() {
        Some("name") => SortColumn::Name,
        Some("rating") => SortColumn::Rating,
        Some("gamesPlayed") => SortColumn::GamesPlayed,
        _ => SortColumn::Rating,
    };

    let sort_order = match params.order.as_deref() {
        Some("asc") => SortOrder::Asc,
        _ => SortOrder::Desc,
    };

    let mut conn = match state.pool.get() {
        Ok(conn) => conn,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "DB Connection Error").into_response(),
    };

    let filter = PlayerFilter {
        name_contains: params.filter,
        min_games: Some(state.config.rating.min_ranked_games),
        rating_type,
        sort_by,
        sort_order,
        limit: page_size,
        offset,
    };

    let (rows, total) = match database::ratings::list_ranked_players(&mut conn, &filter) {
        Ok(result) => result,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Query Error: {}", e)).into_response(),
    };

    let players: Vec<PlayerListItem> = rows.into_iter().enumerate().map(|(i, row)| {
        let player_id_i32 = row.player_id;
        let matches_played = database::games::count_matches_played_for_player(&mut conn, player_id_i32).unwrap_or(0);
        PlayerListItem {
            rank: (offset + i + 1) as i32,
            player_id: row.player_id as i64,
            cuescore_id: row.cuescore_id,
            name: row.name,
            avatar_url: row.avatar_url,
            rating: row.rating,
            games_played: row.games_played,
            confidence_level: row.confidence_level,
            matches_played,
        }
    }).collect();

    Json(PlayerListResponse {
        items: players,
        total: total as i32,
        page: page as i32,
        page_size: page_size as i32,
    }).into_response()
}

pub async fn get_player_detail(
    State(state): State<Arc<AppState>>,
    Path(player_id): Path<i64>,
    Query(params): Query<PlayerParams>,
) -> impl IntoResponse {
    let rating_type = params.rating_type.unwrap_or_else(|| "all".to_string());

    let mut conn = match state.pool.get() {
        Ok(conn) => conn,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "DB Connection Error").into_response(),
    };

    let player_data = match database::ratings::get_player_rating_detail(&mut conn, player_id as i32, &rating_type) {
        Ok(data) => data,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Query Error: {}", e)).into_response(),
    };

    match player_data {
        Some(row) => {
            let established_games = state.config.rating.established_games;
            let starter_rating = state.config.rating.starter_rating;

            let starter_weight = if row.games_played >= established_games {
                0.0
            } else {
                (established_games - row.games_played) as f64 / established_games as f64
            };
            let ml_weight = 1.0 - starter_weight;

            let ml_rating = if ml_weight > 0.0001 {
                (row.rating - (starter_weight * starter_rating)) / ml_weight
            } else {
                row.rating
            };

            let last_played: Option<String> = conn.query_row(
                "SELECT MAX(date) FROM games WHERE first_player_id = ?1 OR second_player_id = ?1",
                rusqlite::params![row.player_id],
                |r| r.get(0)
            ).ok();

            let matches_played: i32 = conn.query_row(
                "SELECT COUNT(DISTINCT date) FROM games WHERE first_player_id = ?1 OR second_player_id = ?1",
                rusqlite::params![row.player_id],
                |r| r.get(0)
            ).unwrap_or(0);

            let encoded_name = encode(&row.name).replace(' ', "+");
            let cuescore_profile_url = format!(
                "https://cuescore.com/player/{}/{}",
                encoded_name,
                row.cuescore_id.unwrap_or(0)
            );

            Json(PlayerDetail {
                player_id: row.player_id as i64,
                cuescore_id: row.cuescore_id,
                name: row.name,
                cuescore_profile_url,
                avatar_url: row.avatar_url,
                rating: row.rating,
                games_played: row.games_played,
                confidence_level: row.confidence_level,
                ml_rating,
                starter_weight,
                ml_weight,
                effective_games: row.games_played,
                last_played,
                matches_played,
            }).into_response()
        },
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_head_to_head_comparison(
    State(state): State<Arc<AppState>>,
    Path((player1_id, player2_id)): Path<(i64, i64)>,
    Query(params): Query<PlayerParams>,
) -> impl IntoResponse {
    let rating_type = params.rating_type.unwrap_or_else(|| "all".to_string());

    let mut conn = match state.pool.get() {
        Ok(conn) => conn,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "DB Connection Error").into_response(),
    };

    let player1_detail_data = match database::ratings::get_player_rating_detail(&mut conn, player1_id as i32, &rating_type) {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Player 1 ({}) not found", player1_id)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Query Error for Player 1: {}", e)).into_response(),
    };
    let player2_detail_data = match database::ratings::get_player_rating_detail(&mut conn, player2_id as i32, &rating_type) {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, format!("Player 2 ({}) not found", player2_id)).into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Query Error for Player 2: {}", e)).into_response(),
    };

    let rating_diff = player1_detail_data.rating - player2_detail_data.rating;
    let probability_p1_wins = 1.0 / (1.0 + (-rating_diff * std::f64::consts::LN_2 / 100.0).exp());

    let matches = match database::games::get_head_to_head_matches(&mut conn, player1_detail_data.player_id, player2_detail_data.player_id) {
        Ok(m) => m,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Query Error for matches: {}", e)).into_response(),
    };

    let mut stats = HeadToHeadStats {
        total_matches: matches.len() as i32,
        player1_match_wins: 0,
        player2_match_wins: 0,
        total_frames: 0,
        player1_frame_wins: 0,
        player2_frame_wins: 0,
    };

    let h2h_matches: Vec<HeadToHeadMatch> = matches.iter().map(|m| {
        stats.total_frames += m.p1_wins + m.p2_wins;
        stats.player1_frame_wins += m.p1_wins;
        stats.player2_frame_wins += m.p2_wins;
        if m.p1_wins > m.p2_wins {
            stats.player1_match_wins += 1;
        } else if m.p2_wins > m.p1_wins {
            stats.player2_match_wins += 1;
        }

        HeadToHeadMatch {
            date: m.date.to_string(),
            tournament_name: m.tournament_name.clone(),
            player1_wins: m.p1_wins,
            player2_wins: m.p2_wins,
        }
    }).collect();

    let get_full_player_detail = |p: database::models::PlayerWithRating| -> PlayerDetail {
        let established_games = state.config.rating.established_games;
        let starter_rating = state.config.rating.starter_rating;

        let starter_weight = if p.games_played >= established_games { 0.0 } else { (established_games - p.games_played) as f64 / established_games as f64 };
        let ml_weight = 1.0 - starter_weight;

        let ml_rating = if ml_weight > 0.0001 { (p.rating - (starter_weight * starter_rating)) / ml_weight } else { p.rating };

        let last_played: Option<String> = conn.query_row(
            "SELECT MAX(date) FROM games WHERE first_player_id = ?1 OR second_player_id = ?1",
            rusqlite::params![p.player_id],
            |r| r.get(0)
        ).ok();

        let matches_played: i32 = conn.query_row(
            "SELECT COUNT(DISTINCT date) FROM games WHERE first_player_id = ?1 OR second_player_id = ?1",
            rusqlite::params![p.player_id],
            |r| r.get(0)
        ).unwrap_or(0);

        let encoded_name = encode(&p.name).replace(' ', "+");
        let cuescore_profile_url = format!("https://cuescore.com/player/{}/{}", encoded_name, p.cuescore_id.unwrap_or(0));

        PlayerDetail {
            player_id: p.player_id as i64,
            cuescore_id: p.cuescore_id,
            name: p.name,
            cuescore_profile_url,
            avatar_url: p.avatar_url,
            rating: p.rating,
            games_played: p.games_played,
            confidence_level: p.confidence_level,
            ml_rating,
            starter_weight,
            ml_weight,
            effective_games: p.games_played,
            last_played,
            matches_played,
        }
    };

    let player1_api_detail = get_full_player_detail(player1_detail_data);
    let player2_api_detail = get_full_player_detail(player2_detail_data);

    Json(HeadToHeadResponse {
        player1: Some(player1_api_detail),
        player2: Some(player2_api_detail),
        probability_p1_wins,
        matches: h2h_matches,
        stats: Some(stats),
    }).into_response()
}
