"""CueScore API client for fetching tournament data."""

import requests
from typing import Dict, List, Optional
import logging
from tenacity import (
    retry,
    stop_after_attempt,
    wait_exponential,
    retry_if_exception_type
)
import time

from app.config import settings

logger = logging.getLogger(__name__)


class CueScoreAPIClient:
    """
    Client for interacting with CueScore API.

    Handles:
    - Tournament data fetching
    - Rate limiting (1 req/sec default)
    - Automatic retries with exponential backoff
    - Error handling
    """

    def __init__(
        self,
        base_url: str = None,
        rate_limit: float = None
    ):
        """
        Initialize CueScore API client.

        Args:
            base_url: Base URL for CueScore API (default from settings)
            rate_limit: Requests per second limit (default from settings)
        """
        self.base_url = base_url or settings.cuescore_api_base_url
        self.rate_limit = rate_limit or settings.cuescore_rate_limit
        self.last_request_time = 0

        logger.info(
            f"CueScoreAPIClient initialized: {self.base_url}, "
            f"rate limit: {self.rate_limit} req/sec"
        )

    def _rate_limit_wait(self):
        """
        Enforce rate limiting by waiting if necessary.

        Ensures minimum time between requests based on rate_limit.
        """
        if self.rate_limit <= 0:
            return  # No rate limiting

        min_interval = 1.0 / self.rate_limit
        time_since_last = time.time() - self.last_request_time

        if time_since_last < min_interval:
            sleep_time = min_interval - time_since_last
            logger.debug(f"Rate limit: sleeping {sleep_time:.2f}s")
            time.sleep(sleep_time)

        self.last_request_time = time.time()

    @retry(
        stop=stop_after_attempt(3),
        wait=wait_exponential(multiplier=1, min=2, max=10),
        retry=retry_if_exception_type((requests.RequestException, requests.Timeout)),
        reraise=True
    )
    def _make_request(self, endpoint: str, params: Dict = None) -> Dict:
        """
        Make HTTP GET request to CueScore API with retries.

        Args:
            endpoint: API endpoint path
            params: Query parameters

        Returns:
            JSON response as dictionary

        Raises:
            requests.RequestException: If request fails after retries
        """
        self._rate_limit_wait()

        url = f"{self.base_url}{endpoint}"

        logger.debug(f"Making request to {url} with params {params}")

        try:
            response = requests.get(
                url,
                params=params,
                timeout=30,
                headers={'User-Agent': 'WarsawPoolRankings/1.0'}
            )
            response.raise_for_status()

            return response.json()

        except requests.HTTPError as e:
            logger.error(f"HTTP error for {url}: {e}")
            raise
        except requests.Timeout as e:
            logger.error(f"Timeout for {url}: {e}")
            raise
        except requests.RequestException as e:
            logger.error(f"Request exception for {url}: {e}")
            raise

    def get_tournament(self, tournament_id: str) -> Optional[Dict]:
        """
        Fetch tournament details by ID.

        Args:
            tournament_id: CueScore tournament ID

        Returns:
            Tournament data dictionary, or None if not found

        Example response structure:
            {
                "id": "72541144",
                "name": "Warsaw Open 2024",
                "startDate": "2024-03-15",
                "participants": [...],
                "matches": [...],
                ...
            }
        """
        logger.info(f"Fetching tournament {tournament_id}")

        try:
            data = self._make_request("/tournament/", params={"id": tournament_id})

            # API response doesn't include the ID, so we add it
            data['id'] = tournament_id

            logger.info(
                f"Tournament {tournament_id} fetched successfully: "
                f"{data.get('name', 'Unknown')}"
            )

            return data

        except requests.HTTPError as e:
            if e.response.status_code == 404:
                logger.warning(f"Tournament {tournament_id} not found (404)")
                return None
            raise

        except Exception as e:
            logger.error(f"Failed to fetch tournament {tournament_id}: {e}")
            raise

    def get_multiple_tournaments(
        self,
        tournament_ids: List[str]
    ) -> Dict[str, Optional[Dict]]:
        """
        Fetch multiple tournaments.

        Args:
            tournament_ids: List of tournament IDs to fetch

        Returns:
            Dictionary mapping tournament_id to tournament data (or None if failed)

        Note:
            Respects rate limiting, so this may take time for large lists
        """
        logger.info(f"Fetching {len(tournament_ids)} tournaments")

        results = {}

        for idx, tournament_id in enumerate(tournament_ids, 1):
            logger.debug(f"Fetching tournament {idx}/{len(tournament_ids)}: {tournament_id}")

            try:
                data = self.get_tournament(tournament_id)
                results[tournament_id] = data

            except Exception as e:
                logger.error(f"Failed to fetch tournament {tournament_id}: {e}")
                results[tournament_id] = None
                # Continue with other tournaments

        successful = sum(1 for v in results.values() if v is not None)
        logger.info(
            f"Fetched {successful}/{len(tournament_ids)} tournaments successfully"
        )

        return results
