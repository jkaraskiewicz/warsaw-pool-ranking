use crate::api::parsers;
use crate::cache::Cache;
use crate::domain::models::{Tournament, TournamentResponse};
use crate::http::RateLimitedClient;
use crate::pagination::{PageIterator, PaginationConfig};
use anyhow::{Context, Result};
use log::{info, warn};
use serde_json::Value;

const API_BASE_URL: &str = "https://api.cuescore.com";
const RATE_LIMIT_MS: u64 = 1000;
const USER_AGENT: &str = "WarsawPoolRankings/2.0";
const TIMEOUT_SECS: u64 = 30;

/// CueScore API client
pub struct CueScoreClient {
    client: RateLimitedClient,
}

impl CueScoreClient {
    /// Create a new CueScore API client
    pub fn new() -> Result<Self> {
        let client = RateLimitedClient::new(USER_AGENT, TIMEOUT_SECS, RATE_LIMIT_MS)?;
        Ok(Self { client })
    }

    /// Fetch all tournaments for a venue
    pub async fn fetch_venue_tournaments(&mut self, venue_id: i64) -> Result<Vec<Tournament>> {
        info!("Fetching tournaments for venue {}", venue_id);

        let config = PaginationConfig::new();
        let mut pages = PageIterator::new(config);
        let tournaments = Vec::new();

        loop {
            if pages.has_reached_max() {
                break;
            }

            let url = Self::build_venue_tournaments_url(venue_id, pages.current_page());
            let response = self.client.get(&url).await?;

            if !self.is_success(&response) {
                warn!("Failed to fetch page {}: {}", pages.current_page(), response.status());
                break;
            }

            let data: Value = response.json().await?;

            // TODO: Parse tournament data from response
            // This is scaffolding - implement actual parsing based on API structure

            if !Self::has_more_pages(&data) {
                break;
            }

            pages.advance();
        }

        info!(
            "Fetched {} tournaments for venue {}",
            tournaments.len(),
            venue_id
        );
        Ok(tournaments)
    }

    /// Fetch tournament details including games
    pub async fn fetch_tournament_details(&mut self, tournament_id: i64) -> Result<TournamentResponse> {
        let url = Self::build_tournament_url(tournament_id);
        info!("Fetching tournament {} from {}", tournament_id, url);

        let response = self.client.get(&url).await?;

        if !response.status().is_success() {
            anyhow::bail!("API returned status: {}", response.status());
        }

        // First get the raw JSON to debug
        let text = response.text().await?;
        info!("Response preview: {}", &text[..text.len().min(500)]);

        // Try to parse it
        let data: TournamentResponse = serde_json::from_str(&text)
            .with_context(|| format!("Failed to parse tournament response. Raw: {}", &text[..text.len().min(200)]))?;

        Ok(data)
    }

    /// Fetch tournament with cache integration
    pub async fn fetch_and_cache_tournament(
        &mut self,
        tournament_id: i64,
        cache: &Cache,
    ) -> Result<TournamentResponse> {
        // Check cache first
        if let Some(cached) = self.load_from_cache(tournament_id, cache)? {
            return Ok(cached);
        }

        // Fetch from API
        let tournament = self.fetch_tournament_details(tournament_id).await?;

        // Save to cache
        self.save_to_cache(tournament_id, &tournament, cache)?;

        Ok(tournament)
    }

    // --- Helper Methods (Short Functions) ---

    fn build_tournament_url(tournament_id: i64) -> String {
        format!("{}/tournament/?id={}", API_BASE_URL, tournament_id)
    }

    fn build_venue_tournaments_url(venue_id: i64, page: usize) -> String {
        let base = format!("{}/venues/{}/tournaments", API_BASE_URL, venue_id);
        crate::pagination::build_paginated_url(&base, page)
    }

    fn is_success(&self, response: &reqwest::Response) -> bool {
        response.status().is_success()
    }

    fn load_from_cache(&self, tournament_id: i64, cache: &Cache) -> Result<Option<TournamentResponse>> {
        let raw = cache.load_raw(&tournament_id.to_string())?;

        match raw {
            Some(json) => {
                let tournament = serde_json::from_value(json)?;
                Ok(Some(tournament))
            }
            None => Ok(None),
        }
    }

    fn save_to_cache(&self, tournament_id: i64, tournament: &TournamentResponse, cache: &Cache) -> Result<()> {
        let raw_json = serde_json::to_value(tournament)?;
        cache.save_raw(&tournament_id.to_string(), &raw_json)?;
        Ok(())
    }

    /// Check if there are more pages in the response
    fn has_more_pages(data: &Value) -> bool {
        parsers::has_more_pages(data)
    }
}
