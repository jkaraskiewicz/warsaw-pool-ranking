use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::database;
use super::AppState;

pub async fn serve_avatar(
    State(state): State<Arc<AppState>>,
    Path((player_id, size)): Path<(i32, String)>,
) -> impl IntoResponse {
    let mut conn = match state.pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to get DB connection: {:?}", e);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let avatar_size = match size.as_str() {
        "small" => "small",
        "medium" => "medium",
        "large" => "large",
        _ => {
            log::warn!("Invalid avatar size requested: {}", size);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    match database::avatars::get_avatar(&mut conn, player_id, avatar_size) {
        Ok(Some(avatar)) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "image/webp")
                .header(header::CACHE_CONTROL, "public, max-age=86400")
                .body(Body::from(avatar.image_data))
                .unwrap_or_else(|e| {
                    log::error!("Failed to build response: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                })
        }
        Ok(None) => {
            log::debug!("Avatar not found for player {} size {}", player_id, avatar_size);
            StatusCode::NOT_FOUND.into_response()
        }
        Err(e) => {
            log::error!("Database error fetching avatar: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
