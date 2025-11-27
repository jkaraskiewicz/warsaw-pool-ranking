# Warsaw Pool Ranking - Rust Backend

High-performance Rust backend for the Warsaw Pool Ranking system.

## Architecture

### Modules

- **`scraper.rs`** - Web scraper for CueScore venue pages (discovers tournament IDs)
  - HTML parsing with `scraper` crate (Rust equivalent of BeautifulSoup)
  - Rate limiting (1 request/second)
  - Pagination support
  - Regex-based tournament ID extraction
- **`api.rs`** - CueScore API client for fetching tournament details
  - Async HTTP client with `reqwest`
  - Rate limiting
  - JSON parsing
- **`cache.rs`** - File-based caching layer for tournament data
  - JSON serialization
  - Save/load/exists/clear operations
- **`db.rs`** - PostgreSQL database operations using SQLx
  - Connection pooling
  - Async queries
  - Migration support
- **`models.rs`** - Core data structures (Player, Game, Tournament, Rating)
  - Serde serialization/deserialization
  - Type-safe models
- **`rating.rs`** - Bradley-Terry Maximum Likelihood rating calculator
  - MM algorithm implementation
  - Fast matrix operations with `ndarray`

### Data Flow

1. **Tournament Discovery (Web Scraping)**
   - Scrape CueScore venue pages to find tournament IDs
   - Handle pagination automatically
   - Rate-limited to 1 request/second
   - Extract IDs using regex: `/tournament/{name}/{id}`

2. **Tournament Details (API)**
   - Fetch full tournament data via CueScore API
   - Parse JSON responses
   - Extract games, players, scores, dates

3. **Caching**
   - Save scraped/fetched data to JSON files
   - Avoid re-fetching data on subsequent runs
   - Configurable cache directory

4. **Database**
   - Insert players, tournaments, games into PostgreSQL
   - Calculate time decay weights
   - Batch operations for performance

5. **Rating Calculation**
   - Load all games from database
   - Run MM algorithm (5-10 seconds)
   - Save ratings back to database

### Rating Algorithm

The rating calculator implements the **MM (Minorization-Maximization) algorithm** (Hunter, 2004) for Bradley-Terry Maximum Likelihood estimation. This algorithm is:

- **Fast**: O(n) per iteration for sparse comparison graphs
- **Stable**: Guaranteed to converge to the ML estimate
- **Scalable**: Efficient for 30K+ players and 300K+ games

Unlike Newton-based methods (which were extremely slow in the Python implementation), the MM algorithm:
- Doesn't require Hessian matrix computation
- Converges reliably for sparse comparison graphs
- Has predictable memory usage

## Setup

### Prerequisites

- Rust 1.70+ (`rustup update`)
- PostgreSQL database
- CueScore API access

### Installation

```bash
# Install dependencies
cargo build

# Set up environment variables
cp .env.example .env
# Edit .env with your database credentials

# Run database migrations (requires sqlx-cli)
cargo install sqlx-cli
sqlx database create
sqlx migrate run
```

### Running

```bash
# Development
cargo run

# Release (optimized)
cargo run --release

# Run tests
cargo test
```

## Database Schema

```sql
players (id, name, cuescore_id)
tournaments (id, name, venue_id, venue_name, start_date, end_date)
games (id, tournament_id, player1_id, player2_id, player1_score, player2_score, date, weight)
ratings (player_id, rating, games_played, confidence_level, calculated_at)
```

## Performance

Expected performance on full dataset (32K players, 342K games):

- **Data fetching**: ~3-5 minutes (rate limited)
- **Database insertion**: ~6-7 minutes
- **Rating calculation**: ~5-10 seconds (vs 20+ minutes in Python)

## Usage Example

```rust
use warsaw_pool_ranking::{scraper::VenueScraper, api::CueScoreClient, cache::Cache};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Scrape venues to discover tournament IDs
    let scraper = VenueScraper::new()?;
    let tournament_ids = scraper.scrape_venue_tournaments(
        1634568,
        "Klub Pictures",
        None  // No page limit
    ).await?;

    println!("Found {} tournaments", tournament_ids.len());

    // 2. Fetch tournament details from API
    let api_client = CueScoreClient::new();
    let mut tournaments = Vec::new();

    for tid in tournament_ids {
        let tournament = api_client.fetch_tournament_details(tid).await?;
        tournaments.push(tournament);
    }

    // 3. Cache the data
    let cache = Cache::new("./cache")?;
    cache.save("tournaments", &tournaments)?;

    // 4. Load games and calculate ratings
    // ... (database and rating calculation)

    Ok(())
}
```

## TODO

- [ ] Fill in CueScore API response parsing in `api.rs`
- [ ] Add time decay weight calculation
- [ ] Implement actual database queries in `db.rs` (currently scaffolding)
- [ ] Add CLI interface with clap
- [ ] Add configuration file support
- [ ] Implement periodic update scheduling
- [ ] Add HTTP API endpoints for serving ratings (Axum or Actix-web)
- [ ] Add comprehensive error handling
- [ ] Add integration tests
- [ ] Add benchmarks
- [ ] Verify HTML selectors match actual CueScore structure

## Development Notes

### Why Rust?

1. **Performance**: 50-100x faster than Python for numerical computation
2. **Memory safety**: No runtime errors from null pointers or data races
3. **Concurrency**: Easy parallel processing for large datasets
4. **Type safety**: Catch errors at compile time
5. **Ecosystem**: Excellent libraries (sqlx, tokio, ndarray)

### Key Differences from Python

- **Async/await**: Using Tokio for async operations
- **Error handling**: Using `Result<T, E>` instead of exceptions
- **Ownership**: Rust's ownership system prevents memory issues
- **Static typing**: All types known at compile time
