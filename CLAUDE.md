# Warsaw Pool Rankings - LLM Context Documentation

**Last Updated:** 2025-12-23
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
14. [Key Design Decisions](#14-key-design-decisions)
15. [File Paths Reference](#15-file-paths-reference)
16. [Testing](#16-testing)
17. [Deployment](#17-deployment)
18. [Future Enhancements](#18-future-enhancements)

---

## 1. Project Overview

### Purpose
Warsaw Pool Rankings is a skill-based rating system for pool players in Warsaw, Poland. It collects game data from various pool venues and calculates player ratings using a sophisticated statistical model.

### Key Features
- **Bradley-Terry ML Rating System**: 100 points = 2:1 winning odds
- **Time Decay**: Exponential decay with 3-year half-life (recent games weighted more)
- **Multiple Rating Periods**: All-time, 1y, 2y, 3y, 4y, 5y
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
- Prefer `?` operator over manual error handling
- Use `impl Trait` for return types where appropriate
- Leverage type system for compile-time guarantees
- Modern async/await (not manual futures)
- Use `cargo clippy` and address ALL warnings
- Format with `cargo fmt`

**Angular**:
- Angular 17+ features exclusively
- Standalone components (NO NgModules)
- Modern control flow (`@if`, `@for`, `@switch`)
- Signals for reactive state (where appropriate)
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

```rust
// GOOD: Single responsibility
fn calculate_bradley_terry(games: &[Game]) -> Result<HashMap<i64, f64>> {
    // Only calculates ratings, nothing else
}

fn store_ratings(ratings: &HashMap<i64, f64>) -> Result<()> {
    // Only stores ratings, nothing else
}

// BAD: Multiple responsibilities
fn calculate_and_store_ratings(games: &[Game]) -> Result<()> {
    // Does too much
}
```

##### Open/Closed Principle (OCP)
- Open for extension, closed for modification
- Use traits (Rust) and interfaces (TypeScript)

```rust
// GOOD: Extensible rating strategy
trait RatingStrategy {
    fn calculate(&self, games: &[Game]) -> Result<f64>;
}

struct BradleyTerryStrategy;
impl RatingStrategy for BradleyTerryStrategy { /* ... */ }

struct EloStrategy;
impl RatingStrategy for EloStrategy { /* ... */ }
```

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

```rust
// BAD: Repetition
fn get_all_time_games(conn: &Connection) -> Result<Vec<Game>> {
    let mut stmt = conn.prepare("SELECT * FROM games")?;
    let games = stmt.query_map([], |row| {
        Ok(Game {
            id: row.get(0)?,
            tournament_id: row.get(1)?,
            // ... 10 more fields
        })
    })?;
    games.collect()
}

fn get_recent_games(conn: &Connection, days: i64) -> Result<Vec<Game>> {
    let mut stmt = conn.prepare("SELECT * FROM games WHERE date > ?")?;
    let games = stmt.query_map([days], |row| {
        Ok(Game {
            id: row.get(0)?,
            tournament_id: row.get(1)?,
            // ... 10 more fields (REPEATED)
        })
    })?;
    games.collect()
}

// GOOD: Extract common mapping
fn map_row_to_game(row: &Row) -> Result<Game> {
    Ok(Game {
        id: row.get(0)?,
        tournament_id: row.get(1)?,
        // ... 10 more fields (defined once)
    })
}

fn get_all_time_games(conn: &Connection) -> Result<Vec<Game>> {
    let mut stmt = conn.prepare("SELECT * FROM games")?;
    stmt.query_map([], map_row_to_game)?.collect()
}

fn get_recent_games(conn: &Connection, days: i64) -> Result<Vec<Game>> {
    let mut stmt = conn.prepare("SELECT * FROM games WHERE date > ?")?;
    stmt.query_map([days], map_row_to_game)?.collect()
}
```

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

```rust
// GOOD: Small, focused file
// backend/src/rating/bradley_terry.rs (150 lines)
pub fn calculate_bradley_terry(...) -> Result<HashMap<i64, f64>> { /* ... */ }
fn initialize_ratings(...) -> HashMap<i64, f64> { /* ... */ }
fn iterate_until_convergence(...) -> HashMap<i64, f64> { /* ... */ }

// GOOD: Related functionality in separate files
// backend/src/rating/mod.rs
// backend/src/rating/bradley_terry.rs
// backend/src/rating/weighting.rs
// backend/src/rating/types.rs
```

**TypeScript/Angular**:
```typescript
// GOOD: Small component
@Component({...})
export class PlayerCardComponent {
  // Max 100 lines of logic
  // Extract complex logic to services
}

// If component grows beyond 150 lines:
// 1. Extract child components
// 2. Extract logic to services
// 3. Extract utilities to separate files
```

#### 6. **Design Patterns**

Use proven patterns where appropriate:

**Rust**:
- **Builder Pattern**: For complex configuration
- **Strategy Pattern**: For interchangeable algorithms (rating strategies)
- **Repository Pattern**: For database access
- **Dependency Injection**: Via `Arc<AppState>`

**Angular**:
- **Service Pattern**: For business logic and state
- **Observable Pattern**: For reactive data streams
- **Facade Pattern**: Simplify complex subsystems
- **Singleton Services**: `providedIn: 'root'`

#### 7. **Boy Scout Rule**

> **"Always leave the codebase cleaner than you found it."**

**When making ANY change**:
1. Fix nearby code smells (long functions, magic numbers, etc.)
2. Add missing documentation
3. Improve variable names
4. Extract repeated code
5. Add type annotations if missing
6. Update outdated comments

**Example**:
```rust
// BEFORE (you're here to add a new field)
fn save_player(conn: &Connection, p: Player) -> Result<()> {
    conn.execute("INSERT INTO players (id, n, r) VALUES (?, ?, ?)",
                 params![p.id, p.n, p.r])?;
    Ok(())
}

// AFTER (you improved it while adding new field)
fn save_player(conn: &Connection, player: Player) -> Result<()> {
    conn.execute(
        "INSERT INTO players (id, name, rating, avatar_url) VALUES (?, ?, ?, ?)",
        params![player.id, player.name, player.rating, player.avatar_url],
    )?;
    Ok(())
}
```

### Code Review Checklist

Before committing ANY code, verify:

- [ ] **No compiler warnings** (`cargo build`, `ng build`)
- [ ] **No linter errors** (`cargo clippy`, `ng lint`)
- [ ] **All tests pass** (`cargo test`, `npm test`)
- [ ] **Code is formatted** (`cargo fmt`, `prettier`)
- [ ] **Functions are small** (< 30 lines)
- [ ] **Files are focused** (< 300 lines)
- [ ] **No code duplication** (DRY principle applied)
- [ ] **SOLID principles followed**
- [ ] **Dependencies updated** (if touching package files)
- [ ] **Documentation updated** (if public API changed)
- [ ] **Improved surrounding code** (Boy Scout Rule)

### Refactoring Triggers

**Immediate refactoring required when**:
- Function exceeds 30 lines
- File exceeds 300 lines
- Same code appears in 3+ places
- Function has more than 4 parameters
- Cyclomatic complexity > 10
- Any `cargo clippy` or `ng lint` warning

**Example - Complex Function Refactoring**:

```rust
// BEFORE: 80 lines, too complex
fn process_tournament(tournament_id: i64) -> Result<()> {
    // Fetch tournament
    let response = reqwest::get(&url).await?;
    let tournament = response.json::<Tournament>().await?;

    // Validate tournament
    if tournament.name.is_empty() { return Err(...); }
    if tournament.games.is_empty() { return Err(...); }

    // Process games
    for game in tournament.games {
        // ... 20 lines of game processing
    }

    // Calculate ratings
    // ... 30 lines of rating calculation

    // Save to database
    // ... 20 lines of database operations

    Ok(())
}

// AFTER: Split into focused functions
fn process_tournament(tournament_id: i64) -> Result<()> {
    let tournament = fetch_tournament(tournament_id).await?;
    validate_tournament(&tournament)?;
    let games = process_games(&tournament.games)?;
    let ratings = calculate_ratings(&games)?;
    save_ratings(&ratings)?;
    Ok(())
}

fn fetch_tournament(id: i64) -> Result<Tournament> { /* ... */ }
fn validate_tournament(t: &Tournament) -> Result<()> { /* ... */ }
fn process_games(games: &[RawGame]) -> Result<Vec<Game>> { /* ... */ }
fn calculate_ratings(games: &[Game]) -> Result<HashMap<i64, f64>> { /* ... */ }
fn save_ratings(ratings: &HashMap<i64, f64>) -> Result<()> { /* ... */ }
```

### Naming Conventions

**Be Explicit, Not Clever**:

```rust
// GOOD
fn calculate_time_decay_weight(game_date: &NaiveDate, reference_date: &NaiveDate) -> f64

// BAD
fn calc_w(d1: &NaiveDate, d2: &NaiveDate) -> f64
```

**Rust**:
- Functions: `snake_case`
- Types/Structs: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

**TypeScript/Angular**:
- Variables/Functions: `camelCase`
- Classes/Interfaces: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Files: `kebab-case.ts`

### Performance Considerations

**Optimize for Readability First**:
- Clean code is easier to optimize later
- Measure before optimizing (use profiling tools)
- Document why any "clever" optimization exists

```rust
// GOOD: Clear, easy to optimize later if needed
fn sum_ratings(players: &[Player]) -> f64 {
    players.iter().map(|p| p.rating).sum()
}

// BAD: Premature optimization, unclear
fn sum_ratings(players: &[Player]) -> f64 {
    unsafe { /* ... hand-rolled SIMD ... */ }
}
```

### Documentation

**Code Should Be Self-Documenting**:
- Good names reduce need for comments
- Comments explain "why", not "what"

```rust
// BAD: Comment explains what (obvious)
// Increments counter by 1
counter += 1;

// GOOD: Comment explains why
// Skip first game to avoid cold-start bias in rating calculation
let games = all_games.iter().skip(1);
```

**Module-Level Documentation**:
```rust
//! # Bradley-Terry Rating Module
//!
//! Implements Maximum Likelihood estimation for Bradley-Terry model.
//! Uses iterative convergence algorithm with configurable tolerance.
//!
//! ## References
//! - Bradley, R. A.; Terry, M. E. (1952). "Rank Analysis of Incomplete Block Designs"
```

---

## 3. Architecture Overview

### High-Level System Design

```
┌─────────────────────────────────────────────────┐
│  Angular Frontend (Port 80)                     │
│  - Player rankings table with search            │
│  - Player detail overlay with history chart     │
│  - Admin panel for data refresh                 │
└────────────┬────────────────────────────────────┘
             │
             │ HTTP GET /api/* (reads from SQLite)
             │ HTTP POST /api/admin/* (triggers processing)
             │
             ▼
┌─────────────────────────────────────────────────┐
│  Rust Backend (Port 8000)                       │
│  - Axum HTTP server for API endpoints           │
│  - CLI commands: serve, ingest, process         │
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
4. **Serving**: Backend exposes REST API endpoints
5. **Display**: Frontend queries API and displays rankings

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
- **Modern Control Flow**: `@if`, `@for`, `@switch` (better than `*ngIf`, `*ngFor`)
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
│   │   ├── client.rs           # CueScore API client
│   │   ├── handlers/           # HTTP request handlers
│   │   │   ├── admin.rs        # Admin panel endpoints
│   │   │   ├── avatars.rs      # Avatar serving endpoints
│   │   │   └── players.rs      # Player data endpoints
│   │   ├── models.rs           # API request/response models (protobuf)
│   │   └── routes.rs           # Axum route definitions
│   ├── cache/                   # Caching layer for API responses
│   │   └── mod.rs
│   ├── cli/                     # Command-line interface
│   │   └── mod.rs              # clap command definitions
│   ├── config/                  # Configuration management
│   │   ├── settings.rs         # AppConfig, RatingSettings, etc.
│   │   └── venues.rs           # Venue configuration (hardcoded list)
│   ├── database/                # SQLite interaction
│   │   ├── mod.rs              # Database connection & initialization
│   │   ├── schema.sql          # SQL schema definition
│   │   └── repositories/       # Data access layer
│   │       ├── games.rs
│   │       ├── players.rs
│   │       ├── ratings.rs
│   │       └── avatars.rs
│   ├── domain/                  # Core domain models
│   │   ├── game.rs
│   │   ├── player.rs
│   │   ├── rating.rs
│   │   └── tournament.rs
│   ├── fetchers/                # Web scraping logic
│   │   ├── mod.rs
│   │   └── tournament_fetcher.rs
│   ├── http/                    # HTTP client with rate limiting
│   │   └── mod.rs
│   ├── rating/                  # Bradley-Terry rating algorithm
│   │   ├── mod.rs
│   │   ├── bradley_terry.rs    # Core algorithm implementation
│   │   ├── types.rs            # Rating-related types
│   │   └── weighting.rs        # Time decay calculations
│   └── services/                # High-level business logic
│       ├── ingestion_service.rs   # Data collection orchestration
│       ├── processing_service.rs  # Rating calculation orchestration
│       └── avatar_service.rs      # Avatar download & processing
├── Cargo.toml                   # Rust dependencies
├── build.rs                     # Build script (protobuf compilation)
└── Dockerfile                   # Docker image definition
```

### Key Design Patterns

#### 1. Dependency Injection via AppState

```rust
pub struct AppState {
    pub config: AppConfig,
    pub db_pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
}

// Injected into handlers via axum's State extractor
pub async fn admin_refresh(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Access state.config and state.db_pool
}
```

**Why**: Centralized configuration and shared resources (connection pool).

#### 2. Separation of Concerns

- **Domain Layer**: Pure business logic, no dependencies on database or HTTP
- **Repository Layer**: Database access, returns domain models
- **Service Layer**: Orchestrates multiple repositories and domain logic
- **API Layer**: HTTP request/response handling, calls services

**Why**: Testability, maintainability, clear boundaries.

#### 3. Error Handling with anyhow

```rust
use anyhow::Result;

pub fn calculate_rating(games: Vec<Game>) -> Result<f64> {
    // ... logic
    Ok(rating)
}
```

**Why**: Simplified error propagation with `?` operator, context-rich error messages.

#### 4. Configuration via Environment Variables

```rust
impl AdminSettings {
    pub fn new() -> Self {
        let password = std::env::var("ADMIN_PASSWORD")
            .unwrap_or_else(|_| {
                log::warn!("ADMIN_PASSWORD not set, using default 'admin' (INSECURE!)");
                "admin".to_string()
            });
        Self { password }
    }
}
```

**Why**: 12-factor app methodology, easy configuration for different environments.

#### 5. Connection Pooling with r2d2

```rust
let manager = r2d2_sqlite::SqliteConnectionManager::file(db_path);
let pool = r2d2::Pool::builder()
    .max_size(10)
    .build(manager)?;
```

**Why**: Efficient concurrent database access without recreating connections.

### CLI Commands

The backend is primarily a CLI application with an HTTP server mode:

#### `serve --port <PORT>`
- Starts Axum HTTP server (default port: 8000)
- Serves API endpoints for frontend
- Does NOT automatically ingest or process data

#### `ingest`
- Scrapes CueScore venue pages for tournament lists
- Fetches tournament details via CueScore API
- Caches raw API responses to `backend/cache/`
- Parses matches into individual games
- Stores data in SQLite database

#### `process`
- Reads cached data from database
- Calculates Bradley-Terry ratings for all periods
- Updates ratings table with new calculations
- Does NOT fetch new data (use `ingest` first if needed)

### Configuration

All configuration is centralized in `backend/src/config/settings.rs`:

```rust
pub struct AppConfig {
    pub rating: RatingSettings,
    pub scraper: ScraperSettings,
    pub admin: AdminSettings,
}
```

**RatingSettings:**
- `starter_rating`: 500.0 (baseline for new players)
- `virtual_games_weight`: 5.0 (weight for virtual games in blending)
- `min_ranked_games`: 50 (minimum games to be "ranked")
- `established_games`: 200 (games needed for "established" status)
- `convergence_tolerance`: 1e-6 (for Bradley-Terry iteration)
- `max_iterations`: 100 (Bradley-Terry iteration limit)
- `periods`: Rating periods (all, 1y, 2y, 3y, 4y, 5y)

**ScraperSettings:**
- `rate_limit_ms`: 100 (10 requests/second)
- `user_agent`: "WarsawPoolRankings/2.0"
- `timeout_secs`: 30
- `base_url`: "https://cuescore.com"
- `api_base_url`: "https://api.cuescore.com"

**AdminSettings:**
- `password`: From `ADMIN_PASSWORD` env var (default: "admin" with warning)

---

## 6. Frontend Architecture

### Angular 17 Patterns

The frontend uses Angular 17's latest features for a modern, performant SPA:

#### Standalone Components
No NgModules! Every component is self-contained:

```typescript
@Component({
  selector: 'app-rankings-list',
  standalone: true,
  imports: [CommonModule, MatTableModule, MatInputModule, ...],
  templateUrl: './rankings-list.component.html',
  styleUrls: ['./rankings-list.component.scss']
})
export class RankingsListComponent {
  // ...
}
```

#### Modern Control Flow
Uses `@if`, `@for`, `@switch` instead of `*ngIf`, `*ngFor`:

```html
@if (players.length > 0) {
  <mat-table [dataSource]="dataSource">
    @for (player of players; track player.id) {
      <mat-row>{{ player.name }}</mat-row>
    }
  </mat-table>
} @else {
  <p>No players found</p>
}
```

**Why**: Better performance, more intuitive syntax, type safety.

#### Services Pattern

All state and HTTP logic is in services:

```typescript
@Injectable({ providedIn: 'root' })
export class DatabaseService {
  private apiUrl = '/api';

  constructor(private http: HttpClient) {}

  getPlayers(period: string): Observable<Player[]> {
    return this.http.get<Player[]>(`${this.apiUrl}/players?period=${period}`);
  }
}
```

**Why**: Separation of concerns, testability, reusability.

### Key Services

#### DatabaseService (`database.service.ts`)
- Queries backend API for player data
- Fetches player details and rating history
- Observable-based for reactive UI

**Example:**
```typescript
getPlayerDetails(playerId: number): Observable<PlayerDetail> {
  return this.http.get<PlayerDetail>(`${this.apiUrl}/players/${playerId}`);
}
```

#### AuthService (`auth.service.ts`)
- Handles admin login with server-side validation
- Stores password in localStorage after validation
- Provides Bearer token for authenticated requests

**Key Flow:**
```typescript
login(password: string): Observable<{ success: boolean; error?: string }> {
  return this.http.post<LoginResponse>(`${this.apiUrl}/login`, { password })
    .pipe(
      map(response => {
        if (response.success) {
          localStorage.setItem(this.tokenKey, password);
          return { success: true };
        } else {
          return { success: false, error: response.message };
        }
      })
    );
}
```

#### AdminService (`admin.service.ts`)
- Wraps AuthService for admin operations
- Triggers data refresh and avatar refresh
- Handles authentication state

#### TranslateService
- i18n support (currently unused, ready for future localization)

### Components

#### RankingsListComponent
**Purpose**: Main player list with search and filtering

**Features:**
- Material table with sorting
- Search by player name
- Filter by rating period (all, 1y, 2y, etc.)
- Click to open player overlay

**State:**
- `players: Player[]` - Current player list
- `dataSource: MatTableDataSource` - Material table data
- `searchQuery: string` - Search input binding

#### PlayerOverlayComponent
**Purpose**: Player detail modal with rating history chart

**Features:**
- Player info (name, avatar, current rating)
- Chart.js line chart of rating history
- Confidence level indicator
- Close button to return to list

**State:**
- `player: PlayerDetail` - Selected player data
- `chart: Chart` - Chart.js instance

#### AdminComponent
**Purpose**: Admin panel with authentication

**Features:**
- Password login form
- Refresh data button (triggers backend `ingest` + `process`)
- Refresh avatars button (triggers avatar re-download)
- Logout button
- Loading states for async operations

**State:**
- `password: string` - Login password input
- `loading: boolean` - Refresh operation in progress
- `loginLoading: boolean` - Login request in progress

### Routing

The app uses a **simple SPA with overlay-based navigation**:
- No traditional Angular Router routes
- Main view: `RankingsListComponent`
- Overlay: `PlayerOverlayComponent` (opened via service)

**Why**: Simpler architecture, better UX (no page refreshes), easier state management.

---

## 7. Database Schema

### Overview

SQLite schema with 5 core tables, fully normalized (3NF):

```
players (1) ──── (N) games (N) ──── (1) tournaments
   │
   │ (1:N)
   ├── ratings
   └── avatars (1:3, one per size)
```

### Tables

#### `players`
Stores player profiles with CueScore ID for deduplication.

```sql
CREATE TABLE players (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cuescore_id INTEGER UNIQUE,
    name TEXT NOT NULL,
    avatar_url TEXT,  -- DEPRECATED: Use avatars table instead
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_players_cuescore_id ON players(cuescore_id);
```

**Design Notes:**
- `cuescore_id` is UNIQUE to prevent duplicate players
- `avatar_url` is deprecated (kept for backward compatibility)
- `name` is NOT unique (some players share names)

#### `tournaments`
Tournament metadata from CueScore.

```sql
CREATE TABLE tournaments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cuescore_id INTEGER UNIQUE NOT NULL,
    name TEXT NOT NULL,
    venue_id INTEGER NOT NULL,
    venue_name TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_tournaments_cuescore_id ON tournaments(cuescore_id);
CREATE INDEX idx_tournaments_start_date ON tournaments(start_date);
```

**Design Notes:**
- `cuescore_id` is UNIQUE to prevent duplicate tournaments
- `start_date` indexed for time-based queries
- `venue_id` and `venue_name` denormalized for convenience

#### `games`
Individual games (expanded from match scores).

```sql
CREATE TABLE games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tournament_id INTEGER NOT NULL REFERENCES tournaments(id),
    first_player_id INTEGER NOT NULL REFERENCES players(id),
    second_player_id INTEGER NOT NULL REFERENCES players(id),
    first_player_score INTEGER NOT NULL,
    second_player_score INTEGER NOT NULL,
    date TEXT NOT NULL,
    weight REAL NOT NULL DEFAULT 1.0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_games_tournament ON games(tournament_id);
CREATE INDEX idx_games_first_player ON games(first_player_id);
CREATE INDEX idx_games_second_player ON games(second_player_id);
CREATE INDEX idx_games_date ON games(date);
```

**Design Notes:**
- Each row is a **single game**, not a match (matches are expanded to games)
- Example: "5-3" match = 8 games (5 wins for player 1, 3 wins for player 2)
- `weight` is for future use (currently always 1.0)
- All player references indexed for rating calculation queries
- `date` indexed for time decay calculations

#### `ratings`
Calculated ratings for each player and period.

```sql
CREATE TABLE ratings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_id INTEGER NOT NULL REFERENCES players(id),
    rating_type TEXT NOT NULL,  -- 'all', '1y', '2y', etc.
    rating REAL NOT NULL,
    games_played INTEGER NOT NULL,
    confidence_level TEXT NOT NULL,  -- 'unranked', 'provisional', etc.
    calculated_at TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ratings_player ON ratings(player_id);
CREATE INDEX idx_ratings_type_rank ON ratings(rating_type, rating DESC);
CREATE INDEX idx_ratings_calculated_at ON ratings(calculated_at);
```

**Design Notes:**
- Multiple ratings per player (one for each period)
- `rating_type`: "all", "1y", "2y", "3y", "4y", "5y"
- `confidence_level`: "unranked", "provisional", "emerging", "established"
- `calculated_at` tracks when rating was calculated (for invalidation)
- Composite index on `(rating_type, rating DESC)` for leaderboard queries

#### `avatars`
Local avatar storage with hash-based change detection.

```sql
CREATE TABLE avatars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_id INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    size TEXT NOT NULL CHECK(size IN ('small', 'medium', 'large')),
    image_data BLOB NOT NULL,
    format TEXT NOT NULL DEFAULT 'webp',
    source_url TEXT NOT NULL,
    source_url_hash TEXT NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(player_id, size)
);

CREATE INDEX idx_avatars_player_size ON avatars(player_id, size);
CREATE INDEX idx_avatars_source_hash ON avatars(source_url_hash);
```

**Design Notes:**
- Three sizes: small (50px), medium (150px), large (300px)
- `image_data` stored as WebP BLOB for efficiency
- `source_url_hash` enables change detection (avoid re-download if URL hash matches)
- `UNIQUE(player_id, size)` ensures one avatar per size per player
- `ON DELETE CASCADE` automatically removes avatars when player deleted

### Design Decisions

#### Why Normalized Schema (3NF)?
- **Data Integrity**: No duplication, easier updates
- **Flexibility**: Easy to add new tables or relationships
- **SQLite Efficiency**: Join performance is excellent for our dataset size

#### Why Index Foreign Keys?
- **Rating Calculation**: Queries like "all games for player X" are critical
- **Join Performance**: Lookups by player_id, tournament_id are frequent

#### Why Deprecate `avatar_url`?
- **Reliability**: CueScore URLs change unpredictably
- **Performance**: Local storage eliminates external HTTP requests
- **Consistency**: Hash-based change detection prevents redundant downloads

---

## 8. Rating Algorithm

### Bradley-Terry Maximum Likelihood

The project uses the **Bradley-Terry model** for skill rating, a probabilistic model where the probability of player *i* beating player *j* is:

```
P(i beats j) = exp(rating_i) / (exp(rating_i) + exp(rating_j))
```

#### Implementation

**File**: `backend/src/rating/bradley_terry.rs`

**Algorithm**: Iterative maximum likelihood estimation
1. Initialize all ratings to `starter_rating` (500.0)
2. Iteratively update ratings until convergence:
   - For each player, calculate expected wins vs. actual wins
   - Adjust rating based on difference
   - Repeat until change < `convergence_tolerance` (1e-6) or max iterations (100)

**Library**: Uses `nalgebra` for matrix operations (efficient linear algebra)

**Scale**: **100 points = 2:1 winning odds**
- Example: Player with 600 rating vs. 500 rating has ~2:1 odds
- 200 point difference = 4:1 odds
- 300 point difference = 8:1 odds

#### Code Snippet
```rust
pub fn calculate_bradley_terry(
    games: &[Game],
    starter_rating: f64,
    convergence_tolerance: f64,
    max_iterations: usize,
) -> Result<HashMap<i64, f64>> {
    // Initialize ratings
    let mut ratings = HashMap::new();
    for game in games {
        ratings.entry(game.player1_id).or_insert(starter_rating);
        ratings.entry(game.player2_id).or_insert(starter_rating);
    }

    // Iterative ML estimation
    for iteration in 0..max_iterations {
        let mut new_ratings = ratings.clone();
        let mut max_change = 0.0;

        for (&player_id, &current_rating) in &ratings {
            // Calculate expected vs. actual wins
            // Update rating based on difference
            // Track max change
        }

        if max_change < convergence_tolerance {
            log::info!("Converged after {} iterations", iteration + 1);
            break;
        }

        ratings = new_ratings;
    }

    Ok(ratings)
}
```

### Time Decay

Recent games are weighted more heavily using **exponential decay** with a **3-year half-life**:

```
weight = exp(-λ × days_ago)

where λ = ln(2) / 1095 (3 years in days)
```

**Examples:**
- Game today: weight = 1.0
- Game 3 years ago: weight = 0.5
- Game 6 years ago: weight = 0.25

**Implementation**: `backend/src/rating/weighting.rs`

```rust
pub fn calculate_time_weight(game_date: &NaiveDate, reference_date: &NaiveDate) -> f64 {
    const HALF_LIFE_DAYS: f64 = 1095.0; // 3 years
    let lambda = LN_2 / HALF_LIFE_DAYS;

    let days_ago = (reference_date.signed_duration_since(*game_date)).num_days() as f64;
    (-lambda * days_ago).exp()
}
```

**Applied During**: Rating calculation (not stored in database)

**Why**: More recent performance is more indicative of current skill.

### New Player Blending

New players' ratings are blended with a starter rating to avoid volatility:

```
0-9 games:    Unranked (100% starter rating, 500.0)
10-49 games:  Provisional (blend starter + ML rating)
50-99 games:  Emerging (mostly ML rating)
100+ games:   Established (pure ML rating)
```

**Implementation**:
```rust
pub fn blend_rating(ml_rating: f64, games_played: i32, config: &RatingSettings) -> (f64, String) {
    let confidence_level = match games_played {
        0..=9 => "unranked",
        10..=49 => "provisional",
        50..=99 => "emerging",
        _ => "established",
    };

    let blended_rating = if games_played < config.min_ranked_games {
        // Virtual games approach: blend starter rating with ML rating
        let virtual_weight = config.virtual_games_weight;
        let total_weight = games_played as f64 + virtual_weight;
        (ml_rating * games_played as f64 + config.starter_rating * virtual_weight) / total_weight
    } else {
        ml_rating
    };

    (blended_rating, confidence_level.to_string())
}
```

**Why**: Prevents new players from having wildly inaccurate ratings after just a few games.

### Rating Periods

Ratings are calculated for **6 different time periods**:

| Period | Name  | Games Included        |
|--------|-------|-----------------------|
| All    | `all` | All games ever        |
| 1 Year | `1y`  | Games in last 365 days|
| 2 Year | `2y`  | Games in last 730 days|
| 3 Year | `3y`  | Games in last 1095 days|
| 4 Year | `4y`  | Games in last 1460 days|
| 5 Year | `5y`  | Games in last 1825 days|

**Implementation**:
- Each period calculated independently
- Stored in `ratings` table with `rating_type` column
- Frontend allows filtering by period

**Why**: Allows tracking recent form vs. all-time skill, handles inactive players better.

---

## 9. Data Collection

### CueScore Integration

The backend collects data from CueScore in 4 steps:

#### Step 1: Scrape Tournament Lists
**File**: `backend/src/fetchers/tournament_fetcher.rs`

For each venue in `backend/src/config/venues.rs`:
1. Fetch `https://cuescore.com/venue/{slug}/{id}/tournaments`
2. Parse HTML for tournament links
3. Extract tournament IDs

**Example**:
```rust
pub struct VenueConfig {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

// From venues.rs
VenueConfig {
    id: 2842336,
    name: "147 Break Zamieniecka".to_string(),
    slug: "147-break-zamieniecka".to_string(),
}
```

#### Step 2: Fetch Tournament Details
**File**: `backend/src/api/client.rs`

For each tournament ID:
1. Call `https://api.cuescore.com/tournaments/{id}`
2. Parse JSON response (protobuf models)
3. Extract matches, players, dates

#### Step 3: Parse Matches into Games
**File**: `backend/src/domain/game.rs`

For each match with score "X-Y":
1. Create X games where first_player wins
2. Create Y games where second_player wins
3. Assign same date to all games

**Example**: Match "5-3" becomes 8 game rows:
- 5 rows: `first_player_score=1, second_player_score=0`
- 3 rows: `first_player_score=0, second_player_score=1`

**Why**: Bradley-Terry model operates on individual game outcomes, not match scores.

#### Step 4: Rate Limiting
**File**: `backend/src/http/mod.rs`

```rust
const RATE_LIMIT_MS: u64 = 100; // 10 requests per second

async fn rate_limited_get(url: &str) -> Result<Response> {
    tokio::time::sleep(Duration::from_millis(RATE_LIMIT_MS)).await;
    reqwest::get(url).await
}
```

**Why**: Respectful scraping, avoid overloading CueScore servers.

### Venues Configuration

**File**: `backend/src/config/venues.rs`

Venues are **hardcoded** in Rust (not in database or env vars):

```rust
pub fn get_venues() -> Vec<VenueConfig> {
    vec![
        VenueConfig {
            id: 2842336,
            name: "147 Break Zamieniecka".to_string(),
            slug: "147-break-zamieniecka".to_string(),
        },
        VenueConfig {
            id: 1698108,
            name: "147 Break Nowogrodzka".to_string(),
            slug: "147-break-nowogrodzka".to_string(),
        },
    ]
}
```

**How to Add Venues**:
1. Find venue on CueScore
2. Extract from URL: `https://cuescore.com/venue/{slug}/{id}/tournaments`
3. Add new `VenueConfig` to `get_venues()` function
4. Rebuild backend

**Why Hardcoded**: Venues change infrequently, simpler than database management.

### Caching Strategy

**Directory**: `backend/cache/`

**What's Cached**:
- Raw CueScore API responses (JSON)
- Per-tournament files: `cache/tournaments/{id}.json`

**How It Works**:
1. `ingest` command fetches data and writes to cache
2. `process` command reads from cache (does not fetch)
3. Subsequent `process` runs use cached data (no network calls)

**When to Refresh**:
- Run `ingest` command to fetch fresh data
- Manually delete cache files to force re-fetch

**Why Cache**:
- **Efficiency**: No need to re-fetch unchanged data
- **Development**: Fast iteration without network calls
- **Respectful**: Reduces load on CueScore servers

---

## 10. Authentication & Security

### Admin Panel Authentication

The admin panel uses **password-based authentication** with server-side validation.

#### Flow

1. **User enters password** in admin login form
2. **Frontend calls** `POST /api/admin/login` with password
3. **Backend validates** password against `ADMIN_PASSWORD` env var
4. **Backend returns** `{ success: true/false, message?: string }`
5. **Frontend stores** password in localStorage only if success=true
6. **Subsequent requests** send password as Bearer token in Authorization header
7. **Backend validates** Bearer token on protected endpoints

#### Implementation

**Backend Endpoint** (`backend/src/api/handlers/admin.rs`):
```rust
pub async fn admin_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let is_valid = payload.password == state.config.admin.password;

    if is_valid {
        log::info!("Admin login successful");
        Json(LoginResponse {
            success: true,
            message: None,
        }).into_response()
    } else {
        log::warn!("Admin login failed - invalid password");
        Json(LoginResponse {
            success: false,
            message: Some("Invalid password".to_string()),
        }).into_response()
    }
}
```

**Note**: Returns `200 OK` with `success: false` (not `401`) to avoid browser password dialogs.

**Frontend Service** (`frontend/src/app/services/auth.service.ts`):
```typescript
login(password: string): Observable<{ success: boolean; error?: string }> {
  return this.http.post<LoginResponse>(`${this.apiUrl}/login`, { password })
    .pipe(
      map(response => {
        if (response.success) {
          localStorage.setItem(this.tokenKey, password);
          return { success: true };
        } else {
          return { success: false, error: response.message || 'Invalid password' };
        }
      }),
      catchError(error => {
        console.error('Login request failed:', error);
        return of({ success: false, error: 'Network error. Please try again.' });
      })
    );
}
```

**Protected Endpoint** (`backend/src/api/handlers/admin.rs`):
```rust
fn validate_auth_header(headers: &HeaderMap, expected_password: &str) -> bool {
    headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|token| token == expected_password)
        .unwrap_or(false)
}

pub async fn admin_refresh(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !validate_auth_header(&headers, &state.config.admin.password) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    // ... trigger refresh
}
```

#### Environment Variable Configuration

**File**: `backend/src/config/settings.rs`

```rust
pub struct AdminSettings {
    pub password: String,
}

impl AdminSettings {
    pub fn new() -> Self {
        let password = std::env::var("ADMIN_PASSWORD")
            .unwrap_or_else(|_| {
                log::warn!("ADMIN_PASSWORD not set, using default 'admin' (INSECURE for production!)");
                "admin".to_string()
            });
        Self { password }
    }
}
```

#### Default Passwords

| Environment | Default Password | Warning Logged |
|-------------|------------------|----------------|
| Development (local) | `admin` | Yes |
| Docker (if env var not set) | `changeme` | Yes |
| Production | **MUST SET** | Fails if not set |

**Example Configuration**:

`.env` (local development):
```bash
ADMIN_PASSWORD=admin
```

`docker-compose.yml`:
```yaml
environment:
  ADMIN_PASSWORD: ${ADMIN_PASSWORD:-changeme}
```

Production:
```bash
# Generate secure password
openssl rand -base64 32

# Set environment variable
export ADMIN_PASSWORD=<generated-password>
```

### Security Notes

#### ✅ Good Practices
- Server-side password validation (not just client-side)
- Environment variable configuration (not hardcoded)
- Logging of failed authentication attempts
- Clear warnings for default passwords

#### ⚠️ Known Limitations
- **No Rate Limiting**: No protection against brute force (future enhancement)
- **No Session Tokens**: Password used directly as Bearer token
- **localStorage Storage**: Password stored in browser (acceptable for admin-only feature)
- **No HTTPS Enforcement**: Must configure reverse proxy in production

#### 🔒 Production Checklist
1. Set strong `ADMIN_PASSWORD` (min 20 characters, use `openssl rand -base64 32`)
2. Configure HTTPS reverse proxy (Nginx, Caddy, etc.)
3. Never commit passwords to version control
4. Rotate password periodically
5. Monitor logs for failed authentication attempts

---

## 11. Avatar Storage

### Local Storage Design

Player avatars are stored **locally in SQLite** rather than linking to external URLs.

#### Schema

**Table**: `avatars`

**Three Sizes**:
- `small`: 50px (list view thumbnails)
- `medium`: 150px (player overlay)
- `large`: 300px (high-DPI displays, future use)

**Format**: WebP (better compression than PNG/JPEG)

**Storage**: BLOB column in SQLite

#### Hash-Based Change Detection

To avoid re-downloading unchanged avatars, the system uses **SHA-256 hashing** of the source URL:

```rust
use sha2::{Sha256, Digest};

pub fn hash_url(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

**Flow**:
1. Fetch player's `avatar_url` from CueScore API
2. Calculate `source_url_hash` from URL
3. Check if `avatars` table has matching `source_url_hash` for this player
4. If hash matches: **Skip download** (avatar unchanged)
5. If hash differs or missing: **Download and process** avatar

**Why**: Dramatically reduces bandwidth and processing time for avatar refreshes.

#### Image Processing

**Library**: `image` crate with `webp` feature

**Process**:
1. Download image from CueScore URL
2. Decode to RGBA8 image
3. Resize to target dimensions (50px, 150px, 300px) maintaining aspect ratio
4. Encode as WebP (quality: 80)
5. Store BLOB in database

**Code Snippet**:
```rust
use image::ImageFormat;
use webp::Encoder;

pub fn resize_and_encode_webp(
    image_data: &[u8],
    target_width: u32,
    quality: f32,
) -> Result<Vec<u8>> {
    // Decode original image
    let img = image::load_from_memory(image_data)?;

    // Resize maintaining aspect ratio
    let resized = img.resize(target_width, u32::MAX, image::imageops::FilterType::Lanczos3);

    // Encode as WebP
    let encoder = Encoder::from_image(&resized)?;
    let webp_data = encoder.encode(quality);

    Ok(webp_data.to_vec())
}
```

### Why Local Storage?

#### Problem with External URLs
- **Unreliable**: CueScore avatar URLs change unpredictably
- **Performance**: Every page load requires external HTTP requests
- **Failures**: Broken images if CueScore is down or changes CDN

#### Benefits of Local Storage
- **Reliability**: Avatars always available (in SQLite database)
- **Performance**: No external HTTP requests, instant loading
- **Consistency**: WebP format with controlled quality
- **Change Detection**: Hash-based system avoids redundant downloads

### API Endpoints

**Serve Avatar** (`GET /api/players/{id}/avatar?size={size}`):
```rust
pub async fn get_player_avatar(
    State(state): State<Arc<AppState>>,
    Path(player_id): Path<i64>,
    Query(params): Query<AvatarParams>,
) -> impl IntoResponse {
    let conn = state.db_pool.get()?;
    let avatar = avatars_repo::get_avatar(&conn, player_id, &params.size)?;

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/webp")
        .body(avatar.image_data)
}
```

**Refresh Avatars** (`POST /api/admin/refresh-avatars`):
- Requires admin authentication
- Re-downloads avatars for all players (using hash-based change detection)
- Returns 202 Accepted (background processing)

---

## 12. Development Workflows

### Backend Development

#### Local Setup (Without Docker)

```bash
cd backend

# First-time setup
cargo build --release

# Run database initialization (creates tables)
./target/release/warsaw_pool_ranking process

# Fetch fresh data from CueScore
./target/release/warsaw_pool_ranking ingest

# Calculate ratings from cached data
./target/release/warsaw_pool_ranking process

# Start HTTP server
./target/release/warsaw_pool_ranking serve --port 8000
```

#### Development with Cargo Watch (Hot Reload)

```bash
# Install cargo-watch
cargo install cargo-watch

# Auto-recompile and run on file changes
cargo watch -x 'run -- serve'
```

#### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_bradley_terry

# With output
cargo test -- --nocapture

# Documentation tests
cargo test --doc
```

#### Logging

Set `RUST_LOG` environment variable:

```bash
# Debug (verbose)
RUST_LOG=debug cargo run -- serve

# Info (default)
RUST_LOG=info cargo run -- serve

# Warnings only
RUST_LOG=warn cargo run -- serve
```

### Frontend Development

#### Local Setup

```bash
cd frontend

# Install dependencies
npm install

# Development server (with hot reload)
npm start
# Open http://localhost:4200

# Production build
npm run build
# Output: dist/warsaw-pool-rankings-frontend
```

#### Running Tests

```bash
# Unit tests (Jasmine + Karma)
npm test

# Lint
npm run lint
```

#### Protobuf Generation

If backend API models change:

```bash
# From frontend directory
npm run proto:generate:local
```

This generates TypeScript models from `../proto/api.proto`.

### Docker Development

#### Build and Start Services

```bash
# Build and start both services
docker-compose up -d --build

# Start without rebuild
docker-compose up -d

# View logs
docker-compose logs -f

# View backend logs only
docker-compose logs -f backend
```

#### Running Commands in Container

```bash
# Ingest fresh data
docker-compose exec backend ./warsaw_pool_ranking ingest

# Process ratings
docker-compose exec backend ./warsaw_pool_ranking process

# Access SQLite database
docker-compose exec backend sqlite3 /app/data/warsaw_pool_ranking.db
```

#### Stopping Services

```bash
# Stop services (keep containers)
docker-compose stop

# Stop and remove containers
docker-compose down

# Stop and remove containers + volumes (deletes database!)
docker-compose down -v
```

### Environment Setup

#### Local Development

1. Copy `.env.example` to `.env`:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env`:
   ```bash
   ADMIN_PASSWORD=admin
   RUST_LOG=info
   ```

3. Run backend:
   ```bash
   cd backend
   cargo run -- serve
   ```

#### Docker Development

1. Set environment variable (optional):
   ```bash
   export ADMIN_PASSWORD=mysecurepassword
   ```

2. Start services:
   ```bash
   docker-compose up -d --build
   ```

### Common Tasks

#### Add a New Venue

1. Find venue on CueScore
2. Extract `id` and `slug` from URL: `https://cuescore.com/venue/{slug}/{id}/tournaments`
3. Edit `backend/src/config/venues.rs`:
   ```rust
   VenueConfig {
       id: 12345,
       name: "New Venue Name".to_string(),
       slug: "new-venue-slug".to_string(),
   },
   ```
4. Rebuild backend: `cargo build --release`
5. Run `ingest` and `process`

#### Clear Cache

```bash
# Local
rm -rf backend/cache/*

# Docker
docker-compose exec backend rm -rf /app/cache/*
```

#### Reset Database

```bash
# Local
rm backend/data/warsaw_pool_ranking.db
cargo run -- process  # Re-initializes

# Docker
docker-compose down -v  # Removes volume
docker-compose up -d --build
```

---

## 13. Common Patterns & Conventions

### Rust Code Style

#### Small, Focused Functions
```rust
// Good: Single responsibility
fn calculate_rating(games: &[Game]) -> f64 {
    // ...
}

fn filter_games_by_period(games: &[Game], period: &str) -> Vec<Game> {
    // ...
}

// Bad: Too many responsibilities
fn calculate_and_filter_and_save_ratings(/* ... */) {
    // ...
}
```

#### Descriptive Variable Names
```rust
// Good
let player_rating_adjustment = calculate_adjustment(player_id);

// Bad
let adj = calc(p);
```

#### Error Handling with `?` Operator
```rust
use anyhow::Result;

fn process_tournament(id: i64) -> Result<()> {
    let tournament = fetch_tournament(id)?;  // Propagate error
    let games = parse_games(&tournament)?;
    save_games(&games)?;
    Ok(())
}
```

#### Logging at Appropriate Levels
```rust
log::debug!("Processing game ID {}", game.id);  // Verbose details
log::info!("Calculated ratings for 150 players");  // Important milestones
log::warn!("ADMIN_PASSWORD not set, using default");  // Warnings
log::error!("Failed to connect to database: {}", e);  // Errors
```

#### Module Organization by Feature
```
backend/src/
├── api/          # Everything related to HTTP API
├── rating/       # Everything related to rating calculation
├── database/     # Everything related to database access
```

### Angular Code Style

#### Standalone Components
```typescript
@Component({
  selector: 'app-player-card',
  standalone: true,
  imports: [CommonModule, MatCardModule],
  templateUrl: './player-card.component.html',
})
export class PlayerCardComponent {
  // ...
}
```

#### Modern Control Flow Syntax
```html
<!-- Use @if, @for, @switch -->
@if (player) {
  <div>{{ player.name }}</div>
} @else {
  <div>Loading...</div>
}

@for (player of players; track player.id) {
  <app-player-card [player]="player" />
}
```

#### Reactive Programming with RxJS
```typescript
// Good: Observable chain
getPlayers(): Observable<Player[]> {
  return this.http.get<Player[]>('/api/players').pipe(
    map(players => players.filter(p => p.games_played > 10)),
    catchError(error => {
      console.error('Failed to fetch players:', error);
      return of([]);
    })
  );
}

// Bad: Imperative with callbacks
getPlayers(callback: (players: Player[]) => void) {
  this.http.get('/api/players').subscribe(/* ... */);
}
```

#### Type Safety with TypeScript
```typescript
// Good: Explicit types
interface Player {
  id: number;
  name: string;
  rating: number;
}

function getTopPlayer(players: Player[]): Player | undefined {
  return players.sort((a, b) => b.rating - a.rating)[0];
}

// Bad: Any types
function getTopPlayer(players: any[]): any {
  // ...
}
```

### Database Patterns

#### Connection Pooling for Concurrency
```rust
// Initialize pool once
let pool = r2d2::Pool::builder()
    .max_size(10)
    .build(manager)?;

// Get connection from pool (not new connection)
let conn = pool.get()?;
```

#### Transactions for Multi-Step Operations
```rust
fn save_tournament_with_games(tournament: Tournament, games: Vec<Game>) -> Result<()> {
    let conn = pool.get()?;
    let tx = conn.transaction()?;

    // Step 1: Insert tournament
    tx.execute("INSERT INTO tournaments (...) VALUES (...)", params![...])?;

    // Step 2: Insert games
    for game in games {
        tx.execute("INSERT INTO games (...) VALUES (...)", params![...])?;
    }

    // Commit or rollback on error
    tx.commit()?;
    Ok(())
}
```

#### Index All Foreign Keys
```sql
-- Every foreign key should have an index
CREATE INDEX idx_games_first_player ON games(first_player_id);
CREATE INDEX idx_games_second_player ON games(second_player_id);
CREATE INDEX idx_games_tournament ON games(tournament_id);
```

### Error Handling

#### Backend: `anyhow::Result<T>` Everywhere
```rust
use anyhow::{Result, Context};

fn fetch_tournament(id: i64) -> Result<Tournament> {
    let response = reqwest::get(&url)
        .await
        .context("Failed to fetch tournament from CueScore")?;

    let tournament = response.json::<Tournament>()
        .await
        .context("Failed to parse tournament JSON")?;

    Ok(tournament)
}
```

#### Frontend: RxJS `catchError` Operator
```typescript
getPlayers(): Observable<Player[]> {
  return this.http.get<Player[]>('/api/players').pipe(
    catchError(error => {
      console.error('Failed to fetch players:', error);
      this.snackBar.open('Failed to load players', 'Close', { duration: 3000 });
      return of([]);  // Return empty array on error
    })
  );
}
```

#### User-Facing Error Messages
```typescript
// Backend error
if (!is_valid) {
    return Json(LoginResponse {
        success: false,
        message: Some("Invalid password".to_string()),
    });
}

// Frontend display
this.snackBar.open(
  result.error || 'Login failed',
  'Close',
  { duration: 4000 }
);
```

---

## 14. Key Design Decisions

### Why Rust for Backend?

#### Performance
- **Rating calculation** on 10,000+ games is CPU-intensive
- Bradley-Terry ML requires iterative matrix operations
- Rust's zero-cost abstractions = C-like performance with high-level code

#### Type Safety
- Compile-time guarantees prevent entire classes of bugs:
  - No null pointer dereferences
  - No data races
  - No use-after-free
- `Result<T, E>` forces error handling

#### Excellent Ecosystem
- `tokio`: Industry-leading async runtime
- `axum`: Fast, ergonomic web framework
- `rusqlite`: Zero-copy SQLite bindings
- `nalgebra`: High-performance linear algebra

#### Memory Safety Without GC
- No garbage collection pauses
- Predictable performance
- Low memory footprint

**Trade-offs**:
- Steeper learning curve
- Slower compilation times
- Smaller talent pool

### Why SQLite?

#### No Separate Database Server
- One less service to deploy and monitor
- Embedded in backend container
- Single file for backup/restore

#### Perfect for Read-Heavy Workload
- Ratings calculated periodically (not real-time)
- Most requests are reads (leaderboard queries)
- Writes are batched (full re-calculation)

#### Easy Backup
- Single file: `warsaw_pool_ranking.db`
- Copy file = complete backup
- No dump/restore process

#### Sufficient Performance
- Handles 100,000+ games easily
- Indexed queries in milliseconds
- Connection pooling for concurrency

**Trade-offs**:
- Not ideal for high-write concurrency
- Limited to single server (no distributed queries)
- Max database size ~140 TB (not a concern for this use case)

**When to Consider PostgreSQL**:
- Need real-time writes (live tournament scoring)
- Multiple backend servers (horizontal scaling)
- Complex full-text search

### Why CLI-based Processing?

#### Simpler Than Always-On Server
- No background jobs or cron within application
- Explicit command execution (clear intent)
- Easier debugging (run command, see output)

#### Explicit Control Over Refresh
- Admin decides when to ingest new data
- No automatic polling (reduces server load)
- Can schedule via cron if desired

#### Resource-Efficient
- Process runs only when needed (not idle loop)
- No memory leaks from long-running process
- Restart is just re-running command

**Trade-offs**:
- Not real-time (manual refresh required)
- Requires external scheduler (cron) for automation
- No WebSocket push updates to frontend

**When to Consider Always-On Processing**:
- Need real-time rating updates
- Live tournament scoring
- Automatic data refresh every X minutes

### Why Angular 17?

#### Modern Framework
- Standalone components (no NgModules boilerplate)
- Modern control flow syntax (`@if`, `@for`)
- Signals for reactive state (better performance)

#### Material Design Integration
- Professional UI components out of the box
- Consistent design language
- Accessibility built-in

#### TypeScript Type Safety
- Compile-time error checking
- Excellent IDE support (autocomplete, refactoring)
- Self-documenting code

**Trade-offs**:
- Larger bundle size than lighter frameworks (React, Svelte)
- Steeper learning curve than simpler frameworks
- Opinionated structure (good for teams, less flexible)

**Alternatives Considered**:
- **React**: More flexible, lighter, but requires more setup decisions
- **Vue**: Easier learning curve, but smaller ecosystem
- **Svelte**: Smallest bundle, but less mature ecosystem

### Why Local Avatar Storage?

#### Problem: Unreliable External URLs
```
Example flow with external URLs:
1. Fetch player data with avatar_url: "https://cdn.cuescore.com/avatars/abc123.jpg"
2. Display avatar in frontend: <img src="https://cdn.cuescore.com/avatars/abc123.jpg">
3. Later: CueScore changes CDN → URL breaks → broken images
```

#### Solution: Local Storage
```
1. Download avatar from CueScore
2. Resize to 3 sizes (small, medium, large)
3. Convert to WebP format
4. Store BLOB in SQLite avatars table
5. Serve via backend API: GET /api/players/{id}/avatar?size=medium
6. Frontend: <img src="/api/players/123/avatar?size=medium">
```

#### Benefits
- **Reliability**: Avatars always available (in database)
- **Performance**: No external HTTP requests, instant loading
- **Consistency**: Controlled format (WebP) and quality
- **Change Detection**: Hash-based system avoids redundant downloads

**Trade-offs**:
- Increased database size (WebP is efficient, but still storage)
- More complex implementation (download, resize, store)
- Requires admin action to refresh avatars

---

## 15. File Paths Reference

### Critical Backend Files

#### Configuration
- **`backend/src/config/settings.rs`** - Application configuration (`AppConfig`, `RatingSettings`, `AdminSettings`)
- **`backend/src/config/venues.rs`** - Venue list (hardcoded, add new venues here)
- **`backend/Cargo.toml`** - Rust dependencies and project metadata
- **`backend/Dockerfile`** - Docker image definition for backend
- **`.env.example`** - Development environment variables template
- **`.env.production.example`** - Production environment variables template

#### Database
- **`backend/src/database/mod.rs`** - Database connection pool initialization
- **`backend/src/database/schema.sql`** - SQL schema definition (CREATE TABLE statements)
- **`backend/src/database/repositories/`** - Data access layer
  - `players.rs` - Player CRUD operations
  - `games.rs` - Game CRUD operations
  - `ratings.rs` - Rating CRUD operations
  - `avatars.rs` - Avatar CRUD operations

#### Rating Algorithm
- **`backend/src/rating/bradley_terry.rs`** - Core Bradley-Terry ML implementation
- **`backend/src/rating/weighting.rs`** - Time decay calculation
- **`backend/src/rating/types.rs`** - Rating-related type definitions

#### API & HTTP
- **`backend/src/api/routes.rs`** - Axum route definitions (all endpoints)
- **`backend/src/api/handlers/admin.rs`** - Admin endpoints (login, refresh, avatars)
- **`backend/src/api/handlers/players.rs`** - Player data endpoints
- **`backend/src/api/handlers/avatars.rs`** - Avatar serving endpoints
- **`backend/src/api/client.rs`** - CueScore API client
- **`backend/src/api/models.rs`** - API request/response models (protobuf)

#### Services
- **`backend/src/services/ingestion_service.rs`** - Data collection orchestration
- **`backend/src/services/processing_service.rs`** - Rating calculation orchestration
- **`backend/src/services/avatar_service.rs`** - Avatar download & processing

#### CLI
- **`backend/src/main.rs`** - Entry point
- **`backend/src/cli.rs`** - CLI command definitions (serve, ingest, process)
- **`backend/src/lib.rs`** - Library exports

### Critical Frontend Files

#### Components
- **`frontend/src/app/components/rankings-list/`** - Main player list
  - `rankings-list.component.ts`
  - `rankings-list.component.html`
  - `rankings-list.component.scss`
- **`frontend/src/app/components/player-overlay/`** - Player details modal
  - `player-overlay.component.ts`
  - `player-overlay.component.html`
  - `player-overlay.component.scss`
- **`frontend/src/app/components/admin/`** - Admin panel
  - `admin.component.ts`
  - `admin.component.html`
  - `admin.component.scss`

#### Services
- **`frontend/src/app/services/database.service.ts`** - Backend API queries
- **`frontend/src/app/services/auth.service.ts`** - Admin authentication
- **`frontend/src/app/services/admin.service.ts`** - Admin operations (wraps auth service)

#### Models
- **`frontend/src/app/models/`** - TypeScript interfaces (generated from protobuf)

#### Configuration
- **`frontend/package.json`** - NPM dependencies and scripts
- **`frontend/angular.json`** - Angular CLI configuration
- **`frontend/tsconfig.json`** - TypeScript compiler configuration
- **`frontend/Dockerfile`** - Docker image definition for frontend

### Configuration Files

- **`docker-compose.yml`** - Docker services definition (backend, frontend)
- **`.env.example`** - Local development environment template
- **`.env.production.example`** - Production environment template
- **`Makefile`** - Convenience commands (optional, some deprecated)

### Documentation

- **`README.md`** - User-facing project documentation
- **`CLAUDE.md`** - **THIS FILE** - LLM context documentation
- **`backend/tests/README.md`** - Testing guide

---

## 16. Testing

### Backend Tests

#### Location
- **`backend/tests/`** - Integration tests
- **`backend/src/**/*_test.rs`** - Unit tests (inline with source)

#### Running Tests

```bash
# All tests
cargo test

# Specific test file
cargo test --test integration_test

# Specific test function
cargo test test_bradley_terry_calculation

# Show output
cargo test -- --nocapture

# Documentation tests
cargo test --doc
```

#### Test Structure

**Unit Tests** (inline):
```rust
// backend/src/rating/bradley_terry.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_bradley_terry() {
        let games = vec![/* ... */];
        let result = calculate_bradley_terry(&games, 500.0, 1e-6, 100);
        assert!(result.is_ok());
    }
}
```

**Integration Tests** (`backend/tests/`):
```rust
// backend/tests/integration_test.rs
use warsaw_pool_ranking::*;

#[tokio::test]
async fn test_full_ingestion_flow() {
    // Test complete flow: fetch → parse → store → calculate
}
```

#### Test Coverage

See **`backend/tests/README.md`** for detailed testing guide.

**Areas Covered**:
- Bradley-Terry rating calculation
- Time decay weighting
- New player blending
- Database operations
- API endpoint responses

### Frontend Tests

#### Framework
- **Jasmine**: Test framework
- **Karma**: Test runner
- **Chrome Headless**: Browser for tests

#### Running Tests

```bash
# Run all tests
npm test

# Run tests in watch mode
npm test -- --watch

# Generate coverage report
npm test -- --code-coverage
```

#### Test Structure

**Unit Tests** (services):
```typescript
// frontend/src/app/services/database.service.spec.ts
describe('DatabaseService', () => {
  let service: DatabaseService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [HttpClientTestingModule],
      providers: [DatabaseService]
    });
    service = TestBed.inject(DatabaseService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  it('should fetch players', () => {
    service.getPlayers('all').subscribe(players => {
      expect(players.length).toBeGreaterThan(0);
    });

    const req = httpMock.expectOne('/api/players?period=all');
    req.flush([{ id: 1, name: 'Test Player' }]);
  });
});
```

**Component Tests**:
```typescript
// frontend/src/app/components/rankings-list/rankings-list.component.spec.ts
describe('RankingsListComponent', () => {
  let component: RankingsListComponent;
  let fixture: ComponentFixture<RankingsListComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [RankingsListComponent]  // Standalone component
    });
    fixture = TestBed.createComponent(RankingsListComponent);
    component = fixture.componentInstance;
  });

  it('should display players', () => {
    component.players = [{ id: 1, name: 'Test Player', rating: 600 }];
    fixture.detectChanges();
    const compiled = fixture.nativeElement;
    expect(compiled.querySelector('.player-name').textContent).toContain('Test Player');
  });
});
```

---

## 17. Deployment

### Docker Compose Setup

#### Services

**Backend** (Rust HTTP server):
- Image: Built from `backend/Dockerfile`
- Port: 8000 (exposed)
- Environment:
  - `DATABASE_PATH=/app/data/warsaw_pool_ranking.db`
  - `RUST_LOG=info`
  - `ADMIN_PASSWORD` (from env var, default: `changeme`)
- Volumes:
  - `app_data:/app/data` (SQLite database persistence)
  - `./backend/cache:/app/cache` (cache directory, host-mounted)
- Command: `./entrypoint.sh` (runs `serve` command)

**Frontend** (Nginx + Angular SPA):
- Image: Built from `frontend/Dockerfile`
- Port: 80 (exposed)
- Serves static Angular build via Nginx
- Proxies `/api/*` requests to backend

#### Volumes

```yaml
volumes:
  app_data:
    driver: local
```

**Purpose**: Persist SQLite database across container restarts.

**Location**: Docker-managed volume (not host path).

**Backup**:
```bash
# Copy database from volume to host
docker run --rm -v warsaw-pool-ranking_app_data:/data -v $(pwd):/backup busybox cp /data/warsaw_pool_ranking.db /backup/

# Restore database from host to volume
docker run --rm -v warsaw-pool-ranking_app_data:/data -v $(pwd):/backup busybox cp /backup/warsaw_pool_ranking.db /data/
```

### Environment Variables

| Variable | Required | Default (Dev) | Default (Docker) | Description |
|----------|----------|---------------|------------------|-------------|
| `ADMIN_PASSWORD` | **Production** | `admin` | `changeme` | Admin panel password |
| `DATABASE_PATH` | No | `backend/data/warsaw_pool_ranking.db` | `/app/data/warsaw_pool_ranking.db` | SQLite file path |
| `RUST_LOG` | No | `info` | `info` | Logging level (`debug`, `info`, `warn`, `error`) |

#### Setting Environment Variables

**Local Development**:
```bash
# .env file
ADMIN_PASSWORD=admin
RUST_LOG=debug
```

**Docker Compose**:
```bash
# Shell export before docker-compose up
export ADMIN_PASSWORD=mysecurepassword
docker-compose up -d

# Or inline
ADMIN_PASSWORD=mysecurepassword docker-compose up -d
```

**Production**:
```bash
# Generate secure password
openssl rand -base64 32

# Set in systemd service file
Environment="ADMIN_PASSWORD=<generated-password>"
Environment="RUST_LOG=warn"
```

### Production Checklist

#### Before Deployment

1. **Set Strong Admin Password**:
   ```bash
   openssl rand -base64 32
   # Use output as ADMIN_PASSWORD
   ```

2. **Configure HTTPS**:
   - Set up reverse proxy (Nginx, Caddy, Traefik)
   - Obtain SSL certificate (Let's Encrypt recommended)
   - Redirect HTTP → HTTPS

3. **Set Logging Level**:
   ```bash
   RUST_LOG=warn  # or 'error' for production
   ```

4. **Verify Database Persistence**:
   - Check that `app_data` volume is created
   - Test database survives container restart

5. **Test Backup/Restore**:
   - Backup database file
   - Restore to new environment
   - Verify data integrity

#### After Deployment

6. **Monitor Logs**:
   ```bash
   docker-compose logs -f backend
   ```

7. **Set Up Automated Data Refresh** (optional):
   ```bash
   # Crontab: Run ingest daily at 2 AM
   0 2 * * * docker-compose exec -T backend ./warsaw_pool_ranking ingest && docker-compose exec -T backend ./warsaw_pool_ranking process
   ```

8. **Monitor Resource Usage**:
   ```bash
   docker stats warsaw-pool-backend warsaw-pool-frontend
   ```

### Reverse Proxy Configuration

**Example: Nginx**

```nginx
server {
    listen 80;
    server_name pool-rankings.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name pool-rankings.example.com;

    ssl_certificate /etc/letsencrypt/live/pool-rankings.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/pool-rankings.example.com/privkey.pem;

    location / {
        proxy_pass http://localhost:80;  # Frontend container
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /api/ {
        proxy_pass http://localhost:8000;  # Backend container
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Backup Strategy

#### What to Backup

1. **SQLite Database**: `app_data` volume
2. **Cache Directory**: `backend/cache/` (optional, can be regenerated)
3. **Configuration**: `.env.production`

#### Backup Script

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backups/pool-rankings"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Backup database from Docker volume
docker run --rm \
  -v warsaw-pool-ranking_app_data:/data \
  -v "$BACKUP_DIR":/backup \
  busybox cp /data/warsaw_pool_ranking.db "/backup/db_$DATE.db"

# Compress
gzip "$BACKUP_DIR/db_$DATE.db"

# Delete backups older than 30 days
find "$BACKUP_DIR" -name "db_*.db.gz" -mtime +30 -delete

echo "Backup completed: db_$DATE.db.gz"
```

**Cron** (daily at 3 AM):
```
0 3 * * * /usr/local/bin/pool-rankings-backup.sh
```

---

## 18. Future Enhancements

### Known Limitations

#### Authentication & Security
- **No Rate Limiting**: Admin endpoints not protected against brute force
- **No Session Tokens**: Password used directly as Bearer token (not ideal for production)
- **No User Management**: Single admin user only
- **No Role-Based Access**: Everyone is admin or not authenticated

#### Real-Time Updates
- **Manual Refresh**: Admin must trigger data refresh manually
- **No WebSocket**: Frontend doesn't receive live updates (must reload page)
- **No Live Scoring**: Tournaments not updated in real-time

#### Features
- **No Player Profiles**: Players can't claim/manage their profiles
- **No Advanced Statistics**: Only ratings shown, no head-to-head, streaks, etc.
- **No Player Comparison**: Can't compare two players side-by-side
- **No Tournament Filtering**: Can't view ratings for specific tournaments

### Potential Improvements

#### Short-Term (Low Effort)

1. **Rate Limiting on Admin Endpoints**
   - Library: `tower-governor` or `axum-rate-limit`
   - Limit: 5 login attempts per minute per IP
   - Benefit: Prevents brute force attacks

2. **Automated Data Refresh**
   - Cron job to run `ingest` + `process` daily
   - Benefit: Always-fresh data without admin action

3. **More Rating Periods**
   - Add: 6 months, 9 months, 18 months
   - Benefit: Better granularity for recent form

4. **Export Ratings to CSV**
   - Endpoint: `GET /api/ratings/export?period=all`
   - Benefit: Offline analysis, integration with other tools

#### Medium-Term (Moderate Effort)

5. **JWT Token Authentication**
   - Replace password-as-bearer-token with proper JWT
   - Add token expiration and refresh
   - Benefit: Industry-standard security

6. **Player Head-to-Head Statistics**
   - Endpoint: `GET /api/players/{id1}/vs/{id2}`
   - Show: Win rate, recent games, rating difference over time
   - Benefit: Deeper insights

7. **Advanced Filtering**
   - Filter by: Venue, tournament, date range
   - Frontend: Filter chips in Material UI
   - Benefit: More targeted analysis

8. **Player Comparison View**
   - Compare 2-4 players side-by-side
   - Show: Rating charts, stats, head-to-head
   - Benefit: Better competitive analysis

#### Long-Term (High Effort)

9. **User Accounts & Player Claiming**
   - Players can create accounts and claim profiles
   - Verify via email or tournament participation
   - Benefit: Player engagement, profile customization

10. **Real-Time Tournament Scoring**
    - WebSocket connection for live updates
    - Admin can submit scores in real-time
    - Ratings updated immediately
    - Benefit: Live leaderboards during tournaments

11. **Mobile App**
    - Native iOS/Android app or PWA
    - Push notifications for rating changes
    - Benefit: Better mobile UX, engagement

12. **Advanced Analytics Dashboard**
    - Rating distribution histogram
    - Activity heatmap (games per week)
    - Venue popularity over time
    - Benefit: Insights for tournament organizers

13. **Elo-Based Prediction Model**
    - Predict match outcomes based on ratings
    - Show win probability before games
    - Benefit: Fairer seeding, betting odds

14. **API for Third-Party Integration**
    - Public REST API with rate limiting
    - API keys for developers
    - Benefit: Tournament software can integrate

### Architecture Changes

#### If Scaling to 100K+ Games

- **Consider PostgreSQL**: Better write concurrency, full-text search
- **Add Redis**: Cache frequently accessed data (top 100 players)
- **Separate Read/Write DBs**: Read replicas for leaderboard queries

#### If Real-Time Updates Required

- **WebSocket Server**: Axum with `axum-websocket`
- **Message Queue**: Redis Pub/Sub or RabbitMQ
- **Event Sourcing**: Store game events, recalculate ratings incrementally

#### If Multi-Region Deployment

- **CDN for Frontend**: Cloudflare, AWS CloudFront
- **Geo-Replicated DB**: PostgreSQL with streaming replication
- **Load Balancer**: Nginx, HAProxy, or cloud LB

---

## Appendix: Quick Reference

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ADMIN_PASSWORD` | `admin` (dev) / `changeme` (docker) | Admin panel password |
| `DATABASE_PATH` | `backend/data/warsaw_pool_ranking.db` | SQLite file path |
| `RUST_LOG` | `info` | Logging level |

### CLI Commands

```bash
# Backend
cargo run -- serve --port 8000    # Start HTTP server
cargo run -- ingest                # Fetch fresh data
cargo run -- process               # Calculate ratings

# Docker
docker-compose up -d --build       # Build and start services
docker-compose exec backend ./warsaw_pool_ranking ingest
docker-compose logs -f backend

# Frontend
npm start                          # Dev server (localhost:4200)
npm run build                      # Production build
```

### API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/players?period={period}` | List players with ratings |
| GET | `/api/players/{id}` | Player details |
| GET | `/api/players/{id}/avatar?size={size}` | Player avatar |
| POST | `/api/admin/login` | Admin login |
| POST | `/api/admin/refresh` | Trigger data refresh |
| POST | `/api/admin/refresh-avatars` | Refresh avatars |

### Rating Thresholds

| Games Played | Confidence Level |
|--------------|------------------|
| 0-9 | Unranked |
| 10-49 | Provisional |
| 50-99 | Emerging |
| 100+ | Established |

### Database Tables

- `players` - Player profiles
- `tournaments` - Tournament metadata
- `games` - Individual games (expanded from matches)
- `ratings` - Calculated ratings (multiple periods)
- `avatars` - Local avatar storage (WebP, 3 sizes)

---

**End of CLAUDE.md**

For questions, issues, or contributions, see the main [README.md](README.md).
