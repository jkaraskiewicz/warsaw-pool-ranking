# Backend Data Directory

This directory contains the **SQLite database** used by the application.

## Purpose

- **Production Database**: Stores all application data (players, tournaments, games, ratings, avatars)
- **Tracked in Git**: Intentionally committed to repository for easy deployment and portability
- **Shared Between Environments**: Same database used locally and in Docker

## Files

### `warsaw_pool_ranking.db`

**Size**: ~50 MB
**Purpose**: Main SQLite database containing all application data
**Schema**: Defined in `backend/src/database/schema.sql`

**Tables**:
- `players` - Player information and metadata
- `tournaments` - Tournament information from CueScore
- `games` - Individual game results
- `ratings` - Calculated ratings (all periods: active, all-time, 1y, 2y)
- `avatars` - Player avatar images (WebP format, small/medium/large sizes)

## Configuration

The database path is configured in `backend/src/config/paths.rs`:

### Local Development
```rust
// Auto-detected path when running from project root
"backend/data/warsaw_pool_ranking.db"
```

### Docker
```yaml
# docker-compose.yml sets environment variable:
DATABASE_PATH: /app/data/warsaw_pool_ranking.db

# Mounted as named volume:
volumes:
  - app_data:/app/data
```

## Important Notes

### Why This Database is Tracked in Git

Unlike typical applications where databases are NOT committed, this project **intentionally tracks the database** for these reasons:

1. **Data Portability**: Clone the repo and have all data immediately available
2. **No External API**: Data is scraped from CueScore (rate-limited, time-consuming)
3. **Historical Data**: Preserves historical tournament and rating data
4. **Quick Setup**: New contributors don't need to run ingestion process

### Database Updates

The database is updated by:
1. **Manual CLI**: `cargo run -- ingest && cargo run -- process`
2. **Admin Panel**: Triggers ingestion/processing via API
3. **Auto-refresh**: On container startup (see `backend/entrypoint.sh`)

### Backup Strategy

Since the database is in git, every commit serves as a backup. However:
- Large file size (~50 MB) increases repo clone time
- Consider git-lfs if database grows significantly (>100 MB)

## Directory Structure

```
backend/data/
├── README.md                     # This file
└── warsaw_pool_ranking.db        # Main database (tracked in git)
```

## See Also

- `backend/src/config/paths.rs` - Database path resolution logic
- `backend/src/database/schema.sql` - Database schema definition
- `docker-compose.yml` - Docker volume configuration
- `CLAUDE.md` - Project architecture documentation
