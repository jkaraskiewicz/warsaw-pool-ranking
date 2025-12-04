use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json},
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::Deserialize;
use std::sync::Arc;

use crate::api::models::{PlayerListItem, PaginatedResponse, PlayerDetail};
use crate::services::ingestion::IngestionService;
use crate::services::processing::ProcessingService;
use crate::config::settings::AppConfig;

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

    let sort_column = match params.sort_by.as_deref() {
        Some("name") => "p.name",
        Some("rating") => "r.rating",
        Some("gamesPlayed") => "r.games_played", // Frontend sends camelCase
        _ => "r.rating", // Default sort
    };

    let sort_order = match params.order.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };

    let conn = match state.pool.get() {
        Ok(conn) => conn,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "DB Connection Error").into_response(),
    };

    // Build query
    let mut where_clauses = Vec::new();
    let mut sql_params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    // Filter by minimum ranked games
    where_clauses.push("r.games_played >= ?");
    sql_params.push(Box::new(state.config.rating.min_ranked_games));

    // Filter by rating type
    where_clauses.push("r.rating_type = ?");
    sql_params.push(Box::new(rating_type.clone()));

    if let Some(filter) = &params.filter {
        where_clauses.push("p.name LIKE ?");
        sql_params.push(Box::new(format!("%{}%", filter)));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Count total
    let count_sql = format!(
        "SELECT COUNT(*) FROM players p JOIN ratings r ON p.id = r.player_id {}",
        where_sql
    );
    
    let total: usize = conn.query_row(&count_sql, rusqlite::params_from_iter(sql_params.iter()), |row| row.get(0)).unwrap_or(0);

    // Fetch data
    let sql = format!(
        "SELECT p.id, p.cuescore_id, p.name, r.rating, r.games_played, r.confidence_level 
         FROM players p 
         JOIN ratings r ON p.id = r.player_id 
         {} 
         ORDER BY {} {} 
         LIMIT ? OFFSET ?",
        where_sql, sort_column, sort_order
    );

    // Append limit/offset to params
    sql_params.push(Box::new(page_size as i64));
    sql_params.push(Box::new(offset as i64));

    let mut stmt = match conn.prepare(&sql) {
        Ok(stmt) => stmt,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("SQL Error: {}", e)).into_response(),
    };

    let players_iter = stmt.query_map(rusqlite::params_from_iter(sql_params.iter()), |row| {
        Ok(PlayerListItem {
            rank: 0, // Placeholder, updated below
            player_id: row.get(0)?,
            cuescore_id: row.get(1)?,
            name: row.get(2)?,
            rating: row.get(3)?,
            games_played: row.get(4)?,
            confidence_level: row.get(5)?,
        })
    });

    let mut players = match players_iter {
        Ok(iter) => iter.filter_map(Result::ok).collect::<Vec<_>>(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Query Error").into_response(),
    };

    // Update rank
    for (i, player) in players.iter_mut().enumerate() {
        player.rank = offset + i + 1;
    }

    Json(PaginatedResponse {
        items: players,
        total,
        page,
        page_size,
    }).into_response()
}

pub async fn get_player_detail(
    State(state): State<Arc<AppState>>,
    Path(player_id): Path<i64>,
    Query(params): Query<PlayerParams>,
) -> impl IntoResponse {
    let rating_type = params.rating_type.unwrap_or_else(|| "all".to_string());

    let conn = match state.pool.get() {
        Ok(conn) => conn,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "DB Connection Error").into_response(),
    };

    let sql = "
        SELECT p.id, p.cuescore_id, p.name, r.rating, r.games_played, r.confidence_level 
        FROM players p 
        JOIN ratings r ON p.id = r.player_id 
        WHERE p.id = ?1 AND r.rating_type = ?2
    ";

    let mut stmt = match conn.prepare(sql) {
        Ok(stmt) => stmt,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("SQL Error: {}", e)).into_response(),
    };

    let player_result = stmt.query_row(rusqlite::params![player_id, rating_type], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<i64>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, f64>(3)?,
            row.get::<_, i32>(4)?,
            row.get::<_, String>(5)?,
        ))
    });

    match player_result {
        Ok((id, cuescore_id, name, rating, games_played, confidence_level)) => {
            // Calculate blending details
            let established_games = state.config.rating.established_games;
            let starter_rating = state.config.rating.starter_rating;
            
            let starter_weight = if games_played >= established_games {
                0.0
            } else {
                (established_games - games_played) as f64 / established_games as f64
            };
            let ml_weight = 1.0 - starter_weight;
            
            let ml_rating = if ml_weight > 0.0001 {
                (rating - (starter_weight * starter_rating)) / ml_weight
            } else {
                rating // Pure starter rating
            };

            // Get last played date
            let last_played: Option<String> = conn.query_row(
                "SELECT MAX(date) FROM games WHERE first_player_id = ?1 OR second_player_id = ?1",
                rusqlite::params![id],
                |row| row.get(0)
            ).ok();

            Json(PlayerDetail {
                player_id: id,
                cuescore_id,
                name,
                rating,
                games_played,
                confidence_level,
                ml_rating,
                starter_weight,
                ml_weight,
                effective_games: games_played,
                last_played,
                recent_change: None,
            }).into_response()
        },
        Err(rusqlite::Error::QueryReturnedNoRows) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Query Error: {}", e)).into_response(),
    }
}

pub async fn admin_refresh(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Simple Auth Check
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    // In production, use a secure compare and env var. For now, hardcoded "secret".
    if auth_header != Some("Bearer secret") {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // Run Ingestion and Processing in background or blocking?
    // Since this might take time, ideally spawn a task.
    // But for simplicity, we'll block (or await async).
    // `IngestionService` is async. `ProcessingService` is sync (mostly).
    
    tokio::spawn(async move {
        log::info!("Admin triggered refresh started");
        
        // 1. Ingest
        let ingest_result = async {
            let mut ingest_service = IngestionService::new()?;
            ingest_service.run().await
        }.await;

        if let Err(e) = ingest_result {
            log::error!("Refresh failed at ingestion: {:?}", e);
            return;
        }

        // 2. Process (Atomic)
        let process_result = async {
            let process_service = ProcessingService::new(state.config.clone())?;
            process_service.run() // This handles atomic swap
        }.await;

        if let Err(e) = process_result {
            log::error!("Refresh failed at processing: {:?}", e);
            return;
        }
        
        log::info!("Admin triggered refresh completed successfully");
    });

    (StatusCode::ACCEPTED, "Refresh triggered").into_response()
}
