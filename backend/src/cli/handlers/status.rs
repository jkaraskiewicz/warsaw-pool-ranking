use anyhow::Result;
use std::path::Path;

use crate::cache::Cache;
use crate::config::paths;
use crate::database;
use crate::fetchers::cuescore_models::TournamentResponse;

pub async fn handle_status() -> Result<()> {
    println!("\n=== Warsaw Pool Rankings Status ===\n");

    print_database_status()?;
    print_cache_status()?;
    print_avatar_status()?;

    Ok(())
}

fn print_database_status() -> Result<()> {
    let db_path = paths::get_database_path();

    if Path::new(&db_path).exists() {
        let size_mb = std::fs::metadata(&db_path)?.len() as f64 / 1_048_576.0;
        println!("Database: {:.1} MB", size_mb);

        let pool = database::create_pool(&db_path)?;
        let conn = pool.get()?;

        let player_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM players", [], |r| r.get(0))?;
        let game_count: i64 = conn.query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))?;
        let rating_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM ratings", [], |r| r.get(0))?;

        println!("  Players: {}", player_count);
        println!("  Games:   {}", game_count);
        println!("  Ratings: {}", rating_count);
    } else {
        println!("Database: Not initialized");
    }

    Ok(())
}

fn print_cache_status() -> Result<()> {
    let cache_dir = paths::get_cache_dir();
    let cache = Cache::new(cache_dir.clone())?;

    let tournaments = cache.load_parsed::<Vec<TournamentResponse>>("tournaments")?;

    println!();
    if let Some(t) = tournaments {
        println!("Cache: {} tournaments", t.len());

        // Count raw cache files
        let raw_dir = cache_dir.join("raw");
        if raw_dir.exists() {
            let raw_count = std::fs::read_dir(&raw_dir)?.count();
            println!("  Raw files: {}", raw_count);
        }
    } else {
        println!("Cache: Empty");
    }

    Ok(())
}

fn print_avatar_status() -> Result<()> {
    let db_path = paths::get_database_path();

    if !Path::new(&db_path).exists() {
        return Ok(());
    }

    let pool = database::create_pool(&db_path)?;
    let conn = pool.get()?;

    // Check if avatars table exists
    let table_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='avatars'",
        [],
        |r| r.get(0),
    )?;

    if table_exists == 0 {
        return Ok(());
    }

    let small_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM avatars WHERE size = 'small'", [], |r| {
            r.get(0)
        })?;
    let medium_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM avatars WHERE size = 'medium'",
        [],
        |r| r.get(0),
    )?;
    let large_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM avatars WHERE size = 'large'", [], |r| {
            r.get(0)
        })?;

    let total_bytes: i64 = conn
        .query_row("SELECT COALESCE(SUM(LENGTH(image_data)), 0) FROM avatars", [], |r| {
            r.get(0)
        })?;

    let total_avatars = small_count + medium_count + large_count;

    if total_avatars > 0 {
        println!();
        println!("Avatars: {} total ({:.2} MB)", total_avatars, total_bytes as f64 / 1_048_576.0);
        println!("  Small (60x60):    {}", small_count);
        println!("  Medium (120x120): {}", medium_count);
        println!("  Large (144x144):  {}", large_count);
    }

    Ok(())
}
