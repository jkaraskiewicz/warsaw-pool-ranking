use anyhow::Result;
use axum::{
    routing::get,
    Router,
};
use log::info;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

pub struct ServerService {
    port: u16,
}

impl ServerService {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn run(&self) -> Result<()> {
        let app = Router::new()
            .route("/health", get(health_check))
            .layer(CorsLayer::permissive()); // Relaxed CORS for development

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!("Server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health_check() -> &'static str {
    "OK"
}
