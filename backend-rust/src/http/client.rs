use crate::rate_limiter::RateLimiter;
use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

/// HTTP client with built-in rate limiting
pub struct RateLimitedClient {
    client: Client,
    rate_limiter: RateLimiter,
}

impl RateLimitedClient {
    pub fn new(user_agent: &str, timeout_secs: u64, rate_limit_ms: u64) -> Result<Self> {
        let client = Self::build_client(user_agent, timeout_secs)?;
        let rate_limiter = RateLimiter::new(rate_limit_ms);

        Ok(Self {
            client,
            rate_limiter,
        })
    }

    pub async fn get(&mut self, url: &str) -> Result<reqwest::Response> {
        self.rate_limiter.wait().await;
        self.send_get_request(url).await
    }

    fn build_client(user_agent: &str, timeout_secs: u64) -> Result<Client> {
        Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .context("Failed to build HTTP client")
    }

    async fn send_get_request(&self, url: &str) -> Result<reqwest::Response> {
        self.client
            .get(url)
            .send()
            .await
            .context("Failed to send GET request")
    }
}
