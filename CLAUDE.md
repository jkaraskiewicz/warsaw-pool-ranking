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
- **Repository Pattern**: All database access must go through `GameRepository` or `PlayerRepository`.
- **Service Layer**: Business logic lives in `PlayerService` and other services.
- **Error Handling**: Use `AppError` enum for centralized error handling.
- Prefer `?` operator over manual error handling
- Use `impl Trait` for return types where appropriate
- Leverage type system for compile-time guarantees
- Modern async/await (not manual futures)
- Use `cargo clippy` and address ALL warnings
- Format with `cargo fmt`

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
│   ├── database/                # SQLite interaction
│   │   ├── mod.rs               # Database connection & initialization
│   │   ├── schema.sql           # SQL schema definition
│   │   ├── repositories/        # Data access layer
│   │   │   ├── game_repository.rs
│   │   │   └── player_repository.rs
│   │   ├── models.rs            # Database Entity models
│   │   └── ...                  # Legacy/Migration modules
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

#### 1. Repository Pattern

All raw SQL queries are encapsulated in `repositories/`:

```rust
// backend/src/database/repositories/player_repository.rs
pub struct PlayerRepository;
impl PlayerRepository {
    pub fn list_ranked_players(...) -> Result<Vec<PlayerWithRating>> {
        // SQL query execution
    }
}
```

**Why**: Decouples API handlers from database details, makes testing easier.

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

### Updated `players` Table

```sql
CREATE TABLE players (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cuescore_id INTEGER UNIQUE,
    name TEXT NOT NULL,
    avatar_url TEXT,
    last_played TEXT, -- NEW: Track last activity for "Active" filter
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

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

## 18. Future Enhancements

- **Database Migrations System**: Replace `schema.sql` resets with a proper migration tool (e.g., `sqlx-cli` or `refinery`) to handle schema evolution without data loss.
- **Automated Protobuf Sync**: Add CI/CD step to ensure frontend models are always in sync with proto definitions.
- **Venue Leaderboards**: "King of the Hill" stats per venue.
- **PWA Support**: Make the app installable on mobile devices.