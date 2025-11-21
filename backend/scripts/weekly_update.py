"""Weekly rating recalculation orchestration script.

This script should be run weekly (e.g., via cron on Sunday nights) to:
1. Fetch new tournament data from CueScore
2. Update database with new games
3. Run full rating simulation
4. Update ratings and snapshots tables

Usage:
    python -m scripts.weekly_update [--venues-file venues.json] [--dry-run]
"""

import sys
import os
import argparse
import logging
import json
from pathlib import Path
from datetime import datetime
from typing import List, Dict

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from sqlalchemy.orm import Session
from sqlalchemy import delete

from app.database import SessionLocal, engine
from app.models import (
    Base, Player, Venue, Tournament, Game,
    Rating, RatingSnapshot
)
from app.data.venue_scraper import VenueScraper
from app.data.cuescore_api import CueScoreAPIClient
from app.data.parser import TournamentParser
from app.rating.simulator import WeeklySimulator
from app.config import settings

import pandas as pd

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('weekly_update.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)


class WeeklyUpdateOrchestrator:
    """Orchestrates the weekly rating update process."""

    def __init__(self, db: Session, dry_run: bool = False):
        """
        Initialize orchestrator.

        Args:
            db: Database session
            dry_run: If True, don't commit changes to database
        """
        self.db = db
        self.dry_run = dry_run

        self.scraper = VenueScraper()
        self.api_client = CueScoreAPIClient()
        self.parser = TournamentParser()
        self.simulator = WeeklySimulator(
            calculation_version=settings.calculation_version
        )

        logger.info(f"WeeklyUpdateOrchestrator initialized (dry_run={dry_run})")

    def run(self, venues: List[Dict]):
        """
        Run the complete weekly update process.

        Args:
            venues: List of venue dictionaries with 'venue_id' and 'venue_name'
        """
        logger.info("="*60)
        logger.info("STARTING WEEKLY UPDATE")
        logger.info("="*60)

        try:
            # Step 1: Discover tournaments
            tournament_ids = self._discover_tournaments(venues)

            # Step 2: Fetch tournament data
            tournaments_data = self._fetch_tournaments(tournament_ids)

            # Step 3: Update database with new data
            self._update_database(tournaments_data)

            # Step 4: Run rating simulation
            self._run_simulation()

            # Step 5: Commit or rollback
            if self.dry_run:
                logger.info("DRY RUN - Rolling back all changes")
                self.db.rollback()
            else:
                logger.info("Committing changes to database")
                self.db.commit()

            logger.info("="*60)
            logger.info("WEEKLY UPDATE COMPLETE")
            logger.info("="*60)

        except Exception as e:
            logger.error(f"Weekly update failed: {e}", exc_info=True)
            self.db.rollback()
            raise

    def _discover_tournaments(self, venues: List[Dict]) -> set:
        """
        Step 1: Scrape venues to discover tournament IDs.

        Args:
            venues: List of venue info dictionaries

        Returns:
            Set of tournament IDs
        """
        logger.info(f"Step 1: Discovering tournaments from {len(venues)} venues")

        all_tournament_ids = set()

        for venue in venues:
            venue_id = venue['venue_id']
            venue_name = venue['venue_name']

            logger.info(f"Scraping venue: {venue_name} ({venue_id})")

            try:
                tournament_ids = self.scraper.scrape_venue_tournaments(
                    venue_id,
                    venue_name,
                    max_pages=2  # Limit to 2 pages for testing
                )

                logger.info(f"Found {len(tournament_ids)} tournaments at {venue_name}")
                all_tournament_ids.update(tournament_ids)

                # Store/update venue in database
                self._upsert_venue(venue)

            except Exception as e:
                logger.error(f"Failed to scrape venue {venue_id}: {e}")
                continue

        logger.info(f"Discovery complete: {len(all_tournament_ids)} unique tournaments")

        return all_tournament_ids

    def _fetch_tournaments(self, tournament_ids: set) -> List[Dict]:
        """
        Step 2: Fetch tournament details from API.

        Args:
            tournament_ids: Set of tournament IDs

        Returns:
            List of parsed tournament data dictionaries
        """
        logger.info(f"Step 2: Fetching {len(tournament_ids)} tournaments from API")

        # Filter to only new/updated tournaments
        existing_ids = set(
            t[0] for t in self.db.query(Tournament.cuescore_id).all()
        )

        new_tournament_ids = list(tournament_ids - existing_ids)

        if not new_tournament_ids:
            logger.info("No new tournaments to fetch")
            return []

        logger.info(f"Fetching {len(new_tournament_ids)} new tournaments")

        tournaments_data = []

        for idx, tournament_id in enumerate(new_tournament_ids, 1):
            if idx % 100 == 0:
                logger.info(f"Progress: {idx}/{len(new_tournament_ids)}")

            try:
                # Fetch from API
                raw_data = self.api_client.get_tournament(tournament_id)

                if raw_data:
                    # Parse tournament data (returns None for non-pool disciplines)
                    parsed = self.parser.parse_tournament(raw_data)
                    if parsed:  # Only include pool tournaments
                        tournaments_data.append(parsed)

            except Exception as e:
                logger.error(f"Failed to fetch/parse tournament {tournament_id}: {e}")
                continue

        logger.info(f"Fetched {len(tournaments_data)} tournaments successfully")

        return tournaments_data

    def _update_database(self, tournaments_data: List[Dict]):
        """
        Step 3: Update database with new tournament/player/game data.

        Args:
            tournaments_data: List of parsed tournament dictionaries
        """
        logger.info(f"Step 3: Updating database with {len(tournaments_data)} tournaments")

        total_players = 0
        total_games = 0

        for tournament_data in tournaments_data:
            try:
                # Upsert players
                for participant in tournament_data['participants']:
                    self._upsert_player(participant)
                    total_players += 1

                # Upsert tournament
                tournament_db_id = self._upsert_tournament(tournament_data['tournament_info'])

                # Insert games
                for game_data in tournament_data['games']:
                    self._insert_game(game_data, tournament_db_id)
                    total_games += 1

            except Exception as e:
                logger.error(f"Failed to update DB for tournament {tournament_data.get('tournament_info', {}).get('cuescore_id')}: {e}")
                continue

        logger.info(f"Database update complete: {total_players} players, {total_games} games")

    def _run_simulation(self):
        """
        Step 4: Run rating simulation and update ratings/snapshots.
        """
        logger.info("Step 4: Running rating simulation")

        # Fetch all games from database
        games = self.db.query(Game).all()

        if not games:
            logger.warning("No games in database, skipping simulation")
            return

        # Convert to DataFrame
        games_df = pd.DataFrame([{
            'player_a_id': g.player_a_id,
            'player_b_id': g.player_b_id,
            'winner_id': g.winner_id,
            'played_at': g.played_at
        } for g in games])

        logger.info(f"Running simulation with {len(games_df)} games")

        # Run simulation
        current_ratings_df, snapshots_df = self.simulator.calculate_current_ratings(games_df)

        # Update ratings table
        self._update_ratings_table(current_ratings_df)

        # Replace snapshots table
        self._replace_snapshots_table(snapshots_df)

        logger.info("Simulation complete")

    def _upsert_venue(self, venue_data: Dict):
        """Insert or update venue."""
        venue = self.db.query(Venue).filter(
            Venue.cuescore_id == venue_data['venue_id']
        ).first()

        if not venue:
            # URL-encode the venue name for the URL
            from urllib.parse import quote_plus
            venue_name_encoded = quote_plus(venue_data['venue_name'])

            venue = Venue(
                cuescore_id=venue_data['venue_id'],
                name=venue_data['venue_name'],
                cuescore_url=f"https://cuescore.com/venue/{venue_name_encoded}/{venue_data['venue_id']}"
            )
            self.db.add(venue)

    def _upsert_player(self, player_data: Dict):
        """Insert or update player."""
        player = self.db.query(Player).filter(
            Player.cuescore_id == player_data['cuescore_id']
        ).first()

        if not player:
            player = Player(**player_data)
            self.db.add(player)
        else:
            # Update name if changed
            player.name = player_data['name']
            player.cuescore_profile_url = player_data.get('cuescore_profile_url')

    def _upsert_tournament(self, tournament_data: Dict) -> int:
        """Insert or update tournament, return DB ID."""
        tournament = self.db.query(Tournament).filter(
            Tournament.cuescore_id == tournament_data['cuescore_id']
        ).first()

        if not tournament:
            # Find venue
            venue = self.db.query(Venue).filter(
                Venue.cuescore_id == tournament_data.get('venue_cuescore_id')
            ).first() if tournament_data.get('venue_cuescore_id') else None

            tournament = Tournament(
                cuescore_id=tournament_data['cuescore_id'],
                name=tournament_data['name'],
                venue_id=venue.id if venue else None,
                start_date=tournament_data.get('start_date'),
                end_date=tournament_data.get('end_date')
            )
            self.db.add(tournament)
            self.db.flush()  # Get ID

        return tournament.id

    def _insert_game(self, game_data: Dict, tournament_db_id: int):
        """Insert game record."""
        # Get player database IDs
        player_a = self.db.query(Player).filter(
            Player.cuescore_id == game_data['player_a_cuescore_id']
        ).first()

        player_b = self.db.query(Player).filter(
            Player.cuescore_id == game_data['player_b_cuescore_id']
        ).first()

        winner = self.db.query(Player).filter(
            Player.cuescore_id == game_data['winner_cuescore_id']
        ).first()

        if not (player_a and player_b and winner):
            logger.warning(f"Skipping game: missing players")
            return

        # Check if game already exists
        existing = self.db.query(Game).filter(
            Game.cuescore_match_id == game_data['cuescore_match_id'],
            Game.player_a_id == player_a.id,
            Game.player_b_id == player_b.id
        ).first()

        if existing:
            return  # Skip duplicates

        game = Game(
            cuescore_match_id=game_data['cuescore_match_id'],
            tournament_id=tournament_db_id,
            player_a_id=player_a.id,
            player_b_id=player_b.id,
            winner_id=winner.id,
            played_at=game_data.get('played_at') or datetime.now()
        )

        self.db.add(game)

    def _update_ratings_table(self, ratings_df: pd.DataFrame):
        """Update current ratings table."""
        logger.info(f"Updating ratings table with {len(ratings_df)} player ratings")

        for _, row in ratings_df.iterrows():
            rating = self.db.query(Rating).filter(
                Rating.player_id == row['player_id']
            ).first()

            if rating:
                # Update existing
                rating.rating = row['rating']
                rating.games_played = row['games_played']
                rating.total_wins = row['total_wins']
                rating.total_losses = row['total_losses']
                rating.confidence_level = row['confidence_level']
                rating.best_rating = row.get('best_rating')
                rating.best_rating_date = row.get('best_rating_date')
                rating.calculated_at = row['calculated_at']
            else:
                # Insert new
                rating = Rating(**row.to_dict())
                self.db.add(rating)

    def _replace_snapshots_table(self, snapshots_df: pd.DataFrame):
        """Replace entire snapshots table."""
        logger.info(f"Replacing snapshots table with {len(snapshots_df)} snapshots")

        # Delete all existing snapshots
        self.db.execute(delete(RatingSnapshot))

        # Insert new snapshots
        for _, row in snapshots_df.iterrows():
            snapshot = RatingSnapshot(**row.to_dict())
            self.db.add(snapshot)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Weekly rating update script")
    parser.add_argument(
        '--venues-file',
        type=str,
        default='venues.json',
        help='Path to venues JSON file'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Run without committing changes'
    )

    args = parser.parse_args()

    # Load venues
    venues_path = Path(args.venues_file)
    if not venues_path.exists():
        logger.error(f"Venues file not found: {venues_path}")
        sys.exit(1)

    with open(venues_path) as f:
        venues = json.load(f)

    logger.info(f"Loaded {len(venues)} venues from {venues_path}")

    # Create database session
    db = SessionLocal()

    try:
        # Run orchestrator
        orchestrator = WeeklyUpdateOrchestrator(db, dry_run=args.dry_run)
        orchestrator.run(venues)

    finally:
        db.close()


if __name__ == '__main__':
    main()
