# Warsaw Pool Ranking System - Design Document

## 1. Project Overview

### 1.1 Purpose
A one-page web application that displays skill-based ratings for Warsaw pool players using a custom maximum likelihood rating system inspired by Fargo.

### 1.2 Key Objectives
- Provide accurate, data-driven player ratings for the Warsaw pool community
- Offer transparency into rating calculation and confidence levels
- Enable players to track their progress over time
- Use individual game results (not just match outcomes) for better granularity

---

## 2. Rating System Design

### 2.1 Core Algorithm: Maximum Likelihood Estimation

**Approach:** Weekly full historical recalculation of all ratings

Unlike traditional Elo systems that update incrementally, this system recalculates ratings from scratch each week by finding the set of ratings that **best explains all observed game outcomes** in the historical data.

### 2.2 Probability Model

**Logarithmic scale with defined odds ratio:**
- **100-point rating gap = 2:1 winning odds** (same as Fargo)

**Win probability formula:**
```
P(Player A wins a game) = 1 / (1 + 2^((Rating_B - Rating_A) / 100))
```

Where:
- Rating_A = Player A's rating
- Rating_B = Player B's rating

**Example interpretations:**
- 100-point advantage → ~67% win probability (2:1 odds)
- 200-point advantage → ~80% win probability (4:1 odds)
- 0-point difference → 50% win probability (1:1 odds)

### 2.3 Game-Level Granularity

**Individual games as base units**, not matches:

- ❌ **Don't do:** Record "Player A beat Player B in a race to 7"
- ✅ **Do:** Record "In a race to 7 with score 7-5, Player A won 7 games and Player B won 5 games"

**Benefits:**
- 12 data points instead of 1 from each race-to-7 match
- Close matches (7-5) vs blowouts (7-1) naturally weighted differently
- More accurate ratings with less total data needed
- Better handles upsets and variance

### 2.4 Maximum Likelihood Calculation

**High-level algorithm:**

1. Start with initial rating estimates for all players (e.g., 500)
2. For each game in the historical dataset, calculate the likelihood that the observed outcome would occur given current ratings
3. Iteratively adjust ratings to maximize the total likelihood across all games
4. Converge to the rating set that best predicts the observed results

**Implementation approach:**
- Use optimization algorithms (e.g., gradient descent, Newton-Raphson, or Bradley-Terry model solvers)
- Process entire game history weekly
- Maintain chronological order awareness (players improve over time)

### 2.5 Handling New Players

**Weighted blend approach** (similar to Fargo):

- New players start with a **default starter rating** (e.g., 500)
- As games accumulate, their rating becomes a weighted blend:
  - More early games → Higher weight on starter rating
  - More total games → Higher weight on calculated rating
- Threshold: **100 games for fully "established" rating**

**Formula:**
```
Displayed_Rating = (Starter_Weight × Starter_Rating) + (Calculated_Weight × ML_Rating)

Where:
Starter_Weight = max(0, (100 - Games_Played) / 100)
Calculated_Weight = 1 - Starter_Weight
```

**Confidence Levels:**
- **Unranked**: <10 games (not shown in rankings)
- **Provisional**: 10-49 games (rating shown, marked as provisional)
- **Emerging**: 50-99 games (rating stabilizing)
- **Established**: 100+ games (fully calculated rating, no blending)

**Transparency for users:**
- Display confidence level badge: "Unranked", "Provisional", "Emerging", or "Established"
- Show game count prominently
- Visual indicator of rating stability (e.g., color coding, confidence badge)

### 2.6 Ranking Eligibility

**Minimum threshold:** Players need **10+ games** to appear in rankings

**Reasoning:**
- Prevents brand-new players with 1-2 lucky/unlucky results from appearing
- Low enough barrier that active players qualify quickly
- Players with <10 games can still have profiles, just marked as "Unranked"

### 2.7 Time Decay

**Exponential time decay with 3-year half-life:**

Older games gradually receive less weight in rating calculations to account for player improvement/decline over time.

**Formula:**
```
Weight(game) = exp(-λ × days_since_game)

Where:
λ = ln(2) / (3 × 365) ≈ 0.000633
days_since_game = days between game date and reference date
```

**Example weights:**
- Game from today: 100% weight
- Game from 1.5 years ago: ~71% weight
- Game from 3 years ago: 50% weight (half-life)
- Game from 6 years ago: 25% weight

**Implementation notes:**
- Applied from day one (no deferral)
- During weekly simulation, weights calculated relative to each simulated week
- Allows ratings to naturally reflect recent performance more heavily
- May be tuned via cross-validation after data collection

