use anyhow::Result;
use crate::cli::{TournamentCommand, OutputFormat};
use crate::cli::prompts::confirm_destructive;
use crate::cli::dependencies::{OperationOrchestrator, Resource};
use crate::cache::Cache;
use crate::config::paths;
use crate::fetchers::cuescore_models::TournamentResponse;

pub async fn handle_tournament_command(cmd: TournamentCommand) -> Result<()> {
    match cmd {
        TournamentCommand::Refresh { force } => handle_refresh(force).await,
        TournamentCommand::Prune { yes } => handle_prune(yes).await,
        TournamentCommand::List { format, limit } => handle_list(format, limit).await,
    }
}

async fn handle_refresh(force: bool) -> Result<()> {
    let orchestrator = OperationOrchestrator::new();
    orchestrator.refresh_with_deps(Resource::Tournaments, force).await
}

async fn handle_prune(yes: bool) -> Result<()> {
    if !yes && !confirm_destructive("tournament data", "delete")? {
        log::info!("Operation cancelled.");
        return Ok(());
    }

    // Delete cache/raw/* and cache/parsed/tournaments.json
    let raw_dir = paths::get_cache_dir().join("raw");
    if raw_dir.exists() {
        std::fs::remove_dir_all(&raw_dir)?;
        std::fs::create_dir_all(&raw_dir)?;
    }

    let parsed_path = paths::get_cache_dir().join("parsed").join("tournaments.json");
    if parsed_path.exists() {
        std::fs::remove_file(&parsed_path)?;
    }

    log::info!("Tournament data pruned successfully.");
    Ok(())
}

async fn handle_list(format: OutputFormat, limit: Option<usize>) -> Result<()> {
    let cache = Cache::new(paths::get_cache_dir())?;
    let tournaments = cache.load_parsed::<Vec<TournamentResponse>>("tournaments")?
        .ok_or_else(|| anyhow::anyhow!("No tournaments in cache. Run 'tournaments refresh' first."))?;

    let display_count = limit.unwrap_or(tournaments.len()).min(tournaments.len());

    match format {
        OutputFormat::Table => {
            println!("\nCached Tournaments (showing {}/{}):\n", display_count, tournaments.len());
            println!("{:<10} {:<50} {:<20}", "ID", "Name", "Start Date");
            println!("{}", "-".repeat(80));
            for t in tournaments.iter().take(display_count) {
                println!("{:<10} {:<50} {:<20}", t.id, truncate(&t.name, 50), &t.starttime[..10]);
            }
        },
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&tournaments[..display_count])?);
        },
        OutputFormat::Csv => {
            println!("id,name,start_date");
            for t in tournaments.iter().take(display_count) {
                println!("{},{},{}", t.id, escape_csv(&t.name), &t.starttime[..10]);
            }
        },
    }

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
