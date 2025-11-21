#!/bin/bash
# Quick start script for Warsaw Pool Rankings

set -e

echo "=== Warsaw Pool Rankings - Quick Start ==="
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose is not installed. Please install docker-compose first."
    exit 1
fi

# Check if .env exists, if not create from example
if [ ! -f .env ]; then
    echo "Creating .env file from .env.example..."
    cp .env.example .env
    echo "✓ .env file created"
fi

# Check if venues are configured
if ! grep -q "147-break-nowogrodzka" backend/config/venues.py; then
    echo "⚠️  Warning: No venues configured in backend/config/venues.py"
    echo "   Edit this file to add Warsaw pool venues before initialization."
fi

echo ""
echo "Building Docker containers..."
docker-compose build

echo ""
echo "Starting services..."
docker-compose up -d

echo ""
echo "Waiting for database to be ready..."
sleep 5

# Check if database has data
PLAYER_COUNT=$(docker-compose exec -T db psql -U pool_app -d warsaw_pool_rankings -t -c "SELECT COUNT(*) FROM players;" 2>/dev/null | tr -d ' ' || echo "0")

if [ "$PLAYER_COUNT" -eq 0 ]; then
    echo ""
    echo "Database is empty. Would you like to initialize it now?"
    echo "This will:"
    echo "  1. Create database tables"
    echo "  2. Scrape tournaments from configured venues"
    echo "  3. Fetch game data from CueScore API"
    echo "  4. Calculate ratings for all players"
    echo ""
    echo "This may take 10-30 minutes depending on tournament count."
    echo ""
    read -p "Initialize database? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        docker-compose exec backend python scripts/init_database.py --auto
    else
        echo "Skipping initialization. Run 'make init' when ready."
    fi
else
    echo "✓ Database already contains $PLAYER_COUNT players"
fi

echo ""
echo "=== Services Started ==="
echo ""
echo "Frontend:  http://localhost"
echo "Backend:   http://localhost:8000"
echo "API Docs:  http://localhost:8000/docs"
echo ""
echo "View logs: docker-compose logs -f"
echo "Stop:      docker-compose down"
echo ""
