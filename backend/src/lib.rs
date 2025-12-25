pub mod api;
pub mod cache;
pub mod cli;
pub mod config;
pub mod database;
pub mod domain;
pub mod errors;
pub mod fetchers;
pub mod http;
pub mod pagination;
pub mod rate_limiter;
pub mod rating;
pub mod services;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use log;

use crate::cli::Command;
use crate::config::settings::AppConfig;
use crate::config::paths;
use crate::database::repositories::player_repository::PlayerRepository;
use crate::services::ingestion::IngestionService;
use crate::services::processing::ProcessingService;
use crate::services::server::ServerService;
use crate::services::avatar_processor::AvatarProcessor;

pub fn interpret() -> Command {
    let cli = Cli::parse();
    cli.command
}

pub fn handle_serve(port: u16) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let config = AppConfig::new();
        let service = ServerService::new(port, config);
        service.run().await
    })
}

pub fn handle_ingest() -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let mut service = IngestionService::new()?;
        service.run().await
    })
}

pub fn handle_process() -> Result<()> {
    let config = AppConfig::new();
    let service = ProcessingService::new(config)?;
    service.run()
}

pub fn handle_refresh_avatars() -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let _config = AppConfig::new(); // Config might be needed for other things implicitly? No, but let's keep it or remove it.
        let db_path = paths::get_database_path();
        let pool = database::create_pool(&db_path)?;
        let mut processor = AvatarProcessor::new()?;
        let mut conn = pool.get()?;

        let players = PlayerRepository::list_all(&mut conn)?;
        log::info!("Starting avatar refresh for {} players...", players.len());

        for player in players {
            if let Some(url) = player.avatar_url {
                if let Err(e) = processor.refresh_avatar_if_changed(&mut conn, player.id, &url).await {
                    log::warn!("Failed to refresh avatar for {}: {}", player.name, e);
                }
            }
        }
        log::info!("Avatar refresh complete.");
        Ok(())
    })
}
