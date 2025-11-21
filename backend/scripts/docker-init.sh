#!/bin/bash
# Docker initialization script
# Runs inside the backend container to initialize the database with data

set -e

echo "=== Warsaw Pool Rankings - Docker Initialization ==="

# Wait for database to be ready
echo "Waiting for database to be ready..."
until pg_isready -h db -U pool_app -d warsaw_pool_rankings; do
  echo "Database is unavailable - sleeping"
  sleep 2
done

echo "Database is ready!"

# Check if database is already populated
PLAYER_COUNT=$(PGPASSWORD=${POSTGRES_PASSWORD:-poolpass123} psql -h db -U pool_app -d warsaw_pool_rankings -t -c "SELECT COUNT(*) FROM players;" 2>/dev/null || echo "0")

if [ "$PLAYER_COUNT" -gt 0 ]; then
  echo "Database already contains $PLAYER_COUNT players. Skipping initialization."
  echo "To reinitialize, drop the database or remove the Docker volume."
  exit 0
fi

echo "Database is empty. Starting data collection..."
echo "This may take 10-30 minutes depending on tournament count..."

# Run the initialization script
python scripts/init_database.py --auto

echo "=== Initialization Complete ==="
