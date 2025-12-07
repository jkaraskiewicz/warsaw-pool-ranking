use axum::{
    extract::State,
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use std::sync::Arc;
use log;

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
