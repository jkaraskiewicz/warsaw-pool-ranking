use anyhow::{Context, Result};
use log::info;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;

const BASE_URL: &str = "https://cuescore.com";
const RATE_LIMIT_MS: u64 = 1000;

/// Web scraper for discovering tournament IDs from CueScore venue pages
pub struct VenueScraper {
    client: Client,
    rate_limit_delay: Duration,
    tournament_id_regex: Regex,
}

impl VenueScraper {
    /// Create a new venue scraper
    pub fn new() -> Result<Self> {
        let client = Self::build_client()?;
        let tournament_id_regex = Self::compile_regex()?;

        Ok(Self {
            client,
            rate_limit_delay: Duration::from_millis(RATE_LIMIT_MS),
            tournament_id_regex,
        })
    }

    /// Scrape tournament IDs from a venue's tournament pages
    pub async fn scrape_venue_tournaments(
        &self,
        venue_id: i64,
        venue_name: &str,
        max_pages: Option<usize>,
    ) -> Result<HashSet<i64>> {
        info!("Discovering tournaments from venue: {} (ID: {})", venue_name, venue_id);

        let venue_name_encoded = urlencoding::encode(venue_name);
        let mut all_ids = HashSet::new();
        let mut page = 1;

        loop {
            if Self::reached_max_pages(page, max_pages) {
                break;
            }

            let url = Self::build_url(&venue_name_encoded, venue_id, page);
            info!("  → Page {}...", page);

            Self::rate_limit(page, self.rate_limit_delay).await;

            let html = match self.fetch_page(&url).await {
                Ok(html) => html,
                Err(_) => break,
            };

            let page_ids = self.extract_ids(&html);
            if page_ids.is_empty() {
                break;
            }

            all_ids.extend(page_ids);

            if !Self::has_next_page(&html) {
                break;
            }

            page += 1;
        }

        info!("  → Found {} tournaments total", all_ids.len());
        Ok(all_ids)
    }

    // --- Construction Helpers ---

    fn build_client() -> Result<Client> {
        Client::builder()
            .user_agent("WarsawPoolRankings/2.0")
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")
    }

    fn compile_regex() -> Result<Regex> {
        Regex::new(r"/tournament/[^/]+/(\d+)")
            .context("Failed to compile tournament ID regex")
    }

    // --- URL Building ---

    fn build_url(venue_name: &str, venue_id: i64, page: usize) -> String {
        if page == 1 {
            format!("{}/venue/{}/{}/tournaments", BASE_URL, venue_name, venue_id)
        } else {
            format!("{}/venue/{}/{}/tournaments?&page={}", BASE_URL, venue_name, venue_id, page)
        }
    }

    // --- Pagination Logic ---

    fn reached_max_pages(current: usize, max: Option<usize>) -> bool {
        max.map_or(false, |m| current > m)
    }

    async fn rate_limit(page: usize, delay: Duration) {
        if page > 1 {
            sleep(delay).await;
        }
    }

    fn has_next_page(html: &Html) -> bool {
        let selector = Selector::parse("a:contains('Next »')").ok();
        if let Some(sel) = selector {
            return html.select(&sel).next().is_some();
        }
        false
    }

    // --- HTTP Fetching ---

    async fn fetch_page(&self, url: &str) -> Result<Html> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let html_text = response.text().await?;
        Ok(Html::parse_document(&html_text))
    }

    // --- Tournament ID Extraction ---

    fn extract_ids(&self, html: &Html) -> Vec<i64> {
        let selector = Selector::parse("a[href*='/tournament/']")
            .expect("Valid selector");

        let mut ids = Vec::new();
        let mut seen = HashSet::new();

        for element in html.select(&selector) {
            if let Some(id) = self.parse_tournament_id(element.value().attr("href")) {
                if seen.insert(id) {
                    ids.push(id);
                }
            }
        }

        ids
    }

    fn parse_tournament_id(&self, href: Option<&str>) -> Option<i64> {
        let href = href?;
        let captures = self.tournament_id_regex.captures(href)?;
        let id_str = captures.get(1)?.as_str();
        id_str.parse().ok()
    }
}
