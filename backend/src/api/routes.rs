use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::api::handlers::{
    admin::{admin_refresh, admin_refresh_avatars},
    avatars::serve_avatar,
    players::{get_players, get_player_detail, get_head_to_head_comparison},
    AppState,
};

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/players", get(get_players))
        .route("/api/player/:id", get(get_player_detail))
        .route("/api/compare/:player1_id/:player2_id", get(get_head_to_head_comparison))
        .route("/api/avatars/:player_id/:size", get(serve_avatar))
        .route("/api/admin/refresh", post(admin_refresh))
        .route("/api/admin/refresh-avatars", post(admin_refresh_avatars))
        .with_state(state)
}
