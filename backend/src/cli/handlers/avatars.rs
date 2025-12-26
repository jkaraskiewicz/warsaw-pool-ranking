use anyhow::Result;
use crate::cli::AvatarCommand;
use crate::cli::prompts::confirm_destructive;
use crate::cli::dependencies::{OperationOrchestrator, Resource};
use crate::config::paths;
use crate::config::settings::AppConfig;
use crate::database;
use crate::database::repositories::player_repository::PlayerRepository;
use crate::services::avatar_processor::AvatarProcessor;

pub async fn handle_avatar_command(cmd: AvatarCommand) -> Result<()> {
    match cmd {
        AvatarCommand::Refresh { player_id, force } => handle_refresh(player_id, force).await,
        AvatarCommand::Prune { yes } => handle_prune(yes).await,
        AvatarCommand::Stats => handle_stats().await,
    }
}

async fn handle_refresh(player_id: Option<i64>, force: bool) -> Result<()> {
    if player_id.is_some() {
        // Refresh single player - bypass dependency resolution
        execute_avatar_refresh(player_id).await
    } else {
        // Refresh all players - use dependency resolution
        let orchestrator = OperationOrchestrator::new();
        orchestrator.refresh_with_deps(Resource::Avatars, force).await
    }
}

async fn execute_avatar_refresh(player_id: Option<i64>) -> Result<()> {
    let config = AppConfig::new();
    let db_path = paths::get_database_path();
    let pool = database::create_pool(&db_path)?;
    let mut conn = pool.get()?;
    let mut processor = AvatarProcessor::new(config.avatar)?;

    let players = if let Some(id) = player_id {
        // Refresh single player by cuescore_id
        let player = PlayerRepository::find_by_cuescore_id(&mut conn, id)?
            .ok_or_else(|| anyhow::anyhow!("Player with cuescore_id {} not found", id))?;
        vec![player]
    } else {
        // Refresh all
        PlayerRepository::list_all(&mut conn)?
    };

    log::info!("Starting avatar refresh for {} players...", players.len());

    let mut updated_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;

    for player in players {
        if let Some(ref url) = player.avatar_url {
            match processor.refresh_avatar_if_changed(&mut conn, player.cuescore_id, url).await {
                Ok(updated) => {
                    if updated {
                        log::info!("Updated avatar for player {} (cuescore_id={})", player.name, player.cuescore_id);
                        updated_count += 1;
                    } else {
                        skipped_count += 1;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to refresh avatar for {}: {}", player.name, e);
                    failed_count += 1;
                }
            }
        } else {
            skipped_count += 1;
        }
    }

    log::info!("Avatar refresh complete: {} updated, {} skipped, {} failed", updated_count, skipped_count, failed_count);
    Ok(())
}

async fn handle_prune(yes: bool) -> Result<()> {
    if !yes && !confirm_destructive("avatar data", "delete")? {
        log::info!("Operation cancelled.");
        return Ok(());
    }

    let db_path = paths::get_database_path();
    let pool = database::create_pool(&db_path)?;
    let conn = pool.get()?;

    let count = conn.execute("DELETE FROM avatars", [])?;

    log::info!("Deleted {} avatar records from database.", count);
    Ok(())
}

async fn handle_stats() -> Result<()> {
    let db_path = paths::get_database_path();
    let pool = database::create_pool(&db_path)?;
    let conn = pool.get()?;

    // Count avatars by size
    let small_count: i64 = conn.query_row("SELECT COUNT(*) FROM avatars WHERE size = 'small'", [], |row| row.get(0))?;
    let medium_count: i64 = conn.query_row("SELECT COUNT(*) FROM avatars WHERE size = 'medium'", [], |row| row.get(0))?;
    let large_count: i64 = conn.query_row("SELECT COUNT(*) FROM avatars WHERE size = 'large'", [], |row| row.get(0))?;

    // Calculate total storage
    let total_bytes: i64 = conn.query_row("SELECT SUM(LENGTH(image_data)) FROM avatars", [], |row| row.get(0)).unwrap_or(0);

    println!("\n📊 Avatar Storage Statistics\n");
    println!("Total avatar records: {}", small_count + medium_count + large_count);
    println!("  Small (60x60):      {}", small_count);
    println!("  Medium (120x120):   {}", medium_count);
    println!("  Large (144x144):    {}", large_count);
    println!("\nTotal storage: {:.2} MB", total_bytes as f64 / 1_048_576.0);

    Ok(())
}
