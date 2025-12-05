use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json},
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::Deserialize;
use std::sync::Arc;
use urlencoding::encode;

use crate::api::models::{PlayerListItem, PlayerListResponse, PlayerDetail};
use crate::services::ingestion::IngestionService;
use crate::services::processing::ProcessingService;
use crate::config::settings::AppConfig;
use crate::database::{self, models::{PlayerFilter, SortColumn, SortOrder}};

pub struct AppState {
    pub pool: Pool<SqliteConnectionManager>,
    pub config: AppConfig,
}

#[derive(Deserialize)]
pub struct PlayerParams {
    page: Option<usize>,
    page_size: Option<usize>,
    sort_by: Option<String>,
    order: Option<String>,
    filter: Option<String>,
    rating_type: Option<String>,
}

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
        PlayerListItem {
            rank: (offset + i + 1) as i32,
            player_id: row.player_id as i64,
            cuescore_id: row.cuescore_id,
            name: row.name,
            rating: row.rating,
            games_played: row.games_played,
            confidence_level: row.confidence_level,
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
                rating: row.rating,
                games_played: row.games_played,
                confidence_level: row.confidence_level,
                ml_rating,
                starter_weight,
                ml_weight,
                effective_games: row.games_played,
                last_played,
            }).into_response()
        },
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn admin_refresh(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    if auth_header != Some("Bearer secret") {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    
    tokio::spawn(async move {
        log::info!("Admin triggered refresh started");
        let ingest_result = async {
            let mut ingest_service = IngestionService::new()?;
            ingest_service.run().await
        }.await;
        if let Err(e) = ingest_result {
            log::error!("Refresh failed at ingestion: {:?}", e);
            return;
        }
        let process_result = async {
            let process_service = ProcessingService::new(state.config.clone())?;
            process_service.run()
        }.await;
        if let Err(e) = process_result {
            log::error!("Refresh failed at processing: {:?}", e);
            return;
        }
        log::info!("Admin triggered refresh completed successfully");
    });

    (StatusCode::ACCEPTED, "Refresh triggered").into_response()
}
