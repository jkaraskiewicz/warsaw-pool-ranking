use anyhow::Result;

use warsaw_pool_ranking::cli::Command;
use warsaw_pool_ranking::cli::handlers::{
    handle_serve, handle_tournament_command, handle_ranking_command,
    handle_avatar_command, handle_database_command,
};
use warsaw_pool_ranking::interpret;

fn main() {
    setup_logging();
    parse_and_execute().unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        std::process::exit(1);
    });
}

fn setup_logging() {
    sensible_env_logger::init!();
}

fn parse_and_execute() -> Result<()> {
    let command = interpret();
    execute_command(command)
}

fn execute_command(command: Command) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        match command {
            Command::Serve { port } => handle_serve(port).await,
            Command::Tournaments(cmd) => handle_tournament_command(cmd).await,
            Command::Rankings(cmd) => handle_ranking_command(cmd).await,
            Command::Avatars(cmd) => handle_avatar_command(cmd).await,
            Command::Database(cmd) => handle_database_command(cmd).await,
        }
    })
}
