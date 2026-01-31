use anyhow::Result;

use crate::cache::Cache;
use crate::cli::{ExportFormat, ExportType};
use crate::config::paths;
use crate::database;
use crate::database::models::DbRating;
use crate::fetchers::cuescore_models::TournamentResponse;

pub async fn handle_export(output: String, export_type: ExportType, format: ExportFormat) -> Result<()> {
    match export_type {
        ExportType::Ratings => export_ratings(&output, &format),
        ExportType::Tournaments => export_tournaments(&output, &format),
    }
}

fn export_ratings(output: &str, format: &ExportFormat) -> Result<()> {
    let db_path = paths::get_database_path();
    let pool = database::create_pool(&db_path)?;
    let conn = pool.get()?;

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
            std::fs::write(output, json)?;
            log::info!("Exported {} ratings to {} (JSON)", ratings.len(), output);
        }
        ExportFormat::Csv => {
            let mut csv = String::from(
                "id,player_id,rating_type,rating,games_played,confidence_level,calculated_at\n",
            );
            for r in &ratings {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    r.id, r.player_id, r.rating_type, r.rating, r.games_played, r.confidence_level, r.calculated_at
                ));
            }
            std::fs::write(output, csv)?;
            log::info!("Exported {} ratings to {} (CSV)", ratings.len(), output);
        }
    }

    Ok(())
}

fn export_tournaments(output: &str, format: &ExportFormat) -> Result<()> {
    let cache = Cache::new(paths::get_cache_dir())?;
    let tournaments = cache
        .load_parsed::<Vec<TournamentResponse>>("tournaments")?
        .ok_or_else(|| anyhow::anyhow!("No tournaments in cache. Run 'sync --step=fetch' first."))?;

    match format {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&tournaments)?;
            std::fs::write(output, json)?;
            log::info!(
                "Exported {} tournaments to {} (JSON)",
                tournaments.len(),
                output
            );
        }
        ExportFormat::Csv => {
            let mut csv = String::from("id,name,start_date\n");
            for t in &tournaments {
                csv.push_str(&format!(
                    "{},{},{}\n",
                    t.id,
                    escape_csv(&t.name),
                    &t.starttime[..10]
                ));
            }
            std::fs::write(output, csv)?;
            log::info!(
                "Exported {} tournaments to {} (CSV)",
                tournaments.len(),
                output
            );
        }
    }

    Ok(())
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
