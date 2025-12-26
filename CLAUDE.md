# Warsaw Pool Rankings - LLM Context Documentation

**Last Updated:** 2025-12-26
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
8. [Common Patterns & Conventions](#8-common-patterns--conventions)
9. [Recent Refactoring](#9-recent-refactoring)
10. [Development Workflows](#10-development-workflows)

---

## 1. Project Overview

Warsaw Pool Rankings is a skill-based rating system for pool players in Warsaw, Poland. It collects game data from CueScore and calculates player ratings using Bradley-Terry ML algorithm.

**Key Features:**
- **Bradley-Terry ML Rating**: 100 points = 2:1 winning odds
- **Time Decay**: 3-year half-life (recent games weighted more)
- **Active Ranking**: 6-month activity filter (default view)
- **Rivalry Analysis**: "Nemesis" (lowest win %) and "Bunny" (highest win %) stats
- **Multiple Periods**: Active, All-time, 1y, 2y
- **Confidence Levels**: Unranked, Provisional, Emerging, Established
- **Local Avatars**: WebP format with hash-based change detection
- **Admin Panel**: Password-protected data refresh

**Data Source:** [CueScore](https://cuescore.com) tournament management platform

---

## 2. Code Quality Standards

> **CRITICAL**: This project adheres to the **highest coding standards**. Every change must improve code quality.

### Core Principles

**Rust:**
- **Repository Pattern**: ALL database access MUST go through repositories (NO SQL in services/handlers)
- **Service Layer**: Business logic in services (orchestrators + specialized processors)
- **Error Handling**: Use `AppError` enum, `?` operator (never `unwrap_or()` to hide errors)
- **Performance**: Batch queries to avoid N+1 problems
- Modern async/await, `impl Trait`, type system guarantees
- `cargo clippy` + `cargo fmt` - address ALL warnings
- Delete dead code (no `#[allow(dead_code)]`)

**Angular:**
- Angular 17+ standalone components (NO NgModules)
- **OnPush + Signals**: `ChangeDetectionStrategy.OnPush` + `input<T>()` for inputs
- **Modern Control Flow**: `@if`, `@for`, `@switch` (NO `*ngIf`, `*ngFor`)
- Typed forms, strict TypeScript

### SOLID Principles

- **SRP**: One function = one responsibility, one file = one concept
- **OCP**: Open for extension (config-driven design)
- **DIP**: Dependency injection via `AppState`

### Code Metrics

- **Functions**: Max 30 lines, max 4 parameters (use structs if more)
- **Files**: Max 300 lines per file
- **Nesting**: Max 2 levels
- **DRY**: 3 uses = extract to function

### Zero Warnings Standard

**CRITICAL**: Zero clippy warnings at all times.

Common patterns:
- **Parameter Reduction**: Use helper structs for >7 parameters (see `UpsertAvatarParams`)
- **Method Names**: Avoid conflicts with standard traits (`from_str()` → `parse()`)
- **Default Trait**: Add for structs with parameterless `new()`

```rust
// Parameter reduction pattern
pub struct UpsertAvatarParams<'a> {
    pub player_cuescore_id: i64,
    pub size: &'a str,
    pub image_data: &'a [u8],
    pub source_url: &'a str,
    pub source_url_hash: &'a str,
    pub width: i32,
    pub height: i32,
}
```

---

## 3. Architecture Overview

```
┌─────────────────────────────────────────┐
│  Angular Frontend (Port 80)             │
│  - Rankings table, overlays, admin      │
│  - Signals & OnPush                     │
└──────────┬──────────────────────────────┘
           │ HTTP /api/*
           ▼
┌─────────────────────────────────────────┐
│  Rust Backend (Port 8000)               │
│  - Axum HTTP + Repository Pattern       │
│  - CLI (tournaments, rankings, avatars) │
│  - Bradley-Terry rating calculation     │
└──────────┬──────────────────────────────┘
           │ rusqlite (r2d2 pool)
           ▼
┌─────────────────────────────────────────┐
│  SQLite (warsaw_pool_ranking.db)        │
│  - players, games, ratings, avatars     │
└─────────────────────────────────────────┘
```

**Data Flow:** Scrape CueScore → Calculate ratings → Store in SQLite → Serve via API → Display with Signals

**Deployment:** Docker Compose (frontend: Nginx, backend: Rust HTTP server + SQLite volume)

---

## 4. Technology Stack

### Backend: Rust 1.73+
- `tokio`, `axum`, `rusqlite` (bundled), `r2d2`
- `reqwest`, `serde`, `anyhow`, `thiserror`
- `nalgebra` (rating calc), `clap` (CLI)
- `scraper`, `image/webp`, `prost` (protobuf)

### Frontend: Angular 17
- Standalone components, Signals, Material Design
- `Chart.js` + `ng2-charts`, RxJS

### Database: SQLite
- Embedded, r2d2 pool (10 connections)
- Perfect for read-heavy workload

---

## 5. Backend Architecture

### Key Repositories (Data Access Layer)
- `PlayerRepository` - CRUD, ranked lists, last played updates
- `GameRepository` - CRUD, H2H matches, **batch queries** (avoid N+1)
- `TournamentRepository`, `RatingRepository`, `AvatarRepository`

**Batch Query Pattern:**
```rust
// Single query returns HashMap for O(1) lookups
let match_counts = GameRepository::count_matches_for_players(&mut conn, &player_ids)?;
let matches = match_counts.get(&player_id).copied().unwrap_or(0);
```

### Service Layer (Business Logic)

**Orchestrators:**
- `processing.rs` - Rating calculation workflow
- `ingestion.rs` - Data collection workflow

**Specialized Processors (SRP):**
- `tournament_processor.rs` - Date parsing, doubles detection
- `game_processor.rs` - Game filtering, team player detection
- `rating_processor.rs` - Rating calculation & persistence
- `avatar_processor.rs` - Download, resize, WebP encoding
- `player_service.rs` - Player detail construction (DRY)

### Configuration-Driven Design (OCP)

All configurable behavior in `config/settings.rs`:

```rust
pub struct AppConfig {
    pub rating: RatingSettings,
    pub avatar: AvatarSettings,
    pub tournament_processing: TournamentProcessingSettings,
}
```

Services receive config via constructor (dependency injection).

### CLI Commands

```bash
warsaw-pool-rankings tournaments refresh  # Fetch from CueScore
warsaw-pool-rankings rankings refresh     # Calculate ratings
warsaw-pool-rankings avatars refresh      # Download/update avatars
warsaw-pool-rankings database reset       # Drop all tables
warsaw-pool-rankings serve --port 8000    # Start HTTP server
```

**Auto-dependency resolution**: `avatars refresh` on empty DB auto-runs: tournaments → rankings → avatars

---

## 6. Frontend Architecture

### Angular 17 Patterns

**Signals + OnPush:**
```typescript
@Component({
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PlayerOverlayComponent {
  player = signal<PlayerDetail | null>(null);
  matches = input.required<HeadToHeadMatch[]>();  // Signal input
}
```

**Modern Control Flow:**
```html
@if (loading()) {
  <app-skeleton />
} @else {
  @for (match of matches(); track match.date) {
    <div>{{ match.opponentName }}</div>
  }
}
```

### Key Components
- `RatingTypeSelectorComponent` - Reusable ranking category toggle
- `SkeletonComponent` - Pulsing loading placeholders
- `AvatarComponent` - Reusable avatar with signal inputs (fixed: `input<T>()` not `@Input()`)

---

## 7. Database Schema

### `players` Table

```sql
CREATE TABLE players (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cuescore_id INTEGER UNIQUE,
    name TEXT NOT NULL,
    avatar_url TEXT,
    last_played TEXT,  -- For "Active" filter
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_players_last_played ON players(last_played);
```

**Performance:** Index enables O(log n) active player filtering

**Other Tables:** `tournaments`, `games`, `ratings`, `avatars` (see `backend/src/database/schema.sql`)

---

## 8. Common Patterns & Conventions

### Repository Pattern Usage

✅ **CORRECT:**
```rust
let players = PlayerRepository::list_all(&mut conn)?;
PlayerRepository::update_player_last_played(&mut conn, player_id, date)?;
```

❌ **NEVER:**
```rust
conn.execute("UPDATE players...", params![...])?;  // NO SQL in services!
```

### Avoiding N+1 Problems

✅ **Batch query with HashMap:**
```rust
let player_ids: Vec<i32> = rows.iter().map(|r| r.player_id).collect();
let match_counts = GameRepository::count_matches_for_players(&mut conn, &player_ids)?;
let matches = match_counts.get(&row.player_id).copied().unwrap_or(0);
```

❌ **Query inside iterator** (violates borrow rules + N+1)

### Error Handling

✅ **Propagate with `?`:**
```rust
let counts = GameRepository::count_matches_for_players(&mut conn, &ids)
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
```

❌ **Silent suppression:**
```rust
.unwrap_or(0)  // Hides real errors!
```

### Repository Helper Patterns

**When methods exceed 60 lines:**

1. **WHERE Clause Builder** - Extract SQL construction to `WhereClauseBuilder` struct
2. **Row Mapping** - Extract to `map_row_to_player()` function (DRY)
3. **Early Returns** - Flatten control flow (max 2 nesting levels)

### Connection Management

✅ Get once, pass to repositories:
```rust
let mut conn = state.pool.get()?;
let players = PlayerRepository::list_ranked_players(&mut conn, &filter)?;
let counts = GameRepository::count_matches_for_players(&mut conn, &ids)?;
```

---

## 9. Recent Refactoring

### Phase 1: Repository Pattern Migration (Dec 2025)

**Achievements:**
- ✅ Eliminated ~400 lines of duplicated SQL
- ✅ Fixed N+1 problems (batch queries + HashMap lookups)
- ✅ Created 5 repositories, removed all legacy DB modules
- ✅ Added `idx_players_last_played` index (O(log n) filtering)
- ✅ Proper error propagation (no silent `unwrap_or()`)

### Phase 2: Code Quality & SOLID (Dec 2025)

**Key Improvements:**

1. **Service Layer Refactoring (SRP)**
   - Split `processing.rs` (375 lines) into specialized processors
   - `tournament_processor.rs` (89 lines), `game_processor.rs` (83 lines), `rating_processor.rs` (73 lines)
   - Main orchestrator: 200 lines, each processor <100 lines

2. **Configuration-Driven (OCP)**
   - `AvatarSettings`, `TournamentProcessingSettings` in `config/settings.rs`
   - Dependency injection via `AppConfig`

3. **DRY Improvements**
   - Extracted `player_service::build_player_detail()` (~40 lines eliminated)

4. **Repository Helpers**
   - `WhereClauseBuilder`, `map_row_to_player()`, early returns
   - Reduced `list_ranked_players()` from 87 to 60 lines

5. **Zero Warnings**
   - `UpsertAvatarParams` (8 params → struct)
   - Renamed `from_str()` → `parse()`
   - Added `Default` traits

**Metrics:**
- `processing.rs`: 375 → 200 lines (-47%)
- `player_repository.rs`: 141 → 109 lines (-23%)
- Total: ~200 lines eliminated

**Quality Achieved:**
- ✅ Zero clippy warnings (`-D warnings`)
- ✅ All files <300 lines (orchestrators <200)
- ✅ All functions <30 lines
- ✅ Max 4 parameters (or structs)
- ✅ Max 2 nesting levels
- ✅ SOLID + DRY compliance

---

## 10. Development Workflows

### Protobuf Generation

If you modify `proto/api.proto`:
```bash
cd frontend
npm run proto:generate:local  # Regenerates frontend/src/app/models/api.ts
```

Backend models regenerate automatically during `cargo build` via `build.rs`.

### Backend Build Verification

```bash
docker-compose build backend  # Verifies protoc is available
```

### File Organization & Git Strategy

**Data files tracked in git** (unlike typical apps):
- `backend/data/warsaw_pool_ranking.db` (~50 MB)
- `backend/cache/raw/*.json` (~1500 files, ~30 MB)

**Why:** CueScore scraping is slow, portability, historical preservation

**Trade-off:** Larger repo (~80 MB) but one-command setup

**CRITICAL:** Database filename is **singular**: `warsaw_pool_ranking.db` (NOT `warsaw_pool_rankings.db`)

### Future Enhancements

- Database migrations (sqlx-cli/refinery)
- CI/CD protobuf sync
- Venue leaderboards
- PWA support
- Repository trait abstraction for testing
