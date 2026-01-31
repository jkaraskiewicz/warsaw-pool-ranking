use anyhow::Result;

use warsaw_pool_ranking::cli::handlers::{
    handle_backup, handle_export, handle_reset, handle_restore, handle_serve, handle_status,
    handle_sync,
};
use warsaw_pool_ranking::cli::Command;
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
            Command::Sync { step, force } => handle_sync(step, force).await,
            Command::Status => handle_status().await,
            Command::Export {
                output,
                r#type,
                format,
            } => handle_export(output, r#type, format).await,
            Command::Reset { keep_cache, yes } => handle_reset(keep_cache, yes).await,
            Command::Backup { to } => handle_backup(to).await,
            Command::Restore { from, yes } => handle_restore(from, yes).await,
        }
    })
}
