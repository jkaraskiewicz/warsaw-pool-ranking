use anyhow::Result;
use crate::cli::DatabaseCommand;
use crate::cli::prompts::confirm_destructive;
use crate::config::paths;
use std::path::Path;

pub async fn handle_database_command(cmd: DatabaseCommand) -> Result<()> {
    match cmd {
        DatabaseCommand::Reset { yes } => handle_reset(yes).await,
        DatabaseCommand::Backup { to } => handle_backup(to).await,
        DatabaseCommand::Restore { from, yes } => handle_restore(from, yes).await,
    }
}

async fn handle_reset(yes: bool) -> Result<()> {
    if !yes && !confirm_destructive("database tables", "drop")? {
        log::info!("Operation cancelled.");
        return Ok(());
    }

    let db_path = paths::get_database_path();

    if Path::new(&db_path).exists() {
        std::fs::remove_file(&db_path)?;
        log::info!("Database reset successfully.");
    } else {
        log::warn!("Database file does not exist.");
    }

    Ok(())
}

async fn handle_backup(to: String) -> Result<()> {
    let db_path = paths::get_database_path();

    if !Path::new(&db_path).exists() {
        anyhow::bail!("Database does not exist. Nothing to backup.");
    }

    std::fs::copy(&db_path, &to)?;

    let size = std::fs::metadata(&to)?.len();
    log::info!("Database backed up to {} ({:.2} MB)", to, size as f64 / 1_048_576.0);

    Ok(())
}

async fn handle_restore(from: String, yes: bool) -> Result<()> {
    if !Path::new(&from).exists() {
        anyhow::bail!("Backup file '{}' does not exist.", from);
    }

    if !yes && !confirm_destructive("current database", "overwrite with backup")? {
        log::info!("Operation cancelled.");
        return Ok(());
    }

    let db_path = paths::get_database_path();
    std::fs::copy(&from, &db_path)?;

    let size = std::fs::metadata(&db_path)?.len();
    log::info!("Database restored from {} ({:.2} MB)", from, size as f64 / 1_048_576.0);

    Ok(())
}
