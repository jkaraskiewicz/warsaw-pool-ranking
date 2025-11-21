# Database Setup Guide

This guide walks through setting up the PostgreSQL database and populating it with Warsaw pool tournament data.

## Prerequisites

- PostgreSQL 12+ installed and running
- Python 3.9+ with backend dependencies installed

## Step 1: Create PostgreSQL Database

```bash
# Login as postgres user
sudo -u postgres psql

# In psql prompt:
CREATE DATABASE warsaw_pool_rankings;
CREATE USER pool_app WITH PASSWORD 'your_password_here';
GRANT ALL PRIVILEGES ON DATABASE warsaw_pool_rankings TO pool_app;

# Exit psql
\q
```

## Step 2: Configure Database Connection

Edit `backend/.env` file (create if doesn't exist):

```env
DATABASE_URL=postgresql://pool_app:your_password_here@localhost:5432/warsaw_pool_rankings
```

Or use the default (if running PostgreSQL locally):
```env
DATABASE_URL=postgresql://localhost:5432/warsaw_pool_rankings
```

## Step 3: Configure Venues

Edit `backend/config/venues.py` to add Warsaw pool venues you want to track.

To find venue information:
1. Go to CueScore and find the venue page
2. Look at the URL: `https://cuescore.com/venue/{slug}/{id}/tournaments`
3. Extract the `id` and `slug`

Example:
```python
WARSAW_VENUES = [
    {
        "id": "67496954",
        "slug": "147-break-nowogrodzka",
        "name": "147 Break Nowogrodzka"
    },
    {
        "id": "another_id",
        "slug": "another-venue-slug",
        "name": "Another Venue Name"
    },
]
```

## Step 4: Initialize Database

Run the initialization script to:
1. Create all tables (from schema.sql)
2. Collect initial tournament data
3. Calculate initial ratings

```bash
cd backend
python scripts/init_database.py
```

**This will:**
- Create all database tables
- Scrape tournament IDs from configured venues
- Fetch tournament and game data from CueScore API
- Run the rating simulation on all historical data
- Populate the database with players, games, and ratings

**Note:** This may take 10-30 minutes depending on the number of tournaments. The script respects rate limits (1 request/second to CueScore).

## Step 5: Verify Database

Check that data was loaded:

```bash
psql warsaw_pool_rankings

-- Check tables
\dt

-- Check player count
SELECT COUNT(*) FROM players;

-- Check game count
SELECT COUNT(*) FROM games;

-- Check top 10 players
SELECT name, rating, games_played, confidence_level
FROM players
JOIN ratings ON players.id = ratings.player_id
ORDER BY rating DESC
LIMIT 10;

-- Exit
\q
```

## Weekly Updates

After initial setup, run weekly updates to refresh data:

```bash
cd backend
python scripts/run_weekly_update.py
```

This should be scheduled weekly (e.g., via cron):

```bash
# Add to crontab (runs every Sunday at 2am)
0 2 * * 0 cd /path/to/warsaw-pool-ranking/backend && python scripts/run_weekly_update.py
```

## Troubleshooting

### Database Connection Error
- Check PostgreSQL is running: `sudo systemctl status postgresql`
- Verify DATABASE_URL in .env file
- Check user permissions: `GRANT ALL PRIVILEGES ON DATABASE warsaw_pool_rankings TO pool_app;`

### No Data After Init
- Check logs for errors during scraping/API calls
- Verify venue IDs and slugs are correct
- Check network connectivity to cuescore.com
- Verify API endpoints are still valid

### Rate Limiting
- The script includes 1 second delay between API requests
- If you hit rate limits, the script will retry with exponential backoff
- Be patient during initial collection of large tournament history

## Database Schema

See `database/schema.sql` for the complete schema definition.

**Main tables:**
- `players` - Player information and CueScore IDs
- `venues` - Pool hall/venue information
- `tournaments` - Tournament details
- `games` - Individual game records (expanded from matches)
- `ratings` - Current player ratings (one row per player)
- `rating_snapshots` - Historical rating data (weekly snapshots)