### 2.8 Weekly Simulation Approach

**Key innovation:** Instead of maintaining append-only snapshots, we **replay entire history week-by-week** each recalculation.

**Algorithm:**
```python
def weekly_recalculation():
    all_games = fetch_all_historical_games()
    weeks = get_week_boundaries(all_games)  # All weeks from first game to now

    snapshots = []
    for week_ending in weeks:
        games_up_to_week = all_games[date <= week_ending]

        # Calculate ratings as if running on that week
        ratings = calculate_ml_ratings(
            games_up_to_week,
            time_weights=time_decay(games_up_to_week, reference_date=week_ending)
        )

        snapshots.append((week_ending, ratings))

    # Replace entire rating_snapshots table
    replace_rating_snapshots(snapshots)
```

**Benefits:**
1. ✅ Algorithm changes automatically recalculate entire history
2. ✅ All snapshots always use same algorithm version (consistent)
3. ✅ Rating history ready for charts without extra infrastructure
4. ✅ Easy to track `recent_change` (compare last two weeks)
5. ✅ Time decay weights adjust correctly for each historical week

**Trade-off:** More computation weekly, but acceptable given we're recalculating anyway.

---

## 3. Game Type Support

### 3.1 Multiple Ranking Systems

**If Cuescore provides game-type-specific data:**
- Separate rankings for: **8-ball**, **9-ball**, **10-ball**
- **Unified cross-game ranking** (all games combined)

**If Cuescore doesn't separate by game type:**
- Single **unified ranking** across all pool variants

### 3.2 Implementation Notes

- Each ranking system runs the same maximum likelihood algorithm independently
- Players may have different ratings in different game types
- Unified ranking treats all games equally regardless of variant

---

## 4. Data Source & Updates

### 4.1 Data Source

**CueScore Data Collection Strategy**

**Two-layer approach:**

1. **Venue-based Tournament Discovery** (Web Scraping)
   - Target Warsaw/Masovian voivodeship venues
   - Scrape venue pages: `https://cuescore.com/venue/{name}/{id}/tournaments`
   - Handle pagination with `?&page=N` parameter (~30 tournaments per page)
   - Iterate until "Next »" link disappears
   - Extract tournament IDs from tournament list

2. **Tournament Details Fetching** (API)
   - Use tournament IDs to fetch detailed data
   - Endpoint: `https://api.cuescore.com/tournament/?id={tournament_id}`
   - Returns: participants, matches, scores, dates
   - Convert match scores to individual game records

**Data points collected:**
- Player names and CueScore IDs
- Match results (scores like 7-5, 9-3)
- Tournament metadata
- Venue information
- Timestamps

**Rate limiting:**
- 1 request/second to respect CueScore servers
- Exponential backoff for errors
- Use `tenacity` library for retry logic

### 4.2 Player Selection Criteria

**Venue-based approach:**
- Include **all players** who participated in tournaments at Warsaw/Masovian venues
- Natural filtering: If a player played in a tournament at a Warsaw venue, they're included
- No need for manual curation or geographic tagging
- Players automatically appear once they play at tracked venues

### 4.3 Update Frequency

**Weekly recalculation with full history simulation:**
- Automated job runs weekly (e.g., Sunday night)
- Fetches new tournament data from CueScore (incremental)
- **Replays entire game history week-by-week** (see Section 2.8)
- Generates complete rating snapshots for all historical weeks
- Replaces `rating_snapshots` table entirely
- Updates `ratings` table with current ratings
- Updates frontend with new data

**Why weekly?**
- Balances freshness with computational cost
- Gives players consistent weekly checkpoint to track progress
- Reduces API/scraping load vs. real-time updates
- Allows algorithm improvements to automatically update all history

---

## 5. Features & User Interface

### 5.1 Core Features

#### 5.1.1 Searchable Player List (Angular Component)
- **Client-side search/filter by name** (Angular pipe, instant filtering)
- Display columns:
  - Rank
  - Player Name (clickable → opens overlay)
  - Current Rating
  - Rating confidence badge ("Unranked" / "Provisional" / "Emerging" / "Established")
  - Games played count
  - Recent change (e.g., +15 from last week)
  - "View on CueScore" link icon

**Implementation:**
- Angular Material Table with sorting
- Search input with reactive forms
- Click on player row opens dialog/overlay

#### 5.1.2 Player Overlay/Dialog (Angular Material Dialog)
**Triggered:** Click on player in list

