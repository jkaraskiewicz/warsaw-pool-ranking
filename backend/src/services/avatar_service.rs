use anyhow::Result;

use crate::config::paths;
use crate::config::settings::AppConfig;
use crate::database;
use crate::database::repositories::player_repository::PlayerRepository;
use crate::services::avatar_processor::AvatarProcessor;

/// Refresh avatars for all players with avatar URLs
pub async fn refresh_all_avatars() -> Result<()> {
    let config = AppConfig::new();
    let db_path = paths::get_database_path();
    let pool = database::create_pool(&db_path)?;
    let mut conn = pool.get()?;
    let mut processor = AvatarProcessor::new(config.avatar)?;

    let players = PlayerRepository::list_all(&mut conn)?;

    log::info!("Starting avatar refresh for {} players...", players.len());

    let mut updated_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;

    for player in players {
        if let Some(ref url) = player.avatar_url {
            match processor
                .refresh_avatar_if_changed(&mut conn, player.cuescore_id, url)
                .await
            {
                Ok(updated) => {
                    if updated {
                        log::info!(
                            "Updated avatar for player {} (cuescore_id={})",
                            player.name,
                            player.cuescore_id
                        );
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

    log::info!(
        "Avatar refresh complete: {} updated, {} skipped, {} failed",
        updated_count,
        skipped_count,
        failed_count
    );
    Ok(())
}
