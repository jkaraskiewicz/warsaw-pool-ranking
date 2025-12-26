use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use std::sync::Arc;
use chrono::{Duration, Utc, NaiveDateTime};

use crate::api::models::{
    PlayerListItem, PlayerListResponse, PlayerDetail, HeadToHeadMatch, HeadToHeadResponse, HeadToHeadStats,
    PlayerRivalriesResponse, RivalryEntry
};
use crate::api::filter::{parser::parse_filter_dsl, sql_builder::build_sql_filter, FilterValue};
use crate::database::models::{PlayerFilter, SortColumn, SortOrder};
use crate::database::repositories::{player_repository::PlayerRepository, game_repository::GameRepository};
use crate::errors::AppError;
use crate::services::player_service;
use super::{AppState, PlayerParams};

pub async fn get_players(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PlayerParams>,
) -> Result<Json<PlayerListResponse>, AppError> {
    // Parse filter DSL
    let mut filter_exprs = parse_filter_dsl(&params.filter.unwrap_or_default())
        .map_err(|e| AppError::BadRequest(format!("Invalid filter syntax: {}", e)))?;

    let mut last_played_cutoff: Option<NaiveDateTime> = None;

    // Check for "active" rating type in filters and adjust
    for expr in &mut filter_exprs {
        if expr.field == "rating_type"
            && let FilterValue::Single(ref mut val) = expr.value
            && val == "active"
        {
            *val = "all".to_string(); // Switch to 'all' rating type for DB query
            let six_months_ago = Utc::now().naive_utc() - Duration::days(6 * 30);
            last_played_cutoff = Some(six_months_ago);
        }
    }

    // Build SQL filter (using modified expressions)
    let sql_filter = build_sql_filter(filter_exprs)
        .map_err(|e| AppError::BadRequest(format!("Invalid filter: {}", e)))?;

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(100).clamp(1, 1000);
    let offset = (page - 1) * page_size;

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

    let mut conn = state.pool.get().map_err(|_| AppError::InternalServerError)?;

    let filter = PlayerFilter {
        sql_filter,
        min_games: Some(state.config.rating.min_ranked_games),
        last_played_cutoff,
        sort_by,
        sort_order,
        limit: page_size,
        offset,
    };

    let (rows, total) = PlayerRepository::list_ranked_players(&mut conn, &filter)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Collect player IDs for batch query
    let player_ids: Vec<i32> = rows.iter().map(|row| row.player_id).collect();

    // Batch fetch match counts for all players (single query instead of N+1)
    let match_counts = GameRepository::count_matches_for_players(&mut conn, &player_ids)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Map to PlayerListItem using the batch results
    let players: Vec<PlayerListItem> = rows.into_iter().enumerate().map(|(i, row)| {
        let matches_played = match_counts.get(&row.player_id).copied().unwrap_or(0);
        PlayerListItem {
            rank: (offset + i + 1) as i32,
            player_id: row.player_id as i64,
            cuescore_id: row.cuescore_id,
            name: row.name,
            rating: row.rating,
            games_played: row.games_played,
            confidence_level: row.confidence_level,
            matches_played,
        }
    }).collect();

    Ok(Json(PlayerListResponse {
        items: players,
        total: total as i32,
        page: page as i32,
        page_size: page_size as i32,
    }))
}

pub async fn get_player_detail(
    State(state): State<Arc<AppState>>,
    Path(player_id): Path<i64>,
    Query(params): Query<PlayerParams>,
) -> Result<Json<PlayerDetail>, AppError> {
    let mut rating_type = params.rating_type.unwrap_or_else(|| "all".to_string());
    if rating_type == "active" {
        rating_type = "all".to_string();
    }

    let mut conn = state.pool.get().map_err(|_| AppError::InternalServerError)?;

    let player_data = PlayerRepository::get_player_rating_detail(&mut conn, player_id as i32, &rating_type)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    match player_data {
        Some(row) => {
            let established_games = state.config.rating.established_games;
            let starter_rating = state.config.rating.starter_rating;

            let detail = player_service::build_player_detail(
                &mut conn,
                row,
                established_games,
                starter_rating,
                true, // include last matches
            )?;

            Ok(Json(detail))
        },
        None => Err(AppError::NotFound(format!("Player {} not found", player_id))),
    }
}