**Content:**
- Player name and CueScore profile link
- Current rating and rank
- Confidence level badge
- Statistics: games played, win percentage, best rating
- **Rating history chart** (loaded on-demand via API)
  - Line graph showing rating progression over all time
  - X-axis: Weeks
  - Y-axis: Rating
  - Implemented with `ng2-charts` or `ngx-charts`
- Recent opponents list
- "Close" button

### 5.2 Additional UI Elements

**Header (Angular Component):**
- Title: "Warsaw Pool Rankings"
- Last updated timestamp
- Brief explanation of rating system (expandable tooltip)
- Link to methodology/about page

**Future Enhancements:**
- Filter by game type (if multiple rankings supported)
- Filter by rating range
- Filter by club/venue
- Dark mode toggle

---

## 6. Technical Architecture

### 6.1 Technology Stack

**Frontend:**
- **Angular** with TypeScript
- **Angular Material** for UI components
- **ng2-charts** or **ngx-charts** for rating history visualization
- **Angular HTTP Client** for API communication
- Responsive, mobile-friendly design

**Backend:**
- **Python FastAPI** for REST API server
- **SQLAlchemy** for database ORM
- **Pydantic** for data validation
- CORS configuration for Angular frontend

**Rating Calculation Engine (Python):**
- **choix** library for Bradley-Terry maximum likelihood implementation
- **NumPy**, **SciPy** for numerical optimization
- **pandas** for data manipulation
- Weekly simulation engine (replay history week-by-week)

**Data Collection:**
- **requests** + **BeautifulSoup4** for venue page scraping
- **tenacity** for retry logic and rate limiting
- CueScore API client for tournament data

**Database:**
- **PostgreSQL** with full CueScore ID tracking

**Deployment:**
- **Frontend hosting** for Angular build (Vercel, Netlify)
- **Backend hosting** for FastAPI (Railway, Render, AWS)
- **Scheduled jobs** (cron, GitHub Actions) for weekly recalculation

### 6.2 Data Flow

```
┌─────────────────┐
│  Cuescore API   │
└────────┬────────┘
         │ (Weekly fetch)
         ▼
┌─────────────────────┐
│  Data Fetcher       │
│  (Python script)    │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Game Database      │
│  (Raw match data)   │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────────┐
│  Rating Calculator      │
│  (Max Likelihood ML)    │
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────┐
│  Ratings Database   │
│  (Current ratings,  │
│   history, stats)   │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  API Endpoints      │
│  (JSON responses)   │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Angular Frontend   │
│  (User interface)   │
└─────────────────────┘
```

### 6.3 Database Schema

**Players Table:**
```sql
players
- id (PK, auto-increment)
- cuescore_id (unique, indexed) -- e.g., "43157920"
- name (varchar)
- cuescore_profile_url (varchar) -- e.g., "https://cuescore.com/player/..."
- created_at (timestamp)
```

**Venues Table:**
```sql
venues
- id (PK, auto-increment)
- cuescore_id (unique, indexed)
- name (varchar)
- cuescore_url (varchar)
- created_at (timestamp)
```

**Tournaments Table:**
```sql
tournaments
- id (PK, auto-increment)
- cuescore_id (unique, indexed) -- e.g., "72541144"
- name (varchar)
- venue_id (FK → venues.id)
- start_date (date)
- end_date (date, nullable)
- cuescore_url (varchar)
- created_at (timestamp)
```

**Games Table:**
```sql
games
- id (PK, auto-increment)
- cuescore_match_id (varchar, indexed) -- from tournament API
- tournament_id (FK → tournaments.id)
- player_a_id (FK → players.id)
- player_b_id (FK → players.id)
- winner_id (FK → players.id)
- game_type (enum: 8ball, 9ball, 10ball, unified)
- played_at (timestamp)
- created_at (timestamp)
```

**Ratings Table (Current):**
```sql
ratings
- id (PK, auto-increment)
- player_id (FK → players.id, unique)
- rating (float)
- games_played (int)
- total_wins (int)
- total_losses (int)
- confidence_level (enum: unranked, provisional, emerging, established)
- best_rating (float)
- best_rating_date (date)
- calculated_at (timestamp)
```

**Rating Snapshots Table (Historical):**
```sql
rating_snapshots
- id (PK, auto-increment)
- player_id (FK → players.id)
- week_ending (date, indexed)
- rating (float)
- games_played (int)
- confidence_level (enum: unranked, provisional, emerging, established)
- calculation_version (varchar) -- e.g., "v1", "v2" for algorithm tracking
- created_at (timestamp)

-- Composite index on (player_id, week_ending) for fast history queries
-- NOTE: This table is REPLACED entirely each week during simulation
```

