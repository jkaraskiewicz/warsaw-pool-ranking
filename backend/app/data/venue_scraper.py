"""Venue page scraper for discovering tournaments."""

import requests
from bs4 import BeautifulSoup
from typing import List, Set, Optional
import logging
import time
import re
from urllib.parse import quote_plus

logger = logging.getLogger(__name__)


class VenueScraper:
    """
    Scrapes CueScore venue pages to discover tournament IDs.

    Handles:
    - Pagination through venue tournament pages
    - Tournament ID extraction
    - Rate limiting
    - Error handling
    """

    def __init__(self, rate_limit: float = 1.0):
        """
        Initialize venue scraper.

        Args:
            rate_limit: Requests per second limit (default 1.0)
        """
        self.rate_limit = rate_limit
        self.last_request_time = 0
        self.base_url = "https://cuescore.com"

        logger.info(f"VenueScraper initialized with rate limit: {rate_limit} req/sec")

    def _rate_limit_wait(self):
        """Enforce rate limiting."""
        if self.rate_limit <= 0:
            return

        min_interval = 1.0 / self.rate_limit
        time_since_last = time.time() - self.last_request_time

        if time_since_last < min_interval:
            sleep_time = min_interval - time_since_last
            time.sleep(sleep_time)

        self.last_request_time = time.time()

    def _fetch_page(self, url: str) -> Optional[BeautifulSoup]:
        """
        Fetch and parse a web page.

        Args:
            url: URL to fetch

        Returns:
            BeautifulSoup object, or None if failed
        """
        self._rate_limit_wait()

        logger.debug(f"Fetching page: {url}")

        try:
            response = requests.get(
                url,
                timeout=30,
                headers={'User-Agent': 'WarsawPoolRankings/1.0'}
            )
            response.raise_for_status()

            soup = BeautifulSoup(response.content, 'lxml')
            return soup

        except requests.RequestException as e:
            logger.error(f"Failed to fetch {url}: {e}")
            return None

    def scrape_venue_tournaments(
        self,
        venue_id: str,
        venue_name: str,
        max_pages: int = None
    ) -> Set[str]:
        """
        Scrape all tournament IDs from a venue's tournament page.

        Args:
            venue_id: CueScore venue ID (e.g., "12345")
            venue_name: Venue display name (e.g., "147 Break Nowogrodzka") - will be URL-encoded
            max_pages: Maximum pages to scrape (None = no limit)

        Returns:
            Set of tournament IDs found

        Example:
            >>> scraper = VenueScraper()
            >>> tournament_ids = scraper.scrape_venue_tournaments(
            ...     "12345",
            ...     "147 Break Nowogrodzka"
            ... )
            >>> print(f"Found {len(tournament_ids)} tournaments")
        """
        # URL-encode the venue name (spaces become +, special chars encoded)
        venue_name_encoded = quote_plus(venue_name)

        logger.info(f"Scraping tournaments for venue {venue_id} ({venue_name})")

        tournament_ids = set()
        page_num = 1

        while True:
            if max_pages and page_num > max_pages:
                logger.info(f"Reached max pages limit ({max_pages})")
                break

            # Construct page URL using URL-encoded venue name
            if page_num == 1:
                url = f"{self.base_url}/venue/{venue_name_encoded}/{venue_id}/tournaments"
            else:
                url = f"{self.base_url}/venue/{venue_name_encoded}/{venue_id}/tournaments?&page={page_num}"

            logger.info(f"Scraping page {page_num}: {url}")

            # Fetch page
            soup = self._fetch_page(url)
            if soup is None:
                logger.error(f"Failed to fetch page {page_num}, stopping")
                break

            # Extract tournament IDs from this page
            page_tournament_ids = self._extract_tournament_ids(soup)

            if not page_tournament_ids:
                logger.info(f"No tournaments found on page {page_num}, stopping")
                break

            logger.info(f"Found {len(page_tournament_ids)} tournaments on page {page_num}")
            tournament_ids.update(page_tournament_ids)

            # Check if there's a "Next" link
            if not self._has_next_page(soup):
                logger.info("No 'Next' link found, reached last page")
                break

            page_num += 1

        logger.info(
            f"Scraping complete for venue {venue_id}: "
            f"found {len(tournament_ids)} tournaments across {page_num} pages"
        )

        return tournament_ids

    def _extract_tournament_ids(self, soup: BeautifulSoup) -> List[str]:
        """
        Extract tournament IDs from a venue tournaments page.

        Args:
            soup: BeautifulSoup parsed page

        Returns:
            List of tournament IDs

        The tournament links are typically in format:
        /tournament/{tournament-name}/{tournament-id}
        """
        tournament_ids = []

        # Find all tournament links
        # Pattern: <a href="/tournament/.../{id}">
        tournament_links = soup.find_all('a', href=re.compile(r'/tournament/[^/]+/(\d+)'))

        for link in tournament_links:
            href = link.get('href')

            # Extract ID from URL using regex
            match = re.search(r'/tournament/[^/]+/(\d+)', href)
            if match:
                tournament_id = match.group(1)
                tournament_ids.append(tournament_id)

        # Remove duplicates while preserving order
        unique_ids = []
        seen = set()
        for tid in tournament_ids:
            if tid not in seen:
                unique_ids.append(tid)
                seen.add(tid)

        return unique_ids

    def _has_next_page(self, soup: BeautifulSoup) -> bool:
        """
        Check if there's a "Next" pagination link on the page.

        Args:
            soup: BeautifulSoup parsed page

        Returns:
            True if "Next »" link exists
        """
        # Look for "Next »" link
        # Common patterns: <a>Next »</a> or <a class="next_page">Next »</a>
        next_link = soup.find('a', string=re.compile(r'Next\s*»'))

        if next_link:
            logger.debug("Found 'Next »' link")
            return True

        # Alternative: look for pagination with "next" class
        next_link = soup.find('a', class_=re.compile(r'next', re.I))

        if next_link and next_link.get('href'):
            logger.debug("Found pagination 'next' link")
            return True

        return False

    def scrape_multiple_venues(
        self,
        venues: List[dict],
        max_pages_per_venue: int = None
    ) -> dict[str, Set[str]]:
        """
        Scrape tournaments from multiple venues.

        Args:
            venues: List of dicts with keys 'venue_id' and 'venue_name_slug'
            max_pages_per_venue: Max pages per venue (None = no limit)

        Returns:
            Dictionary mapping venue_id to set of tournament_ids

        Example:
            >>> venues = [
            ...     {"venue_id": "12345", "venue_name_slug": "147-Break-Nowogrodzka"},
            ...     {"venue_id": "67890", "venue_name_slug": "Pool-Hall-Warsaw"}
            ... ]
            >>> results = scraper.scrape_multiple_venues(venues)
        """
        logger.info(f"Scraping {len(venues)} venues")

        results = {}

        for idx, venue in enumerate(venues, 1):
            venue_id = venue['venue_id']
            venue_name_slug = venue['venue_name_slug']

            logger.info(f"Scraping venue {idx}/{len(venues)}: {venue_id}")

            try:
                tournament_ids = self.scrape_venue_tournaments(
                    venue_id,
                    venue_name_slug,
                    max_pages=max_pages_per_venue
                )
                results[venue_id] = tournament_ids

            except Exception as e:
                logger.error(f"Failed to scrape venue {venue_id}: {e}")
                results[venue_id] = set()

        total_tournaments = sum(len(ids) for ids in results.values())
        logger.info(
            f"Scraping complete: {total_tournaments} tournaments from {len(venues)} venues"
        )

        return results
