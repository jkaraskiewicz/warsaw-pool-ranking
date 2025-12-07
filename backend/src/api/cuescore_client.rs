use crate::api::parsers;
use crate::cache::Cache;
use crate::domain::models::{Tournament, TournamentResponse};
use crate::http::RateLimitedClient;
use crate::pagination::{PageIterator, PaginationConfig};
use anyhow::{Context, Result};
use log::{info, warn};
use serde_json::Value;

const API_BASE_URL: &str = "https://api.cuescore.com";
const RATE_LIMIT_MS: u64 = 100;
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

    /// Fetch tournament raw text
    pub async fn fetch_tournament_raw(&mut self, tournament_id: i64) -> Result<String> {
        let url = Self::build_tournament_url(tournament_id);
        info!("Fetching tournament {} from {}", tournament_id, url);

        let response = self.client.get(&url).await?;

        if !response.status().is_success() {
            anyhow::bail!("API returned status: {}", response.status());
        }

        let text = response.text().await?;
        Ok(text)
    }

    /// Fetch tournament with cache integration
    /// Saves FULL raw JSON to cache, then parses it.
    pub async fn fetch_and_cache_tournament(
        &mut self,
        tournament_id: i64,
        cache: &Cache,
    ) -> Result<Option<TournamentResponse>> {
        // 1. Try load from cache
        let cached_value = cache.load_raw(&tournament_id.to_string())?;

        let json_value = if let Some(val) = cached_value {
            val
        } else {
            // 2. Fetch raw text
            let text = match self.fetch_tournament_raw(tournament_id).await {
                Ok(t) => t,
                Err(e) => {
                    log::error!("Failed to fetch tournament {}: {:?}", tournament_id, e);
                    return Ok(None);
                }
            };

            // 3. Parse to Value to ensure valid JSON and save FULL structure
            let value: Value = serde_json::from_str(&text)
                .with_context(|| format!("Failed to parse JSON for tournament {}", tournament_id))?;

            // 4. Save Value to cache
            if let Err(e) = cache.save_raw(&tournament_id.to_string(), &value) {
                warn!("Failed to save tournament {} to cache: {:?}", tournament_id, e);
            }

            value
        };

        // 5. Parse into domain struct (TournamentResponse)
        // Even if the struct changes later, the cached Value has all fields.
        let tournament: TournamentResponse = serde_json::from_value(json_value)
            .with_context(|| format!("Failed to map JSON to TournamentResponse for {}", tournament_id))?;

        Ok(Some(tournament))
    }

    // --- Helper Methods ---

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

    fn has_more_pages(data: &Value) -> bool {
        parsers::has_more_pages(data)
    }
}