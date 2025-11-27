use anyhow::{Context, Result};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

const BASE_URL: &str = "https://cuescore.com";
const RATE_LIMIT_DELAY_MS: u64 = 1000; // 1 request per second

/// Web scraper for CueScore venue pages
pub struct VenueScraper {
    client: Client,
    rate_limit_delay: Duration,
    tournament_id_regex: Regex,
}

impl VenueScraper {
    /// Create a new venue scraper
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("WarsawPoolRankings/2.0")
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        let tournament_id_regex = Regex::new(r"/tournament/[^/]+/(\d+)")
            .context("Failed to compile tournament ID regex")?;

        Ok(Self {
            client,
            rate_limit_delay: Duration::from_millis(RATE_LIMIT_DELAY_MS),
            tournament_id_regex,
        })
    }

    /// Scrape all tournament IDs from a venue's tournament page
    ///
    /// # Arguments
    /// * `venue_id` - CueScore venue ID (e.g., 12345)
    /// * `venue_name` - Venue display name (will be URL-encoded)
    /// * `max_pages` - Optional limit on pages to scrape
    ///
    /// # Example
    /// ```
    /// let scraper = VenueScraper::new()?;
    /// let tournament_ids = scraper.scrape_venue_tournaments(
    ///     12345,
    ///     "147 Break Nowogrodzka",
    ///     None
    /// ).await?;
    /// ```
    pub async fn scrape_venue_tournaments(
        &self,
        venue_id: i64,
        venue_name: &str,
        max_pages: Option<usize>,
    ) -> Result<HashSet<i64>> {
        info!("Scraping tournaments for venue {} ({})", venue_id, venue_name);

        let venue_name_encoded = urlencoding::encode(venue_name);
        let mut tournament_ids = HashSet::new();
        let mut page_num = 1;

        loop {
            if let Some(max) = max_pages {
                if page_num > max {
                    info!("Reached max pages limit ({})", max);
                    break;
                }
            }

            // Construct page URL
            let url = if page_num == 1 {
                format!(
                    "{}/venue/{}/{}/tournaments",
                    BASE_URL, venue_name_encoded, venue_id
                )
            } else {
                format!(
                    "{}/venue/{}/{}/tournaments?&page={}",
                    BASE_URL, venue_name_encoded, venue_id, page_num
                )
            };

            info!("Scraping page {}: {}", page_num, url);

            // Rate limiting
            if page_num > 1 {
                sleep(self.rate_limit_delay).await;
            }

            // Fetch page
            let html = match self.fetch_page(&url).await {
                Ok(html) => html,
                Err(e) => {
                    warn!("Failed to fetch page {}: {}", page_num, e);
                    break;
                }
            };

            // Parse and extract tournament IDs
            let page_tournament_ids = self.extract_tournament_ids(&html);

            if page_tournament_ids.is_empty() {
                info!("No tournaments found on page {}, stopping", page_num);
                break;
            }

            info!(
                "Found {} tournaments on page {}",
                page_tournament_ids.len(),
                page_num
            );
            tournament_ids.extend(page_tournament_ids);

            // Check if there's a next page
            if !self.has_next_page(&html) {
                info!("No 'Next' link found, reached last page");
                break;
            }

            page_num += 1;
        }

        info!(
            "Scraping complete for venue {}: found {} tournaments across {} pages",
            venue_id,
            tournament_ids.len(),
            page_num
        );

        Ok(tournament_ids)
    }

    /// Fetch and parse an HTML page
    async fn fetch_page(&self, url: &str) -> Result<Html> {
        debug!("Fetching page: {}", url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send HTTP request")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let html_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        Ok(Html::parse_document(&html_text))
    }

    /// Extract tournament IDs from HTML
    ///
    /// Looks for links in format: /tournament/{tournament-name}/{tournament-id}
    fn extract_tournament_ids(&self, html: &Html) -> Vec<i64> {
        let link_selector = Selector::parse("a[href*='/tournament/']")
            .expect("Failed to parse link selector");

        let mut tournament_ids = Vec::new();
        let mut seen = HashSet::new();

        for element in html.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                if let Some(captures) = self.tournament_id_regex.captures(href) {
                    if let Some(id_str) = captures.get(1) {
                        if let Ok(tournament_id) = id_str.as_str().parse::<i64>() {
                            // Deduplicate while preserving order
                            if !seen.contains(&tournament_id) {
                                tournament_ids.push(tournament_id);
                                seen.insert(tournament_id);
                            }
                        }
                    }
                }
            }
        }

        tournament_ids
    }

    /// Check if there's a "Next" pagination link
    fn has_next_page(&self, html: &Html) -> bool {
        // Look for "Next »" text in links
        let link_selector = Selector::parse("a").expect("Failed to parse link selector");

        for element in html.select(&link_selector) {
            let text = element.text().collect::<String>();
            if text.contains("Next") && text.contains("»") {
                debug!("Found 'Next »' link");
                return true;
            }
        }

        // Alternative: look for pagination with "next" class
        let next_selector = Selector::parse("a[class*='next']")
            .expect("Failed to parse next selector");

        if html.select(&next_selector).next().is_some() {
            debug!("Found pagination 'next' link");
            return true;
        }

        false
    }

    /// Scrape tournaments from multiple venues
    pub async fn scrape_multiple_venues(
        &self,
        venues: &[(i64, String)], // (venue_id, venue_name)
        max_pages_per_venue: Option<usize>,
    ) -> Result<Vec<(i64, HashSet<i64>)>> {
        info!("Scraping {} venues", venues.len());

        let mut results = Vec::new();

        for (idx, (venue_id, venue_name)) in venues.iter().enumerate() {
            info!("Scraping venue {}/{}: {}", idx + 1, venues.len(), venue_id);

            match self
                .scrape_venue_tournaments(*venue_id, venue_name, max_pages_per_venue)
                .await
            {
                Ok(tournament_ids) => {
                    results.push((*venue_id, tournament_ids));
                }
                Err(e) => {
                    warn!("Failed to scrape venue {}: {}", venue_id, e);
                    results.push((*venue_id, HashSet::new()));
                }
            }
        }

        let total_tournaments: usize = results.iter().map(|(_, ids)| ids.len()).sum();
        info!(
            "Scraping complete: {} tournaments from {} venues",
            total_tournaments,
            venues.len()
        );

        Ok(results)
    }
}

