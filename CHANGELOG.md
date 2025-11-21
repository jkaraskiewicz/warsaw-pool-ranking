# Changelog

## 2025-11-20 - Initial Release

### Added
- Complete Warsaw Pool Rankings system
- Bradley-Terry ML rating algorithm with 3-year time decay
- Weekly historical simulation engine
- Angular frontend with Material Design
- FastAPI backend with RESTful API
- PostgreSQL database schema
- Docker deployment configuration
- Comprehensive test suite (48 unit tests)

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

### Testing
- 48 unit tests for rating algorithm
- Parser tests including discipline filtering
- Coverage for calculator, time decay, confidence, and parser modules