pub async fn get_head_to_head_comparison(
    State(state): State<Arc<AppState>>,
    Path((player1_id, player2_id)): Path<(i64, i64)>,
    Query(params): Query<PlayerParams>,
) -> Result<Json<HeadToHeadResponse>, AppError> {
    let mut rating_type = params.rating_type.unwrap_or_else(|| "all".to_string());
    if rating_type == "active" {
        rating_type = "all".to_string();
    }

    let mut conn = state.pool.get().map_err(|_| AppError::InternalServerError)?;

    let player1_detail_data = PlayerRepository::get_player_rating_detail(&mut conn, player1_id as i32, &rating_type)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Player 1 ({}) not found", player1_id)))?;
    let player2_detail_data = PlayerRepository::get_player_rating_detail(&mut conn, player2_id as i32, &rating_type)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Player 2 ({}) not found", player2_id)))?;

    let rating_diff = player1_detail_data.rating - player2_detail_data.rating;
    let probability_p1_wins = 1.0 / (1.0 + (-rating_diff * std::f64::consts::LN_2 / 100.0).exp());

    let matches = GameRepository::get_head_to_head_matches(&mut conn, player1_detail_data.player_id, player2_detail_data.player_id)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

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

    let established_games = state.config.rating.established_games;
    let starter_rating = state.config.rating.starter_rating;

    let player1_api_detail = player_service::build_player_detail(
        &mut conn,
        player1_detail_data,
        established_games,
        starter_rating,
        false, // no last matches for comparison view
    )?;

    let player2_api_detail = player_service::build_player_detail(
        &mut conn,
        player2_detail_data,
        established_games,
        starter_rating,
        false, // no last matches for comparison view
    )?;

    Ok(Json(HeadToHeadResponse {
        player1: Some(player1_api_detail),
        player2: Some(player2_api_detail),
        probability_p1_wins,
        matches: h2h_matches,
        stats: Some(stats),
    }))
}

pub async fn get_player_rivalries(
    State(state): State<Arc<AppState>>,
    Path(player_id): Path<i64>,
) -> Result<Json<PlayerRivalriesResponse>, AppError> {
    let mut conn = state.pool.get().map_err(|_| AppError::InternalServerError)?;

    // Minimum games filter - could be configurable or query param
    let min_games = 5;

    let rivalries = GameRepository::get_player_rivalries(&mut conn, player_id as i32, min_games)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Map to RivalryEntry DTO intermediate struct with win_rate
    let mut entries: Vec<RivalryEntry> = rivalries.into_iter().map(|r| {
        let win_rate = if r.matches_played > 0 {
            r.matches_won as f64 / r.matches_played as f64
        } else {
            0.0
        };
        RivalryEntry {
            opponent_id: r.opponent_id as i64,
            opponent_name: r.opponent_name,
            matches_played: r.matches_played,
            matches_won: r.matches_won,
            win_rate,
        }
    }).collect();

    // Nemeses: Lowest win rate (ascending), then highest matches played (descending)
    entries.sort_by(|a, b| {
        a.win_rate.partial_cmp(&b.win_rate).unwrap()
            .then(b.matches_played.cmp(&a.matches_played))
    });
    let nemeses = entries.iter().take(5).cloned().collect();

    // Bunnies: Highest win rate (descending), then highest matches played (descending)
    entries.sort_by(|a, b| {
        b.win_rate.partial_cmp(&a.win_rate).unwrap()
            .then(b.matches_played.cmp(&a.matches_played))
    });
    let bunnies = entries.iter().take(5).cloned().collect();

    Ok(Json(PlayerRivalriesResponse {
        nemeses,
        bunnies,
    }))
}
