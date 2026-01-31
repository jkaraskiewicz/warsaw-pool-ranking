use anyhow::Result;

use crate::cli::dependencies::{OperationOrchestrator, Resource};
use crate::cli::SyncStep;

pub async fn handle_sync(step: Option<SyncStep>, force: bool) -> Result<()> {
    let orchestrator = OperationOrchestrator::new();

    match step {
        None => {
            // Full sync: always run all 3 steps in order
            log::info!("Starting full sync...");

            log::info!("Step 1/3: Fetching tournament data from CueScore...");
            orchestrator.execute(Resource::Tournaments).await?;

            log::info!("Step 2/3: Calculating player ratings...");
            orchestrator.execute(Resource::Rankings).await?;

            log::info!("Step 3/3: Updating player avatars...");
            orchestrator.execute(Resource::Avatars).await?;

            log::info!("Full sync complete!");
        }
        Some(SyncStep::Fetch) => {
            log::info!("Fetching tournament data from CueScore...");
            orchestrator
                .refresh_with_deps(Resource::Tournaments, force)
                .await?;
        }
        Some(SyncStep::Calculate) => {
            log::info!("Calculating player ratings...");
            orchestrator
                .refresh_with_deps(Resource::Rankings, force)
                .await?;
        }
        Some(SyncStep::Avatars) => {
            log::info!("Downloading player avatars...");
            orchestrator
                .refresh_with_deps(Resource::Avatars, force)
                .await?;
        }
    }

    Ok(())
}
