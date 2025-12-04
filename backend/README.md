# Warsaw Pool Ranking - Rust Backend

Data acquisition backend for the Warsaw Pool Ranking system, written in Rust.

## Features

- **Web Scraping**: Discovers tournaments from CueScore venue pages
- **API Client**: Fetches tournament details from CueScore API
- **Two-Tier Caching**: Raw API responses and parsed tournament data
- **Rate Limiting**: Automatic rate limiting (1 req/sec) for API and scraping
- **Progress Tracking**: Human-readable progress indicators

## Architecture

### Modules

- `api/` - CueScore API client with response parsing
- `cache/` - Two-tier caching system (raw + parsed)
- `config/` - Venue configuration
- `domain/` - Data models and collections
- `errors/` - Error handling utilities
- `fetchers/` - Web scraping for tournament discovery
- `http/` - HTTP client with rate limiting
- `pagination/` - Pagination abstractions
- `rate_limiter/` - Rate limiting logic

## Usage

### Configuration

Update venue IDs in `src/config/venues.rs`:

```rust
pub fn get_venues() -> Vec<VenueConfig> {
    vec![
        VenueConfig::new(12345, "your-venue-name"),
        // Add more venues...
    ]
}
```

To find venue IDs:
1. Navigate to https://cuescore.com
2. Search for your venue
3. URL format: `https://cuescore.com/venue/{name}/{id}/`
4. Extract the ID

### Run Data Ingestion

```bash
# Set log level (optional)
export RUST_LOG=info

# Run ingestion
cargo run -- ingest
```

### Output

Data is saved to `cache/` directory:
- `cache/raw/{tournament_id}.json` - Raw API responses
- `cache/parsed/tournaments.json` - Processed tournament data

## Development

```bash
# Check compilation
cargo check

# Run with logs
RUST_LOG=debug cargo run -- ingest

# Build release version
cargo build --release
```

## Architecture Principles

- **Small Files**: Each file focused on one responsibility
- **Short Methods**: Functions kept to 5-15 lines
- **Multiple Abstraction Layers**: Rate limiting, HTTP, pagination, parsing
- **Composability**: Layers compose cleanly for testability
