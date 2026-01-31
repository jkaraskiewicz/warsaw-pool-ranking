use anyhow::Result;
use std::path::Path;

use crate::cli::prompts::confirm_destructive;
use crate::config::paths;

pub async fn handle_reset(keep_cache: bool, yes: bool) -> Result<()> {
    if !yes && !confirm_destructive("system", "reset")? {
        log::info!("Operation cancelled.");
        return Ok(());
    }

    // Delete database
    let db_path = paths::get_database_path();
    if Path::new(&db_path).exists() {
        std::fs::remove_file(&db_path)?;
        log::info!("Deleted database");
    }

    // Delete cache unless --keep-cache
    if !keep_cache {
        let cache_dir = paths::get_cache_dir();

        let raw_cache = cache_dir.join("raw");
        if raw_cache.exists() {
            std::fs::remove_dir_all(&raw_cache)?;
            log::info!("Deleted raw cache");
        }

        let parsed_cache = cache_dir.join("parsed").join("tournaments.json");
        if parsed_cache.exists() {
            std::fs::remove_file(&parsed_cache)?;
            log::info!("Deleted parsed cache");
        }
    }

    log::info!("Reset complete");
    Ok(())
}