**Indexes:**
- `players.cuescore_id` (unique)
- `venues.cuescore_id` (unique)
- `tournaments.cuescore_id` (unique)
- `games.tournament_id` (foreign key)
- `games.cuescore_match_id` (for deduplication)
- `rating_snapshots(player_id, week_ending)` (composite, for history queries)

### 6.4 API Endpoints (MVP)

**Backend API** (Python FastAPI):

#### 1. `GET /api/players`
**Purpose:** Load all players for main list (client-side search in Angular)

**Response:**
```json
[
  {
    "id": 123,
    "cuescore_id": "43157920",
    "name": "Jacek Karaśkiewicz",
    "rank": 1,
    "rating": 687.5,
    "confidence": "Established",
    "games_played": 142,
    "recent_change": 15.3,
    "cuescore_url": "https://cuescore.com/player/..."
  }
]
```

#### 2. `GET /api/player/:id`
**Purpose:** Player details for overlay/dialog

**Response:**
```json
{
  "id": 123,
  "cuescore_id": "43157920",
  "name": "Jacek Karaśkiewicz",
  "rating": 687.5,
  "rank": 1,
  "confidence": "Established",
  "games_played": 142,
  "total_wins": 89,
  "total_losses": 53,
  "win_percentage": 62.7,
  "best_rating": 695.2,
  "best_rating_date": "2025-10-15",
  "rating_trend": "improving",
  "cuescore_url": "https://cuescore.com/player/...",
  "recent_opponents": [
    {"name": "Player X", "games": 12, "cuescore_url": "..."},
    {"name": "Player Y", "games": 8, "cuescore_url": "..."}
  ]
}
```

#### 3. `GET /api/player/:id/history`
**Purpose:** Rating history for chart (loaded on-demand when overlay opens)

**Response:**
```json
[
  {"week_ending": "2025-01-07", "rating": 650.2, "games_played": 45},
  {"week_ending": "2025-01-14", "rating": 655.8, "games_played": 52},
  {"week_ending": "2025-01-21", "rating": 662.1, "games_played": 58},
  ...
]
```

**Notes:**
- All endpoints support CORS for Angular frontend
- Responses include CueScore URLs for "View on CueScore" links
- Client-side filtering/search in Angular (no pagination needed for MVP)

---

## 7. Implementation Phases

### Phase 1: Foundation (Database + Backend Structure)
- [ ] Create PostgreSQL database schema SQL file with all tables and indexes
- [ ] Set up Python backend project structure (FastAPI)
  - [ ] Install dependencies: fastapi, uvicorn, sqlalchemy, psycopg2-binary, pydantic
  - [ ] Configure database connection
  - [ ] Create SQLAlchemy models for all tables
- [ ] Set up local PostgreSQL instance

### Phase 2: Rating Algorithm Core
- [ ] Implement Bradley-Terry ML calculator using `choix` library
  - [ ] Test with synthetic data
  - [ ] Validate 100 pts = 2:1 odds relationship
- [ ] Implement time decay calculation module (3-year half-life)
- [ ] Implement new player blending/confidence module (100-game threshold)
- [ ] Write unit tests for rating calculations

### Phase 3: Data Collection Pipeline
- [ ] Implement CueScore venue scraper
  - [ ] Handle pagination (`?&page=N`)
  - [ ] Extract tournament IDs
  - [ ] Rate limiting (1 req/sec)
- [ ] Implement tournament API client
  - [ ] Fetch tournament details
  - [ ] Parse match scores into individual games
- [ ] Test data collection with a few Warsaw venues
- [ ] Populate games database with historical data

### Phase 4: Weekly Simulation Engine
- [ ] Implement weekly simulator (replay history week-by-week)
  - [ ] Generate week boundaries from game data
  - [ ] Calculate ratings for each historical week
  - [ ] Apply time decay relative to each week
- [ ] Test simulation with collected data
- [ ] Verify snapshot generation and database replacement

### Phase 5: Backend API (FastAPI)
- [ ] Create endpoint: `GET /api/players`
- [ ] Create endpoint: `GET /api/player/:id`
- [ ] Create endpoint: `GET /api/player/:id/history`
- [ ] Add CORS configuration for Angular
- [ ] Test all endpoints with Postman/curl

### Phase 6: Frontend (Angular)
- [ ] Set up Angular project with Angular CLI
  - [ ] Install Angular Material
  - [ ] Install ng2-charts or ngx-charts
- [ ] Create player list component
  - [ ] Angular Material Table
  - [ ] Client-side search with pipe
  - [ ] Sorting functionality
- [ ] Create player overlay/dialog component
  - [ ] Player details display
  - [ ] Rating history chart
  - [ ] CueScore link integration
