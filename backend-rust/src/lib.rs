pub mod api;
pub mod cache;
pub mod cli;
pub mod database;
pub mod domain;
pub mod fetchers;

use std::collections::HashSet;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

use crate::api::CueScoreClient;
use crate::cache::Cache;
use crate::cli::Command;
use crate::fetchers::VenueScraper;

pub fn interpret() -> Command {
    let cli = Cli::parse();
    cli.command
}

pub fn handle_serve(port: u16) -> Result<()> {
    todo!()
}

pub fn handle_ingest() -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(ingest_data())
}

async fn ingest_data() -> Result<()> {
    use log::info;

    info!("=== Starting Data Ingestion ===\n");

    let cache = Cache::new("cache")?;
    let scraper = VenueScraper::new()?;
    let api_client = CueScoreClient::new();

    // Step 1: Discover tournaments
    let tournament_ids = discover_tournaments(&scraper).await?;
    info!("  → Found {} unique tournaments\n", tournament_ids.len());

    // Step 2: Fetch tournament data
    let (fetched, cached) = fetch_tournaments(&api_client, &cache, tournament_ids).await?;
    info!("  → {} fetched, {} from cache\n", fetched, cached);

    info!("=== Ingestion Complete ===");
    Ok(())
}

async fn discover_tournaments(scraper: &VenueScraper) -> Result<HashSet<i64>> {
    use log::info;

    info!("Step 1: Discovering tournaments from venues...");

    // TODO: Load venue list from config file
    let venues = vec![
        (1234, "venue-name-1"),
        (5678, "venue-name-2"),
    ];

    let mut all_ids = HashSet::new();

    for (venue_id, venue_name) in venues {
        let ids = scraper.scrape_venue_tournaments(venue_id, venue_name, None).await?;
        all_ids.extend(ids);
    }

    Ok(all_ids)
}

async fn fetch_tournaments(
    client: &CueScoreClient,
    cache: &Cache,
    tournament_ids: HashSet<i64>,
) -> Result<(usize, usize)> {
    use log::info;

    info!("Step 2: Fetching tournament details...");

    let mut fetched = 0;
    let mut from_cache = 0;

    for tournament_id in tournament_ids {
        let was_cached = cache.load_raw(&tournament_id.to_string())?.is_some();

        client.fetch_and_cache_tournament(tournament_id, cache).await?;

        if was_cached {
            from_cache += 1;
        } else {
            fetched += 1;
        }
    }

    Ok((fetched, from_cache))
}

pub fn handle_process() -> Result<()> {
    todo!()
}
