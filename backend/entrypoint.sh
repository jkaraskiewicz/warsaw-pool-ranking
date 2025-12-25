#!/bin/bash
set -e

# Define database file path (default to env var or standard name)
DB_FILE=${DATABASE_PATH:-warsaw_pool_ranking.db}

# Check if the database file exists and is valid
DB_VALID=false
if [ -f "$DB_FILE" ]; then
    echo "Checking database integrity..."
    if sqlite3 "$DB_FILE" "SELECT count(*) FROM players;" >/dev/null 2>&1; then
        echo "Database check passed."
        DB_VALID=true
    else
        echo "Database file '$DB_FILE' exists but appears invalid (missing players table)."
    fi
fi

if [ "$DB_VALID" = false ]; then
    echo "Initializing database..."

    # Ensure directory exists (e.g., if using /app/data/...)
    mkdir -p "$(dirname "$DB_FILE")"

    # Check if we have cached data (to skip full ingest if possible)
    if [ ! -f "cache/parsed/tournaments.json" ]; then
        echo "Parsed cache file not found. Running full data ingestion..."
        ./warsaw_pool_ranking ingest
    else
        echo "Cache found. Skipping ingestion."
    fi

    echo "Running data processing to initialize database..."
    ./warsaw_pool_ranking process
else
    echo "Database '$DB_FILE' is valid. Skipping initialization."
fi

echo "Starting Server on port 8000..."
exec ./warsaw_pool_ranking serve --port 8000
