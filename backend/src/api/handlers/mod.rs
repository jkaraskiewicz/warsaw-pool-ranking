use axum::extract::FromRef;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::Deserialize;

use crate::config::settings::AppConfig;

pub mod players;
pub mod admin;

#[derive(Clone)] // AppState usually needs Clone if used in FromRef, but here we use Arc<AppState> so it's fine.
pub struct AppState {
    pub pool: Pool<SqliteConnectionManager>,
    pub config: AppConfig,
}

#[derive(Deserialize)]
pub struct PlayerParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub sort_by: Option<String>,
    pub order: Option<String>,
    pub filter: Option<String>,
    pub rating_type: Option<String>,
}
