# Warsaw Pool Rankings

A skill-based rating system for Warsaw pool players using the Bradley-Terry Maximum Likelihood model with time decay. Data collected from CueScore.

## Features

- **Bradley-Terry ML Rating System** - 100 points = 2:1 winning odds
- **3-Year Exponential Time Decay** - Recent games weighted higher
- **Weekly Historical Replay** - Full simulation from scratch each week
- **Confidence Levels** - Unranked, Provisional, Emerging, Established
- **Angular Frontend** - Searchable player list with rating history charts
- **High-Performance Rust Backend** - Efficient data processing and rating calculation
- **SQLite Database** - Persistent storage with weekly snapshots

## Quick Start with Docker (Recommended)

### 1. Configure Venues

Edit `backend/src/config/venues.rs` to add Warsaw pool venues. This is a Rust file now, so the syntax will be Rust code:

```rust
pub fn get_venues() -> Vec<VenueConfig> {
    vec![
        VenueConfig { id: 2842336, name: "147 Break Zamieniecka".to_string(), slug: "147-break-zamieniecka".to_string() },
        VenueConfig { id: 1698108, name: "147 Break Nowogrodzka".to_string(), slug: "147-break-nowogrodzka".to_string() },
    ]
}
```

### 2. Start Services

```bash
# Build and start all services (PostgreSQL, Backend, Frontend)
docker-compose up -d --build

# The Rust backend will automatically run `process` on startup.
# To manually trigger ingest (e.g., for new data):
# docker-compose exec backend ./warsaw_pool_ranking ingest
```

### 3. Access Application

- **Frontend:** http://localhost
- **Backend (Rust CLI):** Run `docker-compose logs backend` to see processing output.

## Using Makefile (Optional)

```bash
make up        # Start all services
make init      # The Rust backend initializes on start, this command is deprecated.
make logs      # View logs
make players   # Show top 10 players (requires custom Docker exec command for Rust)
make stats     # Show database statistics (requires custom Docker exec command for Rust)
make update    # The Rust backend processes on start, this command is deprecated.
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
             │ (Accesses SQLite DB directly in backend container)
             ▼
┌──────────────────────────────────────────────────┐
│  Rust Backend (CLI - processes on startup)       │
│  - Scrapes CueScore data                         │
│  - Calculates Bradley-Terry ratings              │
│  - Stores data in SQLite                         │
└────────────┬─────────────────────────────────────┘
             │
             │ (Embedded within backend container)
             ▼
┌──────────────────────────────────────────────────┐
│  SQLite Database (File: warsaw_pool_ranking.db)  │
│  - Players, Games, Ratings, Snapshots            │
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
├── backend/                  # Rust backend
│   ├── src/                  # Rust source code
│   │   ├── api/              # CueScore API client
│   │   ├── cache/            # Caching logic
│   │   ├── cli/              # Command-line interface
│   │   ├── config/           # Application configuration
│   │   ├── database/         # Database interaction
│   │   ├── domain/           # Core domain models and logic
│   │   ├── fetchers/         # Web scraping logic
│   │   ├── http/             # HTTP client with rate limiting
│   │   ├── pagination/       # Pagination logic
│   │   ├── rate_limiter/     # Rate limiting implementation
│   │   ├── rating/           # Bradley-Terry rating algorithm
│   │   └── services/         # High-level business logic (ingestion, processing)
│   ├── Cargo.toml            # Rust project manifest
│   └── Dockerfile            # Dockerfile for the Rust backend
├── frontend/
│   └── src/app/
│       ├── components/       # Angular components
│       ├── services/         # HTTP services
│       └── models/           # TypeScript interfaces
├── database/
│   └── schema.sql            # SQLite schema
├── docker-compose.yml
└── README.md
```

## Development

### Without Docker

To run the Rust backend locally:

```bash
cd backend
# First, build the project
cargo build --release

# Then, run the processing step. It will ingest data if not cached.
./target/release/warsaw_pool_ranking process

# To ingest new data (and recache):
./target/release/warsaw_pool_ranking ingest
```

### Running Tests

```bash
# With Docker
docker-compose exec backend cargo test

# Without Docker
cd backend
cargo test
```

### Backend Development

```bash
# Docker (builds and runs)
docker-compose up -d backend

# Local (with hot reload via `cargo watch` or manual recompilation)
cd backend
cargo watch -x run -- process # Runs `process` on file changes
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

1.  **Venue Scraping** - Scrapes tournament lists from venue pages
2.  **API Fetching** - Fetches tournament details via API
3.  **Game Parsing** - Converts match scores to individual games
4.  **Rate Limiting** - 10 requests/second with caching and graceful error handling

### Adding Venues

Find venue on CueScore, extract from URL:
```
https://cuescore.com/venue/{slug}/{id}/tournaments
```

Add to `backend/src/config/venues.rs`:
```rust
    VenueConfig { id: 12345, name: "New Venue Name".to_string(), slug: "new-venue-name".to_string() },
```

## Documentation

- **[backend/tests/README.md](backend/tests/README.md)** - Testing guide

## Technology Stack

- **Backend:** Rust (1.73+), `tokio`, `reqwest`, `ndarray`, `rusqlite`, `serde`
- **Frontend:** Angular 17, TypeScript, Angular Material, Chart.js (ng2-charts)
- **Database:** SQLite (embedded within backend container)
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
