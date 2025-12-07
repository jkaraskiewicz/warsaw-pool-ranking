use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::api::handlers::{players::{get_players, get_player_detail, get_head_to_head_comparison}, admin::admin_refresh, AppState};

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/players", get(get_players))
        .route("/api/player/:id", get(get_player_detail))
        .route("/api/compare/:player1_id/:player2_id", get(get_head_to_head_comparison))
        .route("/api/admin/refresh", post(admin_refresh))
        .with_state(state)
}
