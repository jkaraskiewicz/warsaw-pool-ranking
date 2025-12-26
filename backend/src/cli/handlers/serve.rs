use anyhow::Result;
use crate::config::settings::AppConfig;
use crate::services::server::ServerService;

pub async fn handle_serve(port: u16) -> Result<()> {
    let config = AppConfig::new();
    let service = ServerService::new(port, config);
    service.run().await
}