impl Default for VenueScraper {
    fn default() -> Self {
        Self::new().expect("Failed to create VenueScraper")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tournament_id_extraction() {
        let scraper = VenueScraper::new().unwrap();

        let html = Html::parse_document(
            r#"
            <html>
                <body>
                    <a href="/tournament/some-tournament-name/12345">Tournament 1</a>
                    <a href="/tournament/another-tournament/67890">Tournament 2</a>
                    <a href="/tournament/duplicate/12345">Duplicate</a>
                    <a href="/some-other-link">Not a tournament</a>
                </body>
            </html>
            "#,
        );

        let ids = scraper.extract_tournament_ids(&html);

        assert_eq!(ids.len(), 2); // Should deduplicate
        assert!(ids.contains(&12345));
        assert!(ids.contains(&67890));
    }

    #[test]
    fn test_next_page_detection() {
        let scraper = VenueScraper::new().unwrap();

        // Test with "Next »" link
        let html = Html::parse_document(
            r#"
            <html>
                <body>
                    <a href="?page=2">Next »</a>
                </body>
            </html>
            "#,
        );

        assert!(scraper.has_next_page(&html));

        // Test without next link
        let html = Html::parse_document(
            r#"
            <html>
                <body>
                    <a href="?page=1">Previous</a>
                </body>
            </html>
            "#,
        );

        assert!(!scraper.has_next_page(&html));
    }
}
