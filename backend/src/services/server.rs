use anyhow::Result;
use log::info;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::api::routes::create_router;
use crate::api::handlers::AppState;
use crate::config::settings::AppConfig;
use crate::database;

pub struct ServerService {
    port: u16,
    config: AppConfig,
}

impl ServerService {
    pub fn new(port: u16, config: AppConfig) -> Self {
        Self { port, config }
    }

    pub async fn run(&self) -> Result<()> {
        let db_path = std::env::var("DATABASE_PATH")
            .unwrap_or_else(|_| "warsaw_pool_ranking.db".to_string());

        let pool = database::create_pool(&db_path)?;

        let state = Arc::new(AppState {
            pool,
            config: self.config.clone(), // AppConfig should be Clone or cheap
        });

        // Config AppConfig needs Clone? Structs derive Clone usually.
        // I'll need to check if AppConfig derives Clone. If not, add it.

        let app = create_router(state)
            .layer(CorsLayer::permissive());

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!("Server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
