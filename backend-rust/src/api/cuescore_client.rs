use crate::models::Tournament;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

const API_BASE_URL: &str = "https://api.cuescore.com";
const RATE_LIMIT_DELAY_MS: u64 = 1000; // 1 request per second

/// CueScore API client
pub struct CueScoreClient {
    client: Client,
    rate_limit_delay: Duration,
}

impl CueScoreClient {
    /// Create a new CueScore API client
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            rate_limit_delay: Duration::from_millis(RATE_LIMIT_DELAY_MS),
        }
    }

    /// Fetch all tournaments for a venue
    pub async fn fetch_venue_tournaments(&self, venue_id: i64) -> Result<Vec<Tournament>> {
        info!("Fetching tournaments for venue {}", venue_id);

        let tournaments = Vec::new();
        let mut page = 1;

        loop {
            // Rate limiting
            sleep(self.rate_limit_delay).await;

            let url = format!(
                "{}/venues/{}/tournaments?page={}",
                API_BASE_URL, venue_id, page
            );

            let response = self
                .client
                .get(&url)
                .send()
                .await
                .context("Failed to fetch tournaments")?;

            if !response.status().is_success() {
                warn!("Failed to fetch page {}: {}", page, response.status());
                break;
            }

            let data: Value = response.json().await?;

            // TODO: Parse tournament data from response
            // This is scaffolding - you'll need to implement actual parsing
            // based on CueScore API response structure

            // Check if we have more pages
            if !Self::has_more_pages(&data) {
                break;
            }

            page += 1;
        }

        info!(
            "Fetched {} tournaments for venue {}",
            tournaments.len(),
            venue_id
        );
        Ok(tournaments)
    }

    /// Fetch tournament details including games
    pub async fn fetch_tournament_details(&self, tournament_id: i64) -> Result<Value> {
        sleep(self.rate_limit_delay).await;

        let url = format!("{}/tournaments/{}", API_BASE_URL, tournament_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch tournament details")?;

        let data = response.json().await?;
        Ok(data)
    }

    /// Check if there are more pages in the response
    fn has_more_pages(_data: &Value) -> bool {
        // TODO: Implement based on actual API response structure
        // Example: _data["pagination"]["has_more"].as_bool().unwrap_or(false)
        false
    }
}

impl Default for CueScoreClient {
    fn default() -> Self {
        Self::new()
    }
}
