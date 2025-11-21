#!/usr/bin/env python3
"""Run weekly update to refresh rankings with latest data.

This should be run weekly (e.g., via cron job) to:
1. Discover new tournaments
2. Fetch new game data
3. Recalculate all ratings from scratch
4. Update rating snapshots

Usage:
    python scripts/run_weekly_update.py
"""

import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from config.venues import WARSAW_VENUES
from weekly_update import WeeklyUpdateOrchestrator
import logging

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def main():
    """Run weekly update process."""
    logger.info("=== Warsaw Pool Rankings - Weekly Update ===")
    logger.info(f"Updating data from {len(WARSAW_VENUES)} venue(s)...")

    for venue in WARSAW_VENUES:
        logger.info(f"  - {venue['name']}")

    orchestrator = WeeklyUpdateOrchestrator()
    orchestrator.run(WARSAW_VENUES)

    logger.info("=== Weekly Update Complete ===")


if __name__ == "__main__":
    main()