- [ ] Create services for API communication
- [ ] Connect all components to backend API
- [ ] Add responsive design and styling
- [ ] Test on mobile devices

### Phase 7: Automation & Deployment
- [ ] Create weekly recalculation orchestration script
  - [ ] Fetch new tournaments
  - [ ] Run simulation
  - [ ] Update database
- [ ] Set up deployment
  - [ ] Backend: Railway/Render/AWS
  - [ ] Frontend: Vercel/Netlify
- [ ] Configure weekly cron job (GitHub Actions or server cron)
- [ ] Set up monitoring and error logging

### Phase 8: Testing & Launch
- [ ] End-to-end testing
- [ ] Beta testing with Warsaw pool community
- [ ] Gather feedback and iterate on algorithm parameters
- [ ] Public launch
- [ ] Create user documentation

---

## 8. Open Questions & Future Enhancements

### 8.1 Open Questions to Resolve

1. **Warsaw venue list:**
   - Which specific venues to include in data collection?
   - How to discover new venues as they appear?

2. **Time decay parameter tuning:**
   - Validate 3-year half-life with cross-validation
   - May need adjustment based on actual data patterns

3. **Starter rating:**
   - Keep at 500 for everyone, or analyze historical data to set better default?

4. **Computational performance:**
   - How long does weekly simulation take with full dataset?
   - May need optimization if runtime exceeds acceptable threshold

### 8.2 Future Enhancements

- **Head-to-head comparisons:** Show matchup history between two players
- **Club/venue rankings:** Aggregate ratings by club
- **Leaderboards:** Top movers of the week, highest climbers of the month
- **Predictions:** Predict match outcomes based on ratings
- **Social features:** Player comments, match reporting
- **Mobile app:** Native iOS/Android app
- **Multi-region:** Expand beyond Warsaw to other Polish cities
- **Tournament integration:** Direct integration with tournament organizers

---

## 9. Success Metrics

**Adoption:**
- Number of active users visiting the site
- Player engagement (searches, profile views)

**Accuracy:**
- Rating system's predictive accuracy (% of games correctly predicted)
- Community feedback on rating fairness

**Technical:**
- Weekly job success rate (>99%)
- Page load time (<2 seconds)
- API uptime (>99.5%)

---

## 10. Timeline & Resources

**Estimated timeline:** 6-8 weeks for MVP (Phases 1-5)

**Required resources:**
- 1 Full-stack developer (frontend + backend)
- Access to Cuescore API
- Hosting budget (~$10-20/month for small scale)
- Beta testers from Warsaw pool community

---

## Appendix A: Rating System Examples

### Example 1: Close Match
**Match:** Race to 7, Player A (rating 550) vs Player B (rating 500)
**Result:** 7-5 (Player A wins)

**Data recorded:**
- 12 individual games
- Player A won 7 games
- Player B won 5 games

**Effect:** Both players' ratings adjust slightly based on expected vs actual performance.

### Example 2: Upset
**Match:** Race to 5, Player C (rating 400) vs Player D (rating 600)
**Result:** 5-3 (Player C wins)

**Data recorded:**
- 8 individual games
- Player C won 5 games (expected ~2)
- Player D won 3 games (expected ~6)

**Effect:** Player C gains more rating points than normal; Player D loses more than normal.

### Example 3: New Player Blending
**Player E:** Just started, 15 games played
**Calculated ML rating:** 620
**Starter rating:** 500
**Blend weight:** (100-15)/100 = 0.85 starter, 0.15 calculated
**Displayed rating:** 0.85×500 + 0.15×620 = 425 + 93 = **518** (Provisional)
**Confidence level:** Provisional (10-49 games)

---

## Appendix B: Technical Notes on Maximum Likelihood

### Bradley-Terry Model
The rating system can be implemented as a Bradley-Terry model, which is a statistical framework for paired comparisons.

**Likelihood function:**
```
L(ratings) = ∏ P(observed outcome | ratings)
```

**Log-likelihood (easier to optimize):**
```
log L = Σ log(P(game_i outcome | ratings))
```

**Optimization:**
- Use iterative algorithms (MM algorithm, Newton-Raphson)
- Python library: `choix` or custom implementation with SciPy

**Convergence:**
- Iterate until rating changes are below threshold (e.g., 0.01)
- Typically converges in 10-100 iterations depending on data size

---

## Document Version
- **Version:** 2.0
- **Date:** 2025-11-19
- **Status:** Final Design - Ready for Implementation
- **Last Updated:** After comprehensive CueScore API exploration and rating system research
