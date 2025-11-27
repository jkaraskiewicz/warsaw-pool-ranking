pub mod cli;
pub mod cache;
pub mod domain;
pub mod database;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

use crate::cli::Command;

pub fn interpret() -> Command {
    let cli = Cli::parse();
    cli.command
}

pub fn handle_serve(port: u16) -> Result<()> {
    todo!()
}

pub fn handle_ingest() -> Result<()> {
    todo!()
}

pub fn handle_process() -> Result<()> {
    todo!()
}
