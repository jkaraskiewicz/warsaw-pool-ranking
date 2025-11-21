# Warsaw Pool Rankings

A skill-based rating system for Warsaw pool players using the Bradley-Terry Maximum Likelihood model with time decay. Data collected from CueScore.

## Features

- **Bradley-Terry ML Rating System** - 100 points = 2:1 winning odds
- **3-Year Exponential Time Decay** - Recent games weighted higher
- **Weekly Historical Replay** - Full simulation from scratch each week
- **Confidence Levels** - Unranked, Provisional, Emerging, Established
- **Angular Frontend** - Searchable player list with rating history charts
- **FastAPI Backend** - RESTful API with automatic OpenAPI docs
- **PostgreSQL Database** - Persistent storage with weekly snapshots

## Quick Start with Docker (Recommended)

### 1. Configure Venues

Edit `backend/config/venues.py` to add Warsaw pool venues:

```python
WARSAW_VENUES = [
    {
        "id": "67496954",
        "slug": "147-break-nowogrodzka",
        "name": "147 Break Nowogrodzka"
    },
]
```

### 2. Start Services

```bash
# Build and start all services (PostgreSQL, Backend, Frontend)
docker-compose up -d

# Initialize database with historical data (10-30 minutes)
docker-compose exec backend python scripts/init_database.py --auto
```

### 3. Access Application

- **Frontend:** http://localhost
- **Backend API:** http://localhost:8000
- **API Docs:** http://localhost:8000/docs

### 4. Weekly Updates

```bash
# Run manually
docker-compose exec backend python scripts/run_weekly_update.py

# Or schedule with cron
0 2 * * 0 docker-compose exec -T backend python scripts/run_weekly_update.py
```

## Using Makefile (Optional)

```bash
make up        # Start all services
make init      # Initialize database
make logs      # View logs
make players   # Show top 10 players
make stats     # Show database statistics
make update    # Run weekly update
make down      # Stop all services
```

See `make help` for all commands.

## Architecture

```
┌──────────────────────────────────────────────────┐
│  Angular Frontend (Port 80)                      │
│  - Player rankings table with search             │
│  - Player detail overlay                         │
│  - Rating history chart (Chart.js)               │
└────────────┬─────────────────────────────────────┘
             │
             │ HTTP /api/*
             ▼
┌──────────────────────────────────────────────────┐
│  FastAPI Backend (Port 8000)                     │
│  - GET /api/players                              │
│  - GET /api/player/:id                           │
│  - GET /api/player/:id/history                   │
└────────────┬─────────────────────────────────────┘
             │
             │ SQL
             ▼
┌──────────────────────────────────────────────────┐
│  PostgreSQL Database (Port 5432)                 │
│  - Players, Games, Ratings, Snapshots            │
│  - Persistent volume: postgres_data              │
└──────────────────────────────────────────────────┘
```

## Rating Algorithm

### Bradley-Terry Maximum Likelihood
- Uses `choix` library for efficient ML estimation
- Scale: 100 points = 2:1 winning odds (200 pts = 4:1 odds)
- Game-level granularity (match scores expanded to individual games)

### Time Decay
- Exponential decay with 3-year half-life
- Formula: `weight = exp(-λ × days_ago)` where `λ = ln(2) / 1095`
- Applied during rating calculation, not stored

### New Player Blending
- 0-9 games: Unranked (100% starter rating of 500)
- 10-49 games: Provisional (blend starter + ML)
- 50-99 games: Emerging (mostly ML rating)
- 100+ games: Established (pure ML rating)

### Weekly Simulation
- Replays entire game history week-by-week
- Generates consistent snapshots for history charts
- Allows algorithm changes to update all historical ratings

## Project Structure

```
warsaw-pool-ranking/
├── backend/
│   ├── app/
│   │   ├── api/              # FastAPI endpoints
│   │   ├── data/             # CueScore scraper & API client
│   │   ├── rating/           # Rating algorithm
│   │   └── database.py       # SQLAlchemy setup
│   ├── config/
│   │   └── venues.py         # Venue configuration
│   ├── scripts/
│   │   ├── init_database.py  # Initial setup
│   │   └── run_weekly_update.py
│   └── tests/                # Unit tests (pytest)
├── frontend/
│   └── src/app/
│       ├── components/       # Angular components
│       ├── services/         # HTTP services
│       └── models/           # TypeScript interfaces
├── database/
│   └── schema.sql            # PostgreSQL schema
├── docker-compose.yml
└── DESIGN.md                 # Detailed design document
```

## Development

### Without Docker

See [DATABASE_SETUP.md](DATABASE_SETUP.md) for manual setup instructions.

### Running Tests

```bash
# With Docker
docker-compose exec backend pytest -v

# Without Docker
cd backend
pytest -v
```

### Backend Development

```bash
# Docker (with hot reload)
docker-compose up -d

# Local
cd backend
uvicorn app.main:app --reload
```

### Frontend Development

```bash
# Local (recommended for development)
cd frontend
npm install
npm start  # http://localhost:4200

# Docker (production build)
docker-compose up frontend
```

## Data Collection

### CueScore Integration

1. **Venue Scraping** - Scrapes tournament lists from venue pages
2. **API Fetching** - Fetches tournament details via API
3. **Game Parsing** - Converts match scores to individual games
4. **Rate Limiting** - 1 request/second with exponential backoff

### Adding Venues

Find venue on CueScore, extract from URL:
```
https://cuescore.com/venue/{slug}/{id}/tournaments
```

Add to `backend/config/venues.py`:
```python
{
    "id": "venue_id",
    "slug": "venue-slug",
    "name": "Display Name"
}
```

## Documentation

- **[DESIGN.md](DESIGN.md)** - Complete system design and architecture
- **[DOCKER_SETUP.md](DOCKER_SETUP.md)** - Docker deployment guide
- **[DATABASE_SETUP.md](DATABASE_SETUP.md)** - Manual database setup
- **[backend/tests/README.md](backend/tests/README.md)** - Testing guide

## API Documentation

Interactive API docs available at http://localhost:8000/docs when running.

### Endpoints

- `GET /api/players?min_games=10` - List ranked players
- `GET /api/player/{id}` - Player details
- `GET /api/player/{id}/history` - Rating history snapshots

## Technology Stack

- **Backend:** Python 3.11, FastAPI, SQLAlchemy, choix, pandas, BeautifulSoup4
- **Frontend:** Angular 17, TypeScript, Angular Material, Chart.js (ng2-charts)
- **Database:** PostgreSQL 15
- **Deployment:** Docker, docker-compose, Nginx

## License

MIT

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `make test`
5. Submit a pull request

## Support

For issues and questions, please use the GitHub issue tracker.
