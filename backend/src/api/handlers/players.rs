use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use std::sync::Arc;
use urlencoding::encode;
use chrono::{Duration, Utc, NaiveDateTime};

use crate::api::models::{
    PlayerListItem, PlayerListResponse, PlayerDetail, HeadToHeadMatch, HeadToHeadResponse, HeadToHeadStats, MatchResult,
    PlayerRivalriesResponse, RivalryEntry
};
use crate::api::filter::{parser::parse_filter_dsl, sql_builder::build_sql_filter};
use crate::database::{self, models::{PlayerFilter, SortColumn, SortOrder}, repositories::{player_repository::PlayerRepository, game_repository::GameRepository}};
use crate::errors::AppError;
use crate::services::player_service;
use super::{AppState, PlayerParams};

pub async fn get_players(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PlayerParams>,
) -> Result<Json<PlayerListResponse>, AppError> {
    // Parse filter DSL
    let filter_exprs = parse_filter_dsl(&params.filter.unwrap_or_default())
        .map_err(|e| AppError::BadRequest(format!("Invalid filter syntax: {}", e)))?;

    // Build SQL filter
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

    let mut actual_rating_type = params.rating_type.unwrap_or_else(|| "all".to_string());
    let mut last_played_cutoff: Option<NaiveDateTime> = None;

    if actual_rating_type == "active" {
        // For "active" ranking, we still query the "all" time rating, but filter by activity
        actual_rating_type = "all".to_string();
        let six_months_ago = Utc::now().naive_utc() - Duration::days(6 * 30); // Approximate 6 months
        last_played_cutoff = Some(six_months_ago);
    }

    let mut conn = state.pool.get().map_err(|_| AppError::InternalServerError)?;

    let filter = PlayerFilter {
        sql_filter,
        min_games: Some(state.config.rating.min_ranked_games),
        last_played_cutoff, // Pass the new cutoff
        sort_by,
        sort_order,
        limit: page_size,
        offset,
    };

    let (rows, total) = PlayerRepository::list_ranked_players(&mut conn, &filter)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let players: Vec<PlayerListItem> = rows.into_iter().enumerate().map(|(i, row)| {
        let matches_played = GameRepository::count_matches_played_for_player(&mut conn, row.player_id)
            .unwrap_or(0); // Consider handling this error more robustly if it's critical
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
    let rating_type = params.rating_type.unwrap_or_else(|| "all".to_string());

    let mut conn = state.pool.get().map_err(|_| AppError::InternalServerError)?;

    let player_data = PlayerRepository::get_player_rating_detail(&mut conn, player_id as i32, &rating_type)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    match player_data {
        Some(row) => {
            let established_games = state.config.rating.established_games;
            let starter_rating = state.config.rating.starter_rating;

            let (ml_rating, starter_weight, ml_weight) =
                player_service::calculate_adjusted_ratings(row.games_played, row.rating, established_games, starter_rating);

            let last_played = PlayerRepository::get_player_last_match_date(&mut conn, row.player_id)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            let matches_played = PlayerRepository::count_player_distinct_matches_played(&mut conn, row.player_id)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            let last_matches_rows = GameRepository::get_player_last_matches(&mut conn, row.player_id, 10)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            let last_matches: Vec<MatchResult> = last_matches_rows.into_iter().map(|m| MatchResult {
                date: m.date.to_string(),
                tournament_name: m.tournament_name,
                opponent_name: m.opponent_name,
                opponent_id: m.opponent_id as i64,
                player_total_score: m.player_total_score,
                opponent_total_score: m.opponent_total_score,
            }).collect();

            let encoded_name = encode(&row.name).replace(' ', "+");
            let cuescore_profile_url = format!(
                "https://cuescore.com/player/{}/{}",
                encoded_name,
                row.cuescore_id
            );

            Ok(Json(PlayerDetail {
                player_id: row.player_id as i64,
                cuescore_id: row.cuescore_id,
                name: row.name,
                cuescore_profile_url,
                rating: row.rating,
                games_played: row.games_played,
                confidence_level: row.confidence_level,
                ml_rating,
                starter_weight,
                ml_weight,
                effective_games: row.games_played,
                last_played,
                matches_played,
                last_matches,
            }))
        },
        None => Err(AppError::NotFound(format!("Player {} not found", player_id))),
    }
}

pub async fn get_head_to_head_comparison(
    State(state): State<Arc<AppState>>,
    Path((player1_id, player2_id)): Path<(i64, i64)>,
    Query(params): Query<PlayerParams>,
) -> Result<Json<HeadToHeadResponse>, AppError> {
    let rating_type = params.rating_type.unwrap_or_else(|| "all".to_string());

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

    let mut get_full_player_detail = |p: database::models::PlayerWithRating| -> Result<PlayerDetail, AppError> {
        let established_games = state.config.rating.established_games;
        let starter_rating = state.config.rating.starter_rating;

        let (ml_rating, starter_weight, ml_weight) =
            player_service::calculate_adjusted_ratings(p.games_played, p.rating, established_games, starter_rating);

        let last_played = PlayerRepository::get_player_last_match_date(&mut conn, p.player_id)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let matches_played = PlayerRepository::count_player_distinct_matches_played(&mut conn, p.player_id)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let encoded_name = encode(&p.name).replace(' ', "+");
        let cuescore_profile_url = format!("https://cuescore.com/player/{}/{}", encoded_name, p.cuescore_id);

        Ok(PlayerDetail {
            player_id: p.player_id as i64,
            cuescore_id: p.cuescore_id,
            name: p.name,
            cuescore_profile_url,
            rating: p.rating,
            games_played: p.games_played,
            confidence_level: p.confidence_level,
            ml_rating,
            starter_weight,
            ml_weight,
            effective_games: p.games_played,
            last_played,
            matches_played,
            last_matches: vec![],
        })
    };

    let player1_api_detail = get_full_player_detail(player1_detail_data)?;
    let player2_api_detail = get_full_player_detail(player2_detail_data)?;

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
