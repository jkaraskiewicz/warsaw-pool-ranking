# Warsaw Pool Ranking - Rust Backend Implementation Plan

## Executive Summary

This plan details the step-by-step implementation strategy for completing the `backend-rust/` project by leveraging the AI-generated reference (`backend-rust-ai/`) while preserving the user's architectural preferences from the Python implementation (`backend/`).

**Key Insights:**
- User has a working CLI scaffolding (`serve`, `ingest`, `process` commands)
- AI reference has complete, working implementations of all critical features
- Python backend demonstrates the user's preferred narrative, step-by-step orchestration style
- Critical gotchas are well-documented in `CRITICAL_IMPLEMENTATION_NOTES.md`

**Strategy:** Adapt AI implementation to user's style, focusing on clarity, logging, and the 4-step orchestration pattern.

---

## Part 1: Code Mapping & Reuse Analysis

### 1.1 What Can Be Adapted Directly from backend-rust-ai/

**High-Confidence Direct Adaptations (90%+ reusable):**

1. **Bradley-Terry MM Algorithm** (`backend-rust-ai/src/rating.rs`)
   - Lines 123-172: The MM algorithm is mathematically pure - can copy directly
   - Only needs: Change field names from `player1_id/player2_id` to match user's Game struct
   - User's model uses `first_player_id/second_player_id`, AI uses `player1_id/player2_id`
   - Algorithm is battle-tested, works at scale (32K+ players in 8 seconds)

2. **Web Scraper** (`backend-rust-ai/src/scraper.rs`)
   - Nearly identical to Python implementation patterns
   - Already uses proper logging (tracing crate)
   - Rate limiting correctly implemented
   - Needs: Add more narrative logging to match Python's step-by-step style

3. **Two-Tier Cache System** (`backend-rust-ai/src/cache.rs`)
   - Simple, clean implementation
   - Needs: Extension to support raw/ and parsed/ subdirectories
   - Currently single-level, needs to become two-tier like Python

**Moderate Adaptation Needed (50-70% reusable):**

4. **Data Models** (`backend-rust-ai/src/models.rs`)
   - CRITICAL ISSUE: AI models have `player1_score/player2_score` fields
   - User's Python implementation shows matches MUST be expanded to individual games
   - Fix: Remove score fields, ensure Game struct only has winner_id
   - Confidence levels match exactly - can reuse enum

5. **CueScore API Client** (`backend-rust-ai/src/api.rs`)
   - Structure is good, but parsing is incomplete (marked TODO)
   - Needs: Complete JSON parsing based on Python parser logic
   - Needs: Match-to-games expansion logic (7-5 → 12 game records)

**Cannot Reuse (Python-specific logic):**

6. **Parser Logic** (`backend/app/data/parser.py`)
   - Python implementation has critical match-to-games expansion (lines 259-286)
   - Must be reimplemented in Rust, not just adapted
   - This is THE most critical piece - gets the data model right

### 1.2 Critical Architectural Differences to Address

**Issue 1: Game Data Model**
```rust
// ❌ AI Implementation (WRONG - stores match scores)
pub struct Game {
    pub player1_score: i32,  // 7
    pub player2_score: i32,  // 5
}

// ✅ User's Model (CORRECT - individual game record)
pub struct Game {
    pub first_player_id: i64,
    pub second_player_id: i64,
    pub winner_id: i64,  // Either first_player_id or second_player_id
    // NO SCORES!
}
```

**Issue 2: Orchestration Style**
- AI: Single monolithic main with TODOs
- Python: Clear 4-step orchestration with narrative logging
- Solution: Implement orchestration in `lib.rs` functions, call from CLI

**Issue 3: Logging & Progress**
- AI: Uses `tracing` crate (good!)
- Python: Rich narrative logging ("Step 1:", "Found X tournaments", progress bars)
- Solution: Keep tracing, but add Python's narrative style

---

## Part 2: Implementation Order & Dependencies

### Phase 1: Foundation (Data Models & Types) - Day 1

**Goal:** Get data structures right FIRST. Everything depends on this.

**Tasks:**
1. Fix `domain/models.rs` Game struct
   - Remove score fields
   - Ensure only `first_player_id`, `second_player_id`, `winner_id`
   - Add `weight: f64` field for time decay
   - Match database schema exactly

2. Create missing types
   - Parsed tournament structure for cache
   - API response types for CueScore
   - Ensure CueScore IDs are Strings (not i64) - they're "101", "102", not integers

