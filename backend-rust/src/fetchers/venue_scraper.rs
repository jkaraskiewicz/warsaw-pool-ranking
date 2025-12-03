use anyhow::{Context, Result};
use log::info;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashSet;

use crate::http::RateLimitedClient;
use crate::pagination::{PageIterator, PaginationConfig};

const BASE_URL: &str = "https://cuescore.com";
const RATE_LIMIT_MS: u64 = 1000;
const USER_AGENT: &str = "WarsawPoolRankings/2.0";
const TIMEOUT_SECS: u64 = 30;

/// Web scraper for discovering tournament IDs from CueScore venue pages
pub struct VenueScraper {
    client: RateLimitedClient,
    tournament_id_regex: Regex,
}

impl VenueScraper {
    /// Create a new venue scraper
    pub fn new() -> Result<Self> {
        let client = RateLimitedClient::new(USER_AGENT, TIMEOUT_SECS, RATE_LIMIT_MS)?;
        let tournament_id_regex = Self::compile_regex()?;

        Ok(Self {
            client,
            tournament_id_regex,
        })
    }

    /// Scrape tournament IDs from a venue's tournament pages
    pub async fn scrape_venue_tournaments(
        &mut self,
        venue_id: i64,
        venue_name: &str,
        max_pages: Option<usize>,
    ) -> Result<HashSet<i64>> {
        info!("Discovering tournaments from venue: {} (ID: {})", venue_name, venue_id);

        let venue_name_encoded = urlencoding::encode(venue_name);
        let config = Self::build_pagination_config(max_pages);
        let mut pages = PageIterator::new(config);
        let mut all_ids = HashSet::new();

        loop {
            if pages.has_reached_max() {
                break;
            }

            let url = Self::build_url(&venue_name_encoded, venue_id, pages.current_page());
            info!("  → Page {}...", pages.current_page());

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

            pages.advance();
        }

        info!("  → Found {} tournaments total", all_ids.len());
        Ok(all_ids)
    }

    // --- Construction Helpers ---

    fn compile_regex() -> Result<Regex> {
        Regex::new(r"/tournament/[^/]+/(\d+)")
            .context("Failed to compile tournament ID regex")
    }

    // --- Pagination Configuration ---

    fn build_pagination_config(max_pages: Option<usize>) -> PaginationConfig {
        let mut config = PaginationConfig::new();
        if let Some(max) = max_pages {
            config = config.with_max_pages(max);
        }
        config
    }

    // --- URL Building ---

    fn build_url(venue_name: &str, venue_id: i64, page: usize) -> String {
        let base = format!("{}/venue/{}/{}/tournaments", BASE_URL, venue_name, venue_id);
        crate::pagination::build_paginated_url_with_params(&base, page)
    }

    // --- Pagination Logic ---

    fn has_next_page(html: &Html) -> bool {
        let selector = Selector::parse("a:contains('Next »')").ok();
        if let Some(sel) = selector {
            return html.select(&sel).next().is_some();
        }
        false
    }

    // --- HTTP Fetching ---

    async fn fetch_page(&mut self, url: &str) -> Result<Html> {
        let response = self.client.get(url).await?;

        self.check_response_status(&response)?;

        let html_text = self.extract_html_text(response).await?;
        Ok(Html::parse_document(&html_text))
    }

    fn check_response_status(&self, response: &reqwest::Response) -> Result<()> {
        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }
        Ok(())
    }

    async fn extract_html_text(&self, response: reqwest::Response) -> Result<String> {
        response.text().await.context("Failed to extract HTML text")
    }

    // --- Tournament ID Extraction ---

    fn extract_ids(&self, html: &Html) -> Vec<i64> {
        let selector = Selector::parse("a[href*='/tournament/']")
            .expect("Valid selector");

        let mut ids = Vec::new();
        let mut seen = HashSet::new();

        for element in html.select(&selector) {
            let href = element.value().attr("href");
            if let Some(id) = self.parse_tournament_id(href) {
                if seen.insert(id) {
                    info!("  Found tournament ID {} from href: {:?}", id, href);
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
