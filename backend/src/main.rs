use anyhow::Result;

use warsaw_pool_ranking::cli::Command;
use warsaw_pool_ranking::{handle_ingest, handle_process, handle_serve, interpret};

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
    execute_command(&command)
}

fn execute_command(command: &Command) -> Result<()> {
    match command {
        Command::Serve { port } => handle_serve(*port),
        Command::Ingest => handle_ingest(),
        Command::Process => handle_process(),
    }
}
