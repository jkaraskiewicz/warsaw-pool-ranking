use axum::{
    extract::{Query, State},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;
use log;

use crate::database;
use crate::services::avatar_processor::AvatarProcessor;
use crate::services::ingestion::IngestionService;
use crate::services::processing::ProcessingService;
use super::AppState;

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

#[derive(Deserialize)]
pub struct AvatarRefreshParams {
    player_id: Option<i32>,
}

pub async fn admin_refresh_avatars(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<AvatarRefreshParams>,
) -> impl IntoResponse {
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());
    if auth_header != Some("Bearer secret") {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    tokio::spawn(async move {
        log::info!("Admin triggered avatar refresh started");
        let result = refresh_avatars_task(&state, params.player_id).await;
        if let Err(e) = result {
            log::error!("Avatar refresh failed: {:?}", e);
            return;
        }
        log::info!("Avatar refresh completed successfully");
    });

    (StatusCode::ACCEPTED, "Avatar refresh triggered").into_response()
}

async fn refresh_avatars_task(state: &AppState, player_id: Option<i32>) -> anyhow::Result<()> {
    let mut processor = AvatarProcessor::new()?;
    let mut conn = state.pool.get()?;

    let players = match player_id {
        Some(id) => {
            match database::players::find_by_id(&mut conn, id)? {
                Some(player) => vec![player],
                None => anyhow::bail!("Player {} not found", id),
            }
        }
        None => database::players::list_all(&mut conn)?,
    };

    let total_players = players.len();
    let mut updated_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;

    for (idx, player) in players.iter().enumerate() {
        if (idx + 1) % 10 == 0 {
            log::info!("Progress: {}/{} players processed", idx + 1, total_players);
        }

        if let Some(avatar_url) = &player.avatar_url {
            match processor.refresh_avatar_if_changed(&mut conn, player.id, avatar_url).await {
                Ok(updated) => {
                    if updated {
                        log::info!("Updated avatar for player {}: {}", player.id, player.name);
                        updated_count += 1;
                    } else {
                        skipped_count += 1;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to refresh avatar for player {} ({}): {:?}", player.id, player.name, e);
                    failed_count += 1;
                }
            }
        } else {
            log::debug!("Skipping player {} (no avatar URL)", player.id);
            skipped_count += 1;
        }
    }

    log::info!(
        "Avatar refresh summary: {} updated, {} skipped (unchanged/no URL), {} failed",
        updated_count, skipped_count, failed_count
    );

    Ok(())
}
