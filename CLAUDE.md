# Warsaw Pool Rankings - LLM Context Documentation

**Last Updated:** 2025-12-25
**Purpose:** Essential context for Claude Code and other LLM sessions working on this project.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Code Quality Standards](#2-code-quality-standards)
3. [Architecture Overview](#3-architecture-overview)
4. [Technology Stack](#4-technology-stack)
5. [Backend Architecture](#5-backend-architecture)
6. [Frontend Architecture](#6-frontend-architecture)
7. [Database Schema](#7-database-schema)
8. [Rating Algorithm](#8-rating-algorithm)
9. [Data Collection](#9-data-collection)
10. [Authentication & Security](#10-authentication--security)
11. [Avatar Storage](#11-avatar-storage)
12. [Development Workflows](#12-development-workflows)
13. [Common Patterns & Conventions](#13-common-patterns--conventions)
14. [File Organization & Git Strategy](#14-file-organization--git-strategy)
15. [Key Design Decisions](#15-key-design-decisions)
16. [File Paths Reference](#16-file-paths-reference)
17. [Testing](#17-testing)
18. [Recent Refactoring (Completed)](#18-recent-refactoring-completed)
19. [Future Enhancements](#19-future-enhancements)

---

## 1. Project Overview

### Purpose
Warsaw Pool Rankings is a skill-based rating system for pool players in Warsaw, Poland. It collects game data from various pool venues and calculates player ratings using a sophisticated statistical model.

### Key Features
- **Bradley-Terry ML Rating System**: 100 points = 2:1 winning odds
- **Time Decay**: Exponential decay with 3-year half-life (recent games weighted more)
- **Active Ranking**: Filtered ranking for players active in the last 6 months (default view)
- **Rivalry Analysis**: "Nemesis" (lowest win %) and "Bunny" (highest win %) statistics
- **Multiple Rating Periods**: Active, All-time, 1y, 2y
- **Confidence Levels**: Unranked, Provisional, Emerging, Established (based on games played)
- **Local Avatar Storage**: WebP format with hash-based change detection
- **Admin Panel**: Password-protected data refresh functionality

### Data Source
All data is collected from [CueScore](https://cuescore.com), a tournament management platform used by Warsaw pool venues.

### Target Users
- Pool players in Warsaw who want to track their skill progression
- Tournament organizers looking for fair seeding
- Anyone interested in Warsaw's competitive pool scene

---

## 2. Code Quality Standards

> **CRITICAL**: This project adheres to the **highest coding standards**. Every change must improve code quality. We follow industry best practices, modern idioms, and rigorous design principles.

### Core Principles

#### 1. **Idiomatic, Modern Code**

**Rust**:
- Use Rust 2021+ edition idioms
- **Repository Pattern**: ALL database access MUST go through repositories (PlayerRepository, GameRepository, TournamentRepository, RatingRepository, AvatarRepository). Direct SQL in services/handlers is prohibited.
- **Service Layer**: Business logic lives in `PlayerService` and other services.
- **Error Handling**: Use `AppError` enum for centralized error handling.
- **Performance**: Use batch queries to avoid N+1 problems (see GameRepository::count_matches_for_players)
- Prefer `?` operator over manual error handling (never use `unwrap_or()` to hide errors)
- Use `impl Trait` for return types where appropriate
- Leverage type system for compile-time guarantees
- Modern async/await (not manual futures)
- Use `cargo clippy` and address ALL warnings
- Format with `cargo fmt`
- Delete dead code instead of suppressing warnings with `#[allow(dead_code)]`

**Angular**:
- Angular 17+ features exclusively
- Standalone components (NO NgModules)
- **ChangeDetectionStrategy.OnPush**: Mandatory for all components.
- **Signals**: Use Signals for local state and `input.required<T>()` for inputs. Avoid `ngOnChanges`.
- **Modern Control Flow**: `@if`, `@for`, `@switch` (NO `*ngIf`, `*ngFor`)
- Typed forms, typed routes
- Use `ng lint` and fix ALL issues
- Strict TypeScript mode enabled

#### 2. **Keep Dependencies Updated**

- **Monthly dependency review** (minimum)
- Use latest stable versions of core libraries
- Security updates applied immediately
- Document why any dependency is pinned to older version
- Run `cargo update` and `npm update` regularly
- Monitor Dependabot alerts

**Example**:
```bash
# Backend
cd backend
cargo update
cargo audit

# Frontend
cd frontend
npm update
npm audit fix
```

#### 3. **SOLID Principles**

##### Single Responsibility Principle (SRP)
- **One function = one responsibility**
- **One file = one major concept**
- Example: `calculate_rating()` only calculates, doesn't fetch or store

##### Open/Closed Principle (OCP)
- Open for extension, closed for modification
- Use traits (Rust) and interfaces (TypeScript)

##### Liskov Substitution Principle (LSP)
- Subtypes must be substitutable for base types
- Don't violate contracts in implementations

##### Interface Segregation Principle (ISP)
- Small, focused traits/interfaces
- Clients shouldn't depend on methods they don't use

##### Dependency Inversion Principle (DIP)
- Depend on abstractions, not concretions
- Use dependency injection (AppState pattern)

#### 4. **DRY (Don't Repeat Yourself)**

**Aggressive Refactoring**:
- **3 uses = extract to function**
- **2 similar functions = extract common logic**
- No copy-paste code allowed
- Use generics and traits for code reuse

#### 5. **Small Functions & Files**

**Function Guidelines**:
- **Max 30 lines** (excluding comments)
- **Max 4 parameters** (use struct if more needed)
- **One level of abstraction** per function
- If function has "and" in description, split it

**File Guidelines**:
- **Max 300 lines** per file (excluding tests)
- **One major concept** per file
- Split large files into modules

#### 6. **Design Patterns**

Use proven patterns where appropriate:

**Rust**:
- **Repository Pattern**: For database access abstraction
- **Service Pattern**: For business logic encapsulation
- **Builder Pattern**: For complex configuration
- **Strategy Pattern**: For interchangeable algorithms (rating strategies)
- **Dependency Injection**: Via `Arc<AppState>`

**Angular**:
- **Service Pattern**: For business logic and state
- **Signal Store / Facade Pattern**: Simplify complex subsystems
- **Singleton Services**: `providedIn: 'root'`

#### 7. **Boy Scout Rule**

> **"Always leave the codebase cleaner than you found it."**

---

## 3. Architecture Overview

### High-Level System Design

```
┌─────────────────────────────────────────────────┐
│  Angular Frontend (Port 80)                     │
│  - Player rankings table with search            │
│  - Player detail overlay with history chart     │
│  - Admin panel for data refresh                 │
│  - Signals & OnPush Change Detection            │
└────────────┬────────────────────────────────────┘
             │
             │ HTTP GET /api/* (reads from SQLite)
             │ HTTP POST /api/admin/* (triggers processing)
             │
             ▼
┌─────────────────────────────────────────────────┐
│  Rust Backend (Port 8000)                       │
│  - Axum HTTP server for API endpoints           │
│  - Service Layer & Repository Pattern           │
│  - CLI commands: serve, ingest, process, refresh│
│  - Bradley-Terry rating calculation             │
│  - CueScore data scraping/fetching              │
└────────────┬────────────────────────────────────┘
             │
             │ rusqlite (r2d2 connection pool)
             │
             ▼
┌─────────────────────────────────────────────────┐
│  SQLite Database (warsaw_pool_ranking.db)       │
│  - players, tournaments, games                  │
│  - ratings (multiple periods)                   │
│  - avatars (local WebP storage)                 │
└─────────────────────────────────────────────────┘
```

### Data Flow

1. **Data Collection**: Backend scrapes CueScore venue pages and fetches tournament data via API
2. **Processing**: Backend calculates ratings using Bradley-Terry ML algorithm
3. **Storage**: All data stored in SQLite database
4. **Serving**: Backend exposes REST API endpoints via Service/Repository layers
5. **Display**: Frontend queries API and displays rankings using Signals

### Deployment Model

Docker Compose with 2 services:
- **frontend**: Nginx serving Angular SPA on port 80
- **backend**: Rust HTTP server on port 8000, shared SQLite volume

---

## 4. Technology Stack

### Backend: Rust 1.73+ (Edition 2024)

**Core Libraries:**
- `tokio` (1.48.0) - Async runtime with full features
- `axum` (0.7.5) - Web framework for HTTP server
- `rusqlite` (0.32.1) - SQLite database driver with bundled SQLite
- `r2d2` / `r2d2_sqlite` - Connection pooling
- `reqwest` (0.12.24) - HTTP client for API calls
- `serde` (1.0.228) - Serialization/deserialization
- `anyhow` (1.0.100) - Error handling
- `thiserror` (1.0.61) - Custom error types (`AppError`)
- `nalgebra` (0.34.1) - Linear algebra for rating calculations
- `clap` (4.5.53) - CLI argument parsing

**Additional Libraries:**
- `scraper` (0.24.0) - HTML parsing for web scraping
- `image` / `webp` - Avatar image processing
- `chrono` (0.4.42) - Date/time handling
- `log` / `sensible-env-logger` - Logging
- `prost` (0.13) - Protocol Buffers (for API models)

**Why Rust?**
- **Performance**: Rating calculation on large datasets is CPU-intensive
- **Type Safety**: Prevents entire classes of bugs at compile time
- **Memory Safety**: No garbage collection overhead, no null pointer dereferences
- **Ecosystem**: Excellent libraries for web, database, and numerical computing
- **Concurrency**: Fearless concurrency with ownership system

### Frontend: Angular 17

**Core Technologies:**
- **Angular 17**: Modern framework with standalone components
- **TypeScript 5.2**: Type safety for frontend code
- **Angular Material 17**: UI component library (Material Design)
- **Chart.js 4.4**: Rating history visualization
- **ng2-charts 5.0**: Angular wrapper for Chart.js
- **RxJS 7.8**: Reactive programming for async operations

**Why Angular 17?**
- **Standalone Components**: No NgModules, simpler architecture
- **Modern Control Flow**: `@if`, `@for`, `@switch`
- **Signals**: Better reactivity and performance
- **Material Design**: Professional UI out of the box
- **Type Safety**: TypeScript integration prevents runtime errors

### Database: SQLite (Bundled)

**Configuration:**
- Embedded in backend container (no separate server)
- File: `warsaw_pool_ranking.db`
- Connection pooling: r2d2 with 10 max connections

**Why SQLite?**
- **Simplicity**: No separate database server to manage
- **Read-Heavy Workload**: Perfect for this use case (ratings calculated periodically)
- **Portability**: Single file for backup/restore
- **Performance**: Sufficient for tens of thousands of games
- **Embedded**: Deployed in backend container's volume

### Infrastructure

- **Docker**: Containerization for consistent deployment
- **docker-compose**: Multi-service orchestration
- **Nginx**: Static file serving for Angular SPA
- **Volume**: Persistent storage for SQLite database

---

## 5. Backend Architecture

### Project Structure

```
backend/
├── src/
│   ├── api/                     # CueScore API client & HTTP handlers
│   │   ├── dtos/                # API Data Transfer Objects
│   │   ├── filter/              # Filter DSL Parser
│   │   ├── handlers/            # HTTP request handlers
│   │   │   ├── admin.rs         # Admin panel endpoints
│   │   │   ├── avatars.rs       # Avatar serving endpoints
│   │   │   └── players.rs       # Player data endpoints
│   │   ├── models.rs            # Generated Protobuf models
│   │   └── routes.rs            # Axum route definitions
│   ├── cache/                   # Caching layer for API responses
│   │   └── mod.rs
│   ├── cli/                     # Command-line interface
│   │   └── mod.rs               # clap command definitions
│   ├── config/                  # Configuration management
│   │   ├── settings.rs          # AppConfig, RatingSettings, etc.
│   │   └── venues.rs            # Venue configuration
│   ├── database/                # SQLite interaction (Repository Pattern)
│   │   ├── mod.rs               # Database connection & initialization
│   │   ├── schema.sql           # SQL schema definition
│   │   ├── repositories/        # Data access layer (ONLY place for SQL)
│   │   │   ├── mod.rs
│   │   │   ├── avatar_repository.rs   # Avatar CRUD operations
│   │   │   ├── game_repository.rs     # Game CRUD + batch queries
│   │   │   ├── player_repository.rs   # Player CRUD operations
│   │   │   ├── rating_repository.rs   # Rating CRUD operations
│   │   │   └── tournament_repository.rs # Tournament CRUD operations
│   │   ├── connection.rs        # Connection pool management
│   │   ├── models.rs            # Database Entity models
│   │   ├── setup.rs             # Schema initialization
│   │   └── structs.rs           # Helper structures
│   ├── domain/                  # Core domain models
│   │   ├── models.rs            # Shared domain structures
│   │   └── ...
│   ├── fetchers/                # Web scraping logic
│   │   ├── cuescore_models.rs   # External API models
│   │   └── venue_scraper.rs     # Venue scraping
│   ├── errors/                  # Centralized Error Handling
│   │   └── mod.rs               # AppError definition
│   ├── services/                # High-level business logic
│   │   ├── player_service.rs    # Rating calculation logic
│   │   ├── ingestion.rs         # Data collection orchestration
│   │   ├── processing.rs        # Rating calculation orchestration
│   │   └── avatar_processor.rs  # Avatar processing logic
├── Cargo.toml                   # Rust dependencies
├── build.rs                     # Build script (protobuf compilation)
└── Dockerfile                   # Docker image definition
```

### Key Design Patterns

#### 1. Repository Pattern (Strictly Enforced)

All raw SQL queries are encapsulated in `repositories/`. **NO SQL is allowed in handlers or services**.

**Available Repositories:**
- `PlayerRepository` - Player CRUD, ranked lists, last played updates
- `GameRepository` - Game CRUD, H2H matches, batch queries
- `TournamentRepository` - Tournament CRUD
- `RatingRepository` - Rating CRUD
- `AvatarRepository` - Avatar storage and retrieval

```rust
// backend/src/database/repositories/player_repository.rs
pub struct PlayerRepository;
impl PlayerRepository {
    pub fn list_ranked_players(...) -> Result<Vec<PlayerWithRating>> {
        // SQL query execution
    }

    pub fn upsert_player(...) -> Result<Player> { /* ... */ }
    pub fn find_by_id(...) -> Result<Option<Player>> { /* ... */ }
    pub fn list_all(...) -> Result<Vec<Player>> { /* ... */ }
}
```

**Performance Pattern - Batch Queries:**

To avoid N+1 query problems, use batch query methods that return `HashMap` for O(1) lookups:

```rust
// backend/src/database/repositories/game_repository.rs
pub fn count_matches_for_players(
    conn: &mut DbConn,
    player_ids: &[i32],
) -> Result<HashMap<i32, i32>> {
    // Single query using UNION ALL + GROUP BY
    // Returns HashMap for fast lookup
}

// Usage in handler:
let player_ids: Vec<i32> = rows.iter().map(|r| r.player_id).collect();
let match_counts = GameRepository::count_matches_for_players(&mut conn, &player_ids)?;
let matches = match_counts.get(&player_id).copied().unwrap_or(0);
```

**Why**:
- Decouples API handlers from database details
- Makes testing easier (can mock repositories)
- Single source of truth for queries (DRY)
- Enforces performance patterns (batch queries)

#### 2. Service Layer

Business logic resides in `services/`:

```rust
// backend/src/services/player_service.rs
pub fn calculate_adjusted_ratings(...) -> (f64, f64, f64) {
    // Pure logic
}
```

**Why**: Removes duplication between different handlers (e.g. detailed view vs comparison view).

#### 3. Centralized Error Handling

`AppError` (using `thiserror`) maps internal errors to HTTP responses:

```rust
// backend/src/errors/mod.rs
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    // ...
}
```

**Why**: Consistent API error responses (JSON), simplified handler logic with `?`.

### CLI Commands

#### `refresh-avatars`
- Iterates through all players in the database
- Downloads avatars from CueScore if missing or changed
- Processes and stores as WebP in `avatars` table
- Automatically run during container initialization

---

## 6. Frontend Architecture

### Angular 17 Patterns

#### Signals & OnPush

All components use **Angular Signals** for local state and `ChangeDetectionStrategy.OnPush` for performance.

```typescript
@Component({
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PlayerOverlayComponent {
  player = signal<PlayerDetail | null>(null);
  loading = signal<boolean>(true);
  
  // Input as signal
  matches = input.required<HeadToHeadMatch[]>();
}
```

**Why**: Fine-grained reactivity, reduced change detection cycles, better performance.

#### Modern Control Flow

Templates use the new built-in control flow syntax:

```html
@if (loading()) {
  <app-skeleton />
} @else {
  @for (match of matches(); track match.date) {
    <div>{{ match.opponentName }}</div>
  }
}
```

### Key Services

#### PlayerService (`player.service.ts`)
- Fetches player lists, details, and comparisons
- Fetches rivalry data (`/api/player/:id/rivalries`)

### Components

#### RatingTypeSelectorComponent
- **Shared Component**: Reusable toggle group for selecting ranking category
- **Features**: Distinctive styling for "Active" category

#### SkeletonComponent
- **Shared Component**: Pulsing placeholder for loading states
- **Usage**: `PlayerOverlay`, `PlayerList`, `Comparison`

#### MatchHistoryComponent & HeadToHeadStatsComponent
- **Comparison Views**: Dedicated components for displaying H2H data

---

## 7. Database Schema

### `players` Table

```sql
CREATE TABLE players (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cuescore_id INTEGER UNIQUE,
    name TEXT NOT NULL,
    avatar_url TEXT,
    last_played TEXT, -- Track last activity for "Active" filter
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Performance index for active player queries
CREATE INDEX idx_players_last_played ON players(last_played);
```

**Key Points:**
- `last_played` column tracks most recent game date for "Active" filter
- Index on `last_played` enables O(log n) filtering instead of full table scans
- Used in `PlayerRepository::list_ranked_players()` with `WHERE p.last_played >= ?`

### Other Tables

See `backend/src/database/schema.sql` for complete schema:
- `tournaments` - Tournament metadata
- `games` - Individual game results with weights
- `ratings` - Calculated ratings per player per period
- `avatars` - WebP images (small/medium/large) with hash-based change detection

---

## 12. Development Workflows

### Protobuf Generation

The project relies on Protocol Buffers to sync API models between Rust and TypeScript.

**If you modify `proto/api.proto`**:
1. Run:
   ```bash
   cd frontend
   npm run proto:generate:local
   ```
2. This regenerates `frontend/src/app/models/api.ts`
3. Backend models are regenerated automatically during `cargo build` via `build.rs`

### Backend Build Verification

Since `prost-build` requires `protoc` installed on the system, local `cargo check` might fail if `protoc` is missing.

**To verify backend build correctly**:
```bash
docker-compose build backend
```
This runs the build inside the Docker container where `protobuf-compiler` is installed.

---

## 13. Common Patterns & Conventions

### Repository Pattern Usage

**✅ CORRECT - Use repositories in services:**
```rust
// backend/src/services/processing.rs
use crate::database::repositories::player_repository::PlayerRepository;

let players = PlayerRepository::list_all(&mut conn)?;
PlayerRepository::update_player_last_played(&mut conn, player_id, date)?;
```

**❌ INCORRECT - Direct SQL in services:**
```rust
// NEVER DO THIS
conn.execute("UPDATE players SET last_played = ?1 WHERE id = ?2", params![date, id])?;
```

### Avoiding N+1 Query Problems

**✅ CORRECT - Batch query with HashMap:**
```rust
// Collect IDs first
let player_ids: Vec<i32> = rows.iter().map(|r| r.player_id).collect();

// Single batch query
let match_counts = GameRepository::count_matches_for_players(&mut conn, &player_ids)?;

// Map without mutating connection
let players = rows.into_iter().map(|(i, row)| {
    let matches = match_counts.get(&row.player_id).copied().unwrap_or(0);
    // ... build result
}).collect();
```

**❌ INCORRECT - Query inside iterator:**
```rust
// NEVER DO THIS - causes N+1 queries and violates borrow rules
let players = rows.into_iter().map(|row| {
    let matches = GameRepository::count_matches(&mut conn, row.player_id).unwrap_or(0); // ❌
    // ...
}).collect();
```

### Error Handling

**✅ CORRECT - Propagate errors with `?`:**
```rust
let match_counts = GameRepository::count_matches_for_players(&mut conn, &player_ids)
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
```

**❌ INCORRECT - Silent error suppression:**
```rust
// NEVER DO THIS - hides real errors
let matches = GameRepository::count_matches(&mut conn, id).unwrap_or(0);
```

### Dead Code Management

**✅ CORRECT - Delete unused code:**
```rust
// If code is no longer used, DELETE the entire file/module
```

**❌ INCORRECT - Suppressing warnings:**
```rust
// NEVER DO THIS
#[allow(dead_code)]
mod legacy_module {
    // unused code...
}
```

### Connection Management

**✅ CORRECT - Get connection once, pass to repositories:**
```rust
let mut conn = state.pool.get()
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

let players = PlayerRepository::list_ranked_players(&mut conn, &filter)?;
let counts = GameRepository::count_matches_for_players(&mut conn, &ids)?;
```

**❌ INCORRECT - Multiple connection acquisitions:**
```rust
// Avoid this - less efficient
for player in players {
    let mut conn = state.pool.get()?; // ❌ New connection each iteration
    // ...
}
```

### File Organization

**Repository Files:**
- One repository struct per file
- Methods grouped logically (CRUD, queries, batch operations)
- Use `impl` blocks for organization

**Service Files:**
- Business logic only, no SQL
- Use repositories for all data access
- Keep services stateless where possible

---

## 18. Recent Refactoring (Completed)

**Date:** December 2025
**Scope:** Complete Repository Pattern migration and performance optimization

### What Was Fixed:

1. **✅ Eliminated Duplicated SQL Queries**
   - Removed ~400 lines of duplicated code from legacy database modules
   - Established repositories as single source of truth

2. **✅ Fixed N+1 Query Problems**
   - Added `GameRepository::count_matches_for_players()` batch query
   - Refactored handlers to use HashMap-based lookups
   - Changed from O(n) individual queries to O(1) lookups

3. **✅ Completed Repository Pattern Migration**
   - Created 3 new repositories: `TournamentRepository`, `RatingRepository`, `AvatarRepository`
   - Migrated all services from direct SQL to repository pattern
   - Removed all legacy database modules (avatars.rs, games.rs, players.rs, ratings.rs, tournaments.rs)

4. **✅ Added Database Index**
   - Created index on `players.last_played` column
   - Improved "Active" filter performance from O(n) to O(log n)

5. **✅ Improved Error Handling**
   - Replaced silent `unwrap_or()` calls with proper `?` propagation
   - All errors now flow to `AppError` for consistent API responses

6. **✅ Code Quality Improvements**
   - Deleted dead code instead of suppressing warnings
   - Enforced strict repository pattern (NO SQL outside repositories)
   - Applied DRY principle aggressively

### Architecture Before vs After:

**BEFORE:**
- Mixed old/new patterns (confusion)
- 3 copies of same SQL queries
- N+1 query problems
- Silent error handling
- Missing database index

**AFTER:**
- Consistent repository pattern throughout
- Single source of truth for all queries
- Batch queries with O(1) lookups
- Proper error propagation
- Optimized with database indexes

---

## 14. File Organization & Git Strategy

### Design Decision: Data Files Tracked in Git

**Unlike typical web applications**, this project intentionally tracks data files in the repository:

#### What's Tracked:
- ✅ **Database**: `backend/data/warsaw_pool_ranking.db` (~50 MB)
- ✅ **Cache**: `backend/cache/raw/*.json` (~1500 files, ~30 MB)
- ✅ **Cache**: `backend/cache/parsed/tournaments.json`

#### Why:

1. **No External API Dependencies**: CueScore data is scraped (time-consuming, rate-limited)
2. **Portability**: Clone and have full data immediately
3. **Historical Preservation**: Tournaments may be deleted from source
4. **Development Speed**: No need to run ingestion on every fresh clone

#### Trade-offs:

**Pros:**
- One-command setup for new developers
- Guaranteed reproducibility
- Built-in backup through git history

**Cons:**
- Larger repository size (~80 MB vs ~1 MB without data)
- Slower clones (mitigated: only affects initial clone)
- Binary diff issues (git can't show meaningful diffs for .db files)

**Future Consideration**: If database exceeds 100 MB, migrate to git-lfs.

### Directory Structure

```
backend/
├── cache/                        # API response cache (tracked in git)
│   ├── README.md                 # Cache documentation
│   ├── raw/                      # Raw CueScore API responses
│   │   └── {tournament_id}.json  # Individual tournament data
│   └── parsed/                   # Processed data
│       └── tournaments.json      # Consolidated tournament list
│
├── data/                         # Application data (tracked in git)
│   ├── README.md                 # Data directory documentation
│   └── warsaw_pool_ranking.db    # SQLite database (~50 MB)
│
├── src/                          # Rust source code
│   └── ...
│
├── Cargo.toml                    # Rust dependencies
├── Dockerfile                    # Docker image definition
└── entrypoint.sh                 # Container startup script
```

### File Naming Conventions

**CRITICAL**: Database filename is **singular** (no 's'):
- ✅ `warsaw_pool_ranking.db` (correct)
- ❌ `warsaw_pool_rankings.db` (WRONG - this was a typo that has been fixed)

### Documentation Files

Each data directory includes a README.md explaining:
- Purpose of the directory
- File formats and structure
- Why files are tracked in git
- Configuration details
- Usage patterns

See:
- `backend/data/README.md`
- `backend/cache/README.md`

---

## 19. Future Enhancements

- **Database Migrations System**: Replace `schema.sql` resets with a proper migration tool (e.g., `sqlx-cli` or `refinery`) to handle schema evolution without data loss.
- **Automated Protobuf Sync**: Add CI/CD step to ensure frontend models are always in sync with proto definitions.
- **Venue Leaderboards**: "King of the Hill" stats per venue.
- **PWA Support**: Make the app installable on mobile devices.
- **Repository Trait Abstraction**: Add trait interfaces for repositories to enable mocking in tests.