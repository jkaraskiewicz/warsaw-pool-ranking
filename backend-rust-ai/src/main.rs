mod api;
mod cache;
mod db;
mod models;
mod rating;
mod scraper;

use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Warsaw Pool Ranking - Rust Backend");
    info!("====================================");

    // TODO: Main orchestration logic
    // 1. Scrape venue pages to discover tournament IDs (scraper module)
    // 2. Fetch tournament details from CueScore API (api module)
    // 3. Cache the data (cache module)
    // 4. Populate database (db module)
    // 5. Calculate ratings (rating module)

    Ok(())
}
