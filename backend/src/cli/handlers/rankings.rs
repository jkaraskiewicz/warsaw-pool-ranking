use anyhow::Result;
use crate::cli::{RankingCommand, ExportFormat};
use crate::cli::dependencies::{OperationOrchestrator, Resource};
use crate::config::paths;
use crate::database;
use crate::database::models::DbRating;

pub async fn handle_ranking_command(cmd: RankingCommand) -> Result<()> {
    match cmd {
        RankingCommand::Refresh { force } => handle_refresh(force).await,
        RankingCommand::Export { output, format } => handle_export(output, format).await,
    }
}

async fn handle_refresh(force: bool) -> Result<()> {
    let orchestrator = OperationOrchestrator::new();
    orchestrator.refresh_with_deps(Resource::Rankings, force).await
}

async fn handle_export(output: String, format: ExportFormat) -> Result<()> {
    let db_path = paths::get_database_path();
    let pool = database::create_pool(&db_path)?;
    let conn = pool.get()?;

    // Fetch all ratings from database
    let ratings: Vec<DbRating> = conn
        .prepare("SELECT id, player_id, rating_type, rating, games_played, confidence_level, calculated_at, created_at FROM ratings ORDER BY rating_type, rating DESC")?
        .query_map([], |row| {
            Ok(DbRating {
                id: row.get(0)?,
                player_id: row.get(1)?,
                rating_type: row.get(2)?,
                rating: row.get(3)?,
                games_played: row.get(4)?,
                confidence_level: row.get(5)?,
                calculated_at: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    match format {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&ratings)?;
            std::fs::write(&output, json)?;
            log::info!("Exported {} ratings to {} (JSON)", ratings.len(), output);
        },
        ExportFormat::Csv => {
            let mut csv_content = String::from("id,player_id,rating_type,rating,games_played,confidence_level,calculated_at\n");
            for rating in &ratings {
                csv_content.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    rating.id,
                    rating.player_id,
                    rating.rating_type,
                    rating.rating,
                    rating.games_played,
                    rating.confidence_level,
                    rating.calculated_at
                ));
            }
            std::fs::write(&output, csv_content)?;
            log::info!("Exported {} ratings to {} (CSV)", ratings.len(), output);
        },
    }

    Ok(())
}
