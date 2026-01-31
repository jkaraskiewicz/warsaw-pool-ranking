# Changelog

## 2025-12-26 - Rust Rewrite

### Changed
- **Complete backend rewrite** from Python/FastAPI to Rust/Axum
- **Database migration** from PostgreSQL to SQLite (embedded, portable)
- **Architecture refactoring** following Repository Pattern and SOLID principles

### Added
- **CLI Commands**: tournaments, rankings, avatars, database, serve
- **Avatar System**: WebP encoding with hash-based change detection, multiple sizes
- **Admin Panel**: Password-protected with background refresh jobs
- **Head-to-Head Comparison**: Direct player matchup analysis
- **Rivalry Analysis**: "Nemesis" (lowest win %) and "Bunny" (highest win %) stats
- **Filter DSL**: Advanced player queries with syntax like `rating:>600 matches:>50`
- **Time Decay Weighting**: 3-year half-life (recent games weighted more)
- **Interested Players**: Bookmark players for quick access
- **Dark/Light Theme**: Persistent user preference
- **i18n Support**: English and Polish translations

### Technical Improvements
- Zero clippy warnings (`cargo clippy -- -D warnings`)
- Repository Pattern (5 repositories: Player, Game, Rating, Tournament, Avatar)
- Service Layer with specialized processors (<100 lines each)
- Batch queries preventing N+1 problems
- Configuration-driven design (AppConfig with DI)
- r2d2 connection pooling (10 connections)
- Protobuf for API type definitions

### API Endpoints
- `GET /api/players` - Paginated rankings with filtering, sorting
- `GET /api/player/{id}` - Player details with stats
- `GET /api/compare/{id1}/{id2}` - Head-to-head comparison
- `GET /api/player/{id}/rivalries` - Nemesis & Bunny analysis
- `GET /api/avatars/{id}/{size}` - WebP avatar images
- `POST /api/admin/login` - Authentication
- `POST /api/admin/refresh` - Trigger data sync (async background)
- `POST /api/admin/refresh-avatars` - Trigger avatar refresh

### Frontend Improvements
- Angular 17 standalone components (no NgModules)
- Signal inputs (`input<T>()`) with OnPush change detection
- Modern control flow (`@if`, `@for`, `@switch`)
- Skeleton loading placeholders
- Reusable AvatarComponent and RatingTypeSelectorComponent

---

## 2025-11-20 - Initial Release (Python - Superseded)

> **Note:** This version was completely replaced by the Rust rewrite above.

### Added
- Complete Warsaw Pool Rankings system
- Bradley-Terry ML rating algorithm with 3-year time decay
- Weekly historical simulation engine
- Angular frontend with Material Design
- FastAPI backend with RESTful API
- PostgreSQL database schema
- Docker deployment configuration

### Venues Configured (9 Warsaw venues)
1. 147 Break Zamieniecka
2. 147 Break Fort Wola
3. 147 Break Nowogrodzka
4. Shooters
5. Eighty Nine
6. Złota Bila - Centrum Bilardowe
7. Billboard Pool & Snooker
8. Klub Pictures
9. The Lounge - Billiards Club

### Features
- **Discipline Filtering**: Automatically excludes snooker and pyramid tournaments
  - Filters: snooker, pyramid, piramida, russian pyramid, russian pool
  - Only includes pool tournaments (8-ball, 9-ball, 10-ball, etc.)

- **Rating System**:
  - 100 points = 2:1 winning odds
  - Exponential time decay (3-year half-life)
  - New player blending (100 games threshold)
  - Confidence levels: Unranked, Provisional, Emerging, Established

- **Data Collection**:
  - CueScore API integration
  - Venue page scraping with pagination
  - Rate limiting (1 req/sec)
  - Exponential backoff on failures

- **Frontend**:
  - Searchable player rankings table
  - Player detail overlay with stats
  - Rating history chart (Chart.js)
  - CueScore profile links

- **Backend**:
  - GET /api/players - List ranked players
  - GET /api/player/:id - Player details
  - GET /api/player/:id/history - Rating history

### Documentation
- README.md - Quick start guide
- DOCKER_SETUP.md - Complete Docker documentation
- DATABASE_SETUP.md - Manual database setup
- DESIGN.md - System architecture and design decisions
- Makefile - Convenient commands for Docker operations
