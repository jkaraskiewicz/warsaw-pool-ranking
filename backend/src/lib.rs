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

use crate::cli::Command;
use crate::config::settings::AppConfig;
use crate::services::ingestion::IngestionService;
use crate::services::processing::ProcessingService;
use crate::services::server::ServerService;

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