use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::api::handlers::{
    admin::{admin_login, admin_refresh},
    avatars::serve_avatar,
    players::{get_player_detail, get_head_to_head_comparison, get_player_rivalries, get_players},
    AppState,
};

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/players", get(get_players))
        .route("/api/player/:id", get(get_player_detail))
        .route("/api/player/:id/rivalries", get(get_player_rivalries))
        .route("/api/compare/:player1_id/:player2_id", get(get_head_to_head_comparison))
        .route("/api/avatars/:player_id/:size", get(serve_avatar))
        .route("/api/admin/login", post(admin_login))
        .route("/api/admin/refresh", post(admin_refresh))
        .with_state(state)
}