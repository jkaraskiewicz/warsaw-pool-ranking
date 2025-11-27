#!/usr/bin/env python3
"""Initialize database with schema and run initial data collection.

Usage:
    python scripts/init_database.py [--auto]

Options:
    --auto    Skip confirmation prompts (for Docker automation)
"""

import sys
import os
from pathlib import Path
import argparse

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from sqlalchemy import create_engine, text
from app.database import Base, engine, SessionLocal
from app.config import settings
from config.venues import WARSAW_VENUES
from weekly_update import WeeklyUpdateOrchestrator
import logging

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def create_tables():
    """Create all database tables from schema."""
    logger.info("Creating database tables...")

    # Read and execute schema.sql
    # In Docker, database directory is mounted at /database
    schema_path = Path("/database/schema.sql")
    if not schema_path.exists():
        # Fallback for local development
        schema_path = Path(__file__).parent.parent.parent / "database" / "schema.sql"

    with open(schema_path, 'r') as f:
        schema_sql = f.read()

    # Split by statement (simple approach - split on semicolons outside of quotes)
    # Execute schema
    with engine.connect() as conn:
        # Enable execution of multiple statements
        for statement in schema_sql.split(';'):
            statement = statement.strip()
            if statement and not statement.startswith('--'):
                try:
                    conn.execute(text(statement))
                    conn.commit()
                except Exception as e:
                    logger.warning(f"Statement execution note: {e}")

    logger.info("✓ Database tables created successfully")


def run_initial_data_collection():
    """Run initial data collection from configured venues."""
    logger.info(f"Starting initial data collection from {len(WARSAW_VENUES)} venue(s)...")

    for venue in WARSAW_VENUES:
        logger.info(f"  - {venue['name']}")

    # Create database session
    db = SessionLocal()
    try:
        # Convert WARSAW_VENUES format to match orchestrator expectations
        venues_for_orchestrator = [
            {
                'venue_id': venue['id'],
                'venue_name': venue['name']
            }
            for venue in WARSAW_VENUES
        ]

        # Use cache directory for faster re-runs during development
        cache_dir = Path("/app/cache")
        orchestrator = WeeklyUpdateOrchestrator(db=db, dry_run=False, cache_dir=cache_dir)
        orchestrator.run(venues_for_orchestrator)

        logger.info("✓ Initial data collection completed")
    finally:
        db.close()


def main():
    """Main initialization process."""
    parser = argparse.ArgumentParser(description='Initialize database with schema and data')
    parser.add_argument('--auto', action='store_true', help='Skip confirmation prompts')
    args = parser.parse_args()

    logger.info("=== Warsaw Pool Rankings - Database Initialization ===")

    # Step 1: Create tables
    logger.info("\nStep 1: Creating database schema...")
    create_tables()

    # Step 2: Collect initial data
    logger.info("\nStep 2: Collecting initial tournament data...")
    logger.info("This may take a while depending on the number of tournaments...")

    if not args.auto:
        confirm = input("\nReady to start data collection? (y/n): ")
        if confirm.lower() != 'y':
            logger.info("Data collection cancelled. Run this script again when ready.")
            return

    run_initial_data_collection()

    logger.info("\n=== Initialization Complete ===")
    logger.info("Your database is now populated with historical data.")
    logger.info("You can now start the FastAPI server and frontend.")


if __name__ == "__main__":
    main()
