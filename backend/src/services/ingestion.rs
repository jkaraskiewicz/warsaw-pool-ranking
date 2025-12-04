use std::collections::HashSet;
use anyhow::Result;
use log::info;

use crate::api::CueScoreClient;
use crate::cache::Cache;
use crate::config::get_venues;
use crate::domain::{FetchProgress, TournamentCollection};
use crate::fetchers::VenueScraper;

pub struct IngestionService {
    cache: Cache,
    scraper: VenueScraper,
    api_client: CueScoreClient,
}

impl IngestionService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            cache: Cache::new("cache")?,
            scraper: VenueScraper::new()?,
            api_client: CueScoreClient::new()?,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("=== Starting Data Ingestion ===\n");

        // Step 1: Discover tournaments
        let tournament_ids = self.discover_tournaments().await?;
        info!("  → Found {} unique tournaments\n", tournament_ids.len());

        // Step 2: Fetch tournament data
        let collection = self.fetch_tournaments(tournament_ids).await?;
        info!("  → Fetched {} tournaments with data\n", collection.len());

        // Step 3: Save to parsed cache
        self.save_parsed_cache(collection)?;
        info!("  → Saved to parsed cache\n");

        info!("=== Ingestion Complete ===");
        Ok(())
    }

    async fn discover_tournaments(&mut self) -> Result<HashSet<i64>> {
        info!("Step 1: Discovering tournaments from venues...");

        let venues = get_venues();
        let mut all_ids = HashSet::new();

        for venue in venues {
            let ids = self.scraper.scrape_venue_tournaments(venue.id, venue.name, None).await?;
            all_ids.extend(ids);
        }

        Ok(all_ids)
    }

    async fn fetch_tournaments(&mut self, tournament_ids: HashSet<i64>) -> Result<TournamentCollection> {
        info!("Step 2: Fetching tournament details...");

        let total = tournament_ids.len();
        let mut progress = FetchProgress::new(total);
        let mut collection = TournamentCollection::new();

        for tournament_id in tournament_ids {
            let was_cached = self.is_cached(tournament_id)?;

            if let Some(tournament) = self.fetch_single_tournament(tournament_id).await? {
                collection.add(tournament);
            }

            self.update_progress(&mut progress, was_cached);
        }

        Ok(collection)
    }

    fn is_cached(&self, tournament_id: i64) -> Result<bool> {
        Ok(self.cache.load_raw(&tournament_id.to_string())?.is_some())
    }

    async fn fetch_single_tournament(&mut self, tournament_id: i64) -> Result<Option<crate::domain::TournamentResponse>> {
        self.api_client.fetch_and_cache_tournament(tournament_id, &self.cache).await
    }

    fn update_progress(&self, progress: &mut FetchProgress, was_cached: bool) {
        if was_cached {
            progress.increment_cached();
        } else {
            progress.increment_fetched();
        }
    }

    fn save_parsed_cache(&self, collection: TournamentCollection) -> Result<()> {
        info!("Step 3: Saving parsed tournament cache...");
        let tournaments = collection.into_vec();
        self.cache.save_parsed("tournaments", &tournaments)?;
        Ok(())
    }
}
