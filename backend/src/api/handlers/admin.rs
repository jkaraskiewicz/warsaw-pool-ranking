use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use log;
use std::sync::Arc;

use crate::api::models::{LoginRequest, LoginResponse};
use crate::database::repositories::player_repository::PlayerRepository;
use crate::services::avatar_processor::AvatarProcessor;
use crate::services::ingestion::IngestionService;
use crate::services::processing::ProcessingService;
use super::AppState;

// Helper function to validate Authorization header
fn validate_auth_header(headers: &HeaderMap, expected_password: &str) -> bool {
    headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|token| token == expected_password)
        .unwrap_or(false)
}

// Admin login endpoint
pub async fn admin_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let is_valid = payload.password == state.config.admin.password;

    if is_valid {
        log::info!("Admin login successful");
        Json(LoginResponse {
            success: true,
            message: None,
        }).into_response()
    } else {
        log::warn!("Admin login failed - invalid password");
        Json(LoginResponse {
            success: false,
            message: Some("Invalid password".to_string()),
        }).into_response()
    }
}

pub async fn admin_refresh(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !validate_auth_header(&headers, &state.config.admin.password) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    tokio::spawn(async move {
        log::info!("=== Admin Full Sync Started ===");

        // Step 1: Fetch tournaments
        log::info!("Step 1/3: Fetching tournament data...");
        let ingest_result = async {
            let mut ingest_service = IngestionService::new()?;
            ingest_service.run().await
        }.await;
        if let Err(e) = ingest_result {
            log::error!("Sync failed at ingestion: {:?}", e);
            return;
        }

        // Step 2: Calculate ratings
        log::info!("Step 2/3: Calculating player ratings...");
        let process_result = async {
            let process_service = ProcessingService::new(state.config.clone())?;
            process_service.run()
        }.await;
        if let Err(e) = process_result {
            log::error!("Sync failed at processing: {:?}", e);
            return;
        }

        // Step 3: Update avatars
        log::info!("Step 3/3: Updating player avatars...");
        let avatar_result = refresh_avatars_task(&state).await;
        if let Err(e) = avatar_result {
            log::error!("Sync failed at avatar refresh: {:?}", e);
            return;
        }

        log::info!("=== Admin Full Sync Completed ===");
    });

    (StatusCode::ACCEPTED, "Full sync triggered").into_response()
}

async fn refresh_avatars_task(state: &AppState) -> anyhow::Result<()> {
    let mut processor = AvatarProcessor::new(state.config.avatar.clone())?;
    let mut conn = state.pool.get()?;
    let players = PlayerRepository::list_all(&mut conn)?;

    let total_players = players.len();
    let mut updated_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;

    for (idx, player) in players.iter().enumerate() {
        if (idx + 1) % 10 == 0 {
            log::info!("Progress: {}/{} players processed", idx + 1, total_players);
        }

        if let Some(avatar_url) = &player.avatar_url {
            match processor.refresh_avatar_if_changed(&mut conn, player.cuescore_id, avatar_url).await {
                Ok(updated) => {
                    if updated {
                        log::info!("Updated avatar for player {} (cuescore_id={})", player.name, player.cuescore_id);
                        updated_count += 1;
                    } else {
                        skipped_count += 1;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to refresh avatar for player {} (cuescore_id={}): {:?}", player.name, player.cuescore_id, e);
                    failed_count += 1;
                }
            }
        } else {
            log::debug!("Skipping player {} (no avatar URL)", player.name);
            skipped_count += 1;
        }
    }

    log::info!(
        "Avatar refresh summary: {} updated, {} skipped (unchanged/no URL), {} failed",
        updated_count, skipped_count, failed_count
    );

    Ok(())
}