3. Add database models (Diesel ORM)
   - Schema matches `database/schema.sql` exactly
   - Use `diesel` codegen to generate from existing schema
   - Ensure foreign keys use database IDs, not CueScore IDs

**Validation:**
- Compare Game struct to database schema line-by-line
- Confirm winner_id constraint logic is enforceable
- No compilation errors

### Phase 2: Two-Tier Caching System - Day 1-2

**Goal:** Enable fast development iteration and respect API rate limits.

**Tasks:**
1. Extend `cache/mod.rs` to support two tiers
   ```rust
   pub struct TwoTierCache {
       raw_cache: Cache,      // cache/raw/
       parsed_cache: Cache,   // cache/parsed/
   }
   ```

2. Implement cache check order (critical!)
   - Check `parsed/tournaments.json` FIRST
   - If miss, check `raw/{tournament_id}.json` for each tournament
   - If miss, fetch from API and save to raw cache
   - After processing all, save to parsed cache

3. Add cache invalidation strategy
   - Raw cache: NEVER auto-delete
   - Parsed cache: Manual deletion only (file presence = skip everything)

**Validation:**
- Run fetch twice - second run should be instant (using parsed cache)
- Delete parsed cache, run again - should use raw cache (faster than API)
- Delete all caches, run again - should fetch from API

### Phase 3: Web Scraper (Tournament Discovery) - Day 2

**Goal:** Discover tournament IDs from venue pages.

**Tasks:**
1. Adapt `backend-rust-ai/src/scraper.rs`
   - Already 90% complete
   - Add progress logging: "Scraping page X/Y", "Found N tournaments on page X"
   - Return `HashSet<String>` (tournament IDs as strings)

2. Integrate with CLI
   - Create `fetchers/venue_scraper.rs` module
   - Call from `handle_ingest()` function
   - Load venues from config (JSON file or hardcoded list)

3. Add venue database upsert
   - Save discovered venues to database
   - Track which venues have been scraped

**Validation:**
- Run scraper on one venue, verify correct tournament IDs
- Check logging shows clear progress
- Verify rate limiting (1 req/sec)

### Phase 4: CueScore API Client & Parser - Day 2-3

**Goal:** Fetch tournament data and expand matches to games.

**Tasks:**
1. Complete API client (`api/cuescore_client.rs`)
   - Implement JSON parsing for tournament response
   - Extract: tournament info, participants, matches
   - Handle multiple date formats (CRITICAL_IMPLEMENTATION_NOTES.md, lines 250-277)

2. **CRITICAL:** Implement match-to-games expansion
   - Python reference: `backend/app/data/parser.py`, lines 259-286
   - For match with score 7-5:
     - Create 7 game records with `winner_id = player_a_id`
     - Create 5 game records with `winner_id = player_b_id`
   - This is NOT optional - it's the core of the data model

