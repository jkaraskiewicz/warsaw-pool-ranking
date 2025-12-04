use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::api::handlers::{get_players, get_player_detail, admin_refresh, AppState};

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/players", get(get_players))
        .route("/api/player/:id", get(get_player_detail))
        .route("/api/admin/refresh", post(admin_refresh))
        .with_state(state)
}
