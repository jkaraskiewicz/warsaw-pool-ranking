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

use clap::Parser;
use cli::Cli;
use crate::cli::Command;

/// Parse CLI arguments and return the command
pub fn interpret() -> Command {
    let cli = Cli::parse();
    cli.command
}