3. Implement discipline filtering
   - Exclude: snooker, pyramid, piramida, russian pyramid, russian pool
   - Case-insensitive substring matching
   - Return `None` for excluded tournaments (don't process)

4. Create parsed data structure
   ```rust
   pub struct ParsedTournament {
       pub tournament_info: TournamentInfo,
       pub participants: Vec<ParticipantInfo>,
       pub games: Vec<GameRecord>,  // Already expanded!
   }
   ```

**Validation:**
- Fetch one tournament, verify match 7-5 creates 12 game records
- Verify snooker tournament returns None
- Check all participants are extracted (deduplicated)
- Confirm played_at timestamps parse correctly

### Phase 5: Database Operations - Day 3-4

**Goal:** Store parsed data in PostgreSQL.

**Tasks:**
1. Set up Diesel ORM
   - Generate schema from `database/schema.sql`
   - Create connection pool
   - Add to Cargo.toml (already present)

2. Implement upsert operations (Python reference: `weekly_update.py`, lines 403-497)
   - `upsert_venue()`: Insert or update venue
   - `upsert_player()`: Insert or update player by cuescore_id
   - `upsert_tournament()`: Insert or update, return database ID
   - `insert_game()`: Insert game record

3. **CRITICAL:** CueScore ID → Database ID mapping
   - Players/tournaments use auto-increment IDs (1, 2, 3...)
   - Games table foreign keys use database IDs, NOT CueScore IDs
   - Must query: `SELECT id FROM players WHERE cuescore_id = ?`

4. Implement `handle_ingest()` orchestration
   ```rust
   pub fn handle_ingest() -> Result<()> {
       // Step 1: Discover tournaments
       // Step 2: Fetch tournament data (with caching)
       // Step 3: Update database
   }
   ```

**Validation:**
- Insert one tournament, verify all tables populated
- Check foreign keys resolve correctly (database IDs, not CueScore IDs)
- Verify duplicates are handled (upsert, not re-insert)
- Confirm 7-5 match creates 12 rows in games table

### Phase 6: Time Decay Calculation - Day 4

**Goal:** Calculate game weights based on recency.

**Tasks:**
1. Create `rating/time_decay.rs` module
   ```rust
   const HALF_LIFE_DAYS: f64 = 1095.0;  // 3 years
   const LAMBDA: f64 = 0.000633;  // ln(2) / 1095
   
   pub fn calculate_weight(played_at: DateTime<Utc>, reference_date: DateTime<Utc>) -> f64 {
       let days_ago = (reference_date - played_at).num_days() as f64;
       (-LAMBDA * days_ago).exp()
   }
   ```

2. Add weight field to Game queries
   - Fetch all games from database
   - Calculate weight for each based on `played_at`
   - Pass weighted games to rating calculator

**Validation:**
- Game today: weight = 1.0
- Game 1095 days ago (3 years): weight ≈ 0.50
- Game 2190 days ago (6 years): weight ≈ 0.25

### Phase 7: Bradley-Terry MM Algorithm - Day 4-5

**Goal:** Calculate player ratings from weighted game data.

**Tasks:**
1. Adapt `backend-rust-ai/src/rating.rs`
   - Copy MM algorithm (lines 123-172) - it's mathematically pure
   - Fix field names to match user's Game struct
   - Change `player1_id/player2_id` to `first_player_id/second_player_id`

2. Update comparison matrix logic
   - Python shows games have winner_id, not scores
   - Determine winner by checking `game.winner_id == game.first_player_id`
   - If true, player A won; else player B won

3. Create `rating/calculator.rs` module
   - Wrap MM algorithm
   - Add logging: "Calculating ratings for N games", "Found M unique players"
   - Return ratings in ML scale (will blend with starter rating next)

**Validation:**
- Run on small dataset (100 games, 20 players): completes in < 1 second
- Run on full dataset (342K games, 32K players): completes in < 30 seconds
- Verify convergence logging shows iterations count

### Phase 8: Confidence Levels & Rating Blending - Day 5

**Goal:** Apply new player blending and assign confidence levels.

**Tasks:**
1. Create `rating/confidence.rs` module
   ```rust
   pub fn get_confidence_level(games_played: usize) -> ConfidenceLevel {
       match games_played {
           0..=9 => Unranked,
           10..=49 => Provisional,
           50..=199 => Emerging,
           _ => Established,
       }
   }
   
   pub fn blend_rating(ml_rating: f64, games_played: usize) -> f64 {
       if games_played >= 200 {
           return ml_rating;
       }
       let starter_weight = (200 - games_played) as f64 / 200.0;
       starter_weight * 500.0 + (1.0 - starter_weight) * ml_rating
   }
   ```

2. Count games per player
   - Query database: how many games each player played
   - Use for both confidence level and blending

3. Save ratings to database
   - Update `ratings` table with blended ratings
   - Set confidence_level, games_played, total_wins, total_losses
   - Calculate win/loss stats from game records

**Validation:**
- Player with 25 games, ML rating 623.45: blended ≈ 515.43
- Player with 200+ games: blended = ML rating (no adjustment)
- Confidence levels match Python implementation

### Phase 9: Orchestration & CLI Integration - Day 5-6

**Goal:** Wire everything together in user's narrative style.

**Tasks:**
1. Implement `handle_ingest()` (lib.rs)
   ```rust
   pub fn handle_ingest() -> Result<()> {
       log::info!("=".repeat(60));
       log::info!("STARTING DATA INGESTION");
       log::info!("=".repeat(60));
       
       // Step 1: Discover tournaments
       log::info!("Step 1: Discovering tournaments from venues");
       let tournament_ids = discover_tournaments(venues)?;
       log::info!("Discovery complete: {} unique tournaments", tournament_ids.len());
       
       // Step 2: Fetch tournament data
       log::info!("Step 2: Fetching {} tournaments", tournament_ids.len());
       let tournaments_data = fetch_tournaments(tournament_ids)?;
       
       // Step 3: Update database
       log::info!("Step 3: Updating database with {} tournaments", tournaments_data.len());
       update_database(tournaments_data)?;
       
       log::info!("=".repeat(60));
       log::info!("DATA INGESTION COMPLETE");
       log::info!("=".repeat(60));
       Ok(())
   }
   ```

2. Implement `handle_process()` (lib.rs)
   ```rust
   pub fn handle_process() -> Result<()> {
       log::info!("=".repeat(60));
       log::info!("CALCULATING RATINGS");
       log::info!("=".repeat(60));
       
       // Step 4: Calculate ratings
       log::info!("Step 4: Running Bradley-Terry ML optimization");
       calculate_and_save_ratings()?;
       
       log::info!("=".repeat(60));
       log::info!("RATING CALCULATION COMPLETE");
       log::info!("=".repeat(60));
       Ok(())
   }
   ```

3. Implement `handle_serve()` (stub for now)
   - Log: "Starting server on port X"
   - Return `Ok(())` with TODO comment
   - Full API server is Phase 10

**Validation:**
- Run `cargo run -- ingest`: See 3-step narrative logging
- Run `cargo run -- process`: See rating calculation with progress
- Logs match Python's style and clarity

### Phase 10: API Server (Optional - Post-MVP) - Day 7+

**Goal:** REST API for frontend to query ratings.

**Tasks:**
1. Add axum web framework to Cargo.toml
2. Create routes:
   - GET /api/players - List all players with ratings
   - GET /api/players/:id - Get player details
   - GET /api/leaderboard - Top players by rating
3. Implement in `handle_serve()`

**Note:** This can be deferred - frontend can read database directly initially.

---

## Part 3: Critical Algorithm Implementations

### MM Algorithm Location & Structure

**Module:** `src/rating/mm_algorithm.rs`

**Rationale:**
- Separate from confidence/blending logic
- Pure mathematical algorithm - easy to test
- Can be swapped for other algorithms later

**Interface:**
```rust
pub struct MMAlgorithm {
    convergence_tolerance: f64,
    max_iterations: usize,
}

impl MMAlgorithm {
    pub fn calculate_log_ratings(
        &self,
        n_players: usize,
        comparison_matrix: &Array2<f64>,
        wins: &Array1<f64>,
    ) -> Array1<f64> {
        // Hunter (2004) MM algorithm
        // Adapted from backend-rust-ai/src/rating.rs, lines 123-172
    }
}
```

### Rating Calculator Structure

**Module:** `src/rating/calculator.rs`

**Responsibilities:**
1. Build player index (database ID → array index mapping)
2. Build comparison matrix and wins vector from games
3. Call MM algorithm
4. Convert log ratings to actual ratings
5. Return `HashMap<i64, f64>` (player_id → ml_rating)

**Interface:**
```rust
pub struct RatingCalculator {
    mm_algorithm: MMAlgorithm,
}

impl RatingCalculator {
    pub fn calculate_ratings(
        &self,
        games: &[WeightedGame],
    ) -> Result<HashMap<i64, f64>> {
        // Build comparison data
        // Run MM algorithm
        // Convert to ratings
    }
}

pub struct WeightedGame {
    pub first_player_id: i64,
    pub second_player_id: i64,
    pub winner_id: i64,
    pub weight: f64,
}
```

### Time Decay Integration

**Where:** Before passing games to rating calculator

**Flow:**
```rust
// In handle_process()

// 1. Fetch games from database
let games_from_db: Vec<Game> = fetch_all_games()?;

// 2. Calculate weights
let reference_date = Utc::now();
let weighted_games: Vec<WeightedGame> = games_from_db
    .into_iter()
    .map(|game| {
        let weight = calculate_weight(game.played_at, reference_date);
        WeightedGame {
            first_player_id: game.first_player_id,
            second_player_id: game.second_player_id,
            winner_id: game.winner_id,
            weight,
        }
    })
    .collect();

// 3. Calculate ratings
let ml_ratings = calculator.calculate_ratings(&weighted_games)?;

// 4. Apply blending and confidence levels
// 5. Save to database
```

**Critical:** Time decay happens BEFORE Bradley-Terry, not after!

---

## Part 4: Critical Files for Implementation

### Must-Read Files (Priority Order)

**Priority 1 - Data Model Understanding:**

1. `/Users/jay/development/projects/ai/warsaw-pool-ranking/CRITICAL_IMPLEMENTATION_NOTES.md`
   - Lines 7-40: Match vs Game records (THE most critical gotcha)
   - Lines 148-180: Database foreign keys (CueScore ID → DB ID mapping)
   - Lines 206-244: Bradley-Terry input format
   - **Reason:** Contains all the critical pitfalls and gotchas

2. `/Users/jay/development/projects/ai/warsaw-pool-ranking/database/schema.sql`
   - Lines 52-68: Games table definition (shows winner_id, no scores)
   - Line 67: Winner constraint (validates data model)
   - Lines 73-87: Ratings table (what we're calculating)
   - **Reason:** Source of truth for data structures

3. `/Users/jay/development/projects/ai/warsaw-pool-ranking/backend/app/data/parser.py`
   - Lines 259-286: Match-to-games expansion (THE critical logic)
   - Lines 355-376: Discipline filtering
   - Lines 290-316: Date parsing (multiple formats)
   - **Reason:** Shows how to correctly transform match data

**Priority 2 - Implementation Patterns:**

4. `/Users/jay/development/projects/ai/warsaw-pool-ranking/backend/scripts/weekly_update.py`
   - Lines 135-174: Orchestration flow (4 steps)
   - Lines 176-215: Step 1 - Tournament discovery
   - Lines 217-286: Step 2 - Fetch with two-tier caching
   - Lines 288-319: Step 3 - Database update
   - Lines 321-401: Step 4 - Rating calculation
   - **Reason:** User's preferred orchestration style

5. `/Users/jay/development/projects/ai/warsaw-pool-ranking/backend-rust-ai/src/rating.rs`
   - Lines 123-172: MM algorithm (copy with field name fixes)
   - Lines 87-119: Comparison matrix building (adapt winner logic)
   - Lines 175-182: Confidence level logic (exact match)
   - **Reason:** Working algorithm implementation

---

## Summary: Critical Success Factors

### 1. Get the Data Model Right (Phase 1)
- No match scores in database
- Only game-level records with winner_id
- This is non-negotiable

### 2. Implement Match-to-Games Expansion Correctly (Phase 4)
- 7-5 match = 12 game records
- Test this thoroughly before moving on
- One match wrong = entire rating system wrong

### 3. Use Database IDs in Foreign Keys (Phase 5)
- Map CueScore ID → Database ID before inserting games
- Foreign keys reference auto-increment IDs
- Test with join queries

### 4. Preserve User's Narrative Style (Phase 9)
- Clear step-by-step logging
- Progress indicators
- Results summaries
- Make it obvious what's happening

### 5. Ensure Rating Calculation Performs (Phase 7)
- Must complete in < 30 seconds for full dataset
- Use proven MM algorithm from AI reference
- No cutting corners on this - it's the bottleneck in Python

---

## Critical Files for Implementation

### Files in backend-rust/ to Read First

1. `/Users/jay/development/projects/ai/warsaw-pool-ranking/backend-rust/src/main.rs`
   - Entry point, CLI wiring
   
2. `/Users/jay/development/projects/ai/warsaw-pool-ranking/backend-rust/src/lib.rs`
   - Where orchestration will live (currently stubs)
   
3. `/Users/jay/development/projects/ai/warsaw-pool-ranking/backend-rust/src/domain/models.rs`
   - **CRITICAL FIX NEEDED:** Remove score fields from Game struct

### Files from backend-rust-ai/ to Reference

1. `/Users/jay/development/projects/ai/warsaw-pool-ranking/backend-rust-ai/src/rating.rs`
   - MM algorithm (lines 123-172): Copy directly, fix field names
   
2. `/Users/jay/development/projects/ai/warsaw-pool-ranking/backend-rust-ai/src/scraper.rs`
   - Web scraping: 90% reusable, add narrative logging
   
3. `/Users/jay/development/projects/ai/warsaw-pool-ranking/backend-rust-ai/src/cache.rs`
   - Cache system: Extend to two-tier

### Python Files That Inform Design

1. `/Users/jay/development/projects/ai/warsaw-pool-ranking/backend/app/data/parser.py`
   - **Lines 259-286:** Match-to-games expansion (must replicate)
   
2. `/Users/jay/development/projects/ai/warsaw-pool-ranking/backend/scripts/weekly_update.py`
   - **Lines 135-174:** Orchestration pattern (must match style)
