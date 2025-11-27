# Python Implementation Deep Dive

Complete technical documentation of the current Python backend to help replicate functionality in Rust.

---

## Table of Contents
1. [Caching Strategy](#1-caching-strategy)
2. [Database Schema & Mapping](#2-database-schema--mapping)
3. [Data Preprocessing Pipeline](#3-data-preprocessing-pipeline)
4. [Complete Data Flow](#4-complete-data-flow)
5. [Key Algorithms](#5-key-algorithms)

---

## 1. CACHING STRATEGY

### Directory Structure
```
backend/cache/
├── raw/                    # Individual tournament API responses
│   ├── 1621296.json       # Tournament ID 1621296 raw API response
│   ├── 1672158.json       # Tournament ID 1672158 raw API response
│   └── ...
└── parsed/                 # Processed data ready for database
    └── tournaments.json    # All tournaments parsed into game records
```

### Two-Tier Caching System

#### Tier 1: Raw API Cache (`cache/raw/`)
**Purpose**: Avoid re-fetching tournament data from CueScore API

**File Naming**: `{tournament_id}.json`
- Example: `1621296.json` = raw API response for tournament 1621296

**Content**: Complete, unmodified CueScore API JSON response
```json
{
  "id": "1621296",
  "name": "Weekly 9-Ball Tournament",
  "discipline": "9-ball",
  "startDate": "2024-03-15",
  "venues": [{"venueId": "1634568", "name": "Klub Pictures"}],
  "matches": [
    {
      "id": "12345",
      "playerA": {"playerId": "101", "name": "Jan Kowalski"},
      "playerB": {"playerId": "102", "name": "Anna Nowak"},
      "scoreA": 7,
      "scoreB": 5,
      "starttime": "2024-03-15T19:30:00Z"
    }
    // ... more matches
  ]
}
```

**Invalidation Strategy**:
- **NEVER automatically deleted**
- Only cleared manually or when cache directory is wiped
- Assumption: Tournament results don't change once finalized
- For ongoing tournaments: data will be re-fetched and overwritten

**Code Location**: `backend/scripts/weekly_update.py`
```python
def _save_raw_tournament(self, tournament_id: str, raw_data: Dict):
    cache_file = self.raw_cache_dir / f"{tournament_id}.json"
    with open(cache_file, 'w') as f:
        json.dump(raw_data, f, default=str)

def _load_raw_tournament(self, tournament_id: str) -> Optional[Dict]:
    cache_file = self.raw_cache_dir / f"{tournament_id}.json"
    if not cache_file.exists():
        return None
    with open(cache_file, 'r') as f:
        return json.load(f)
```

#### Tier 2: Parsed Data Cache (`cache/parsed/`)
**Purpose**: Skip both API fetching AND parsing

**File Naming**: Single file `tournaments.json`

**Content**: Array of parsed tournament objects ready for database insertion
```json
[
  {
    "tournament_info": {
      "cuescore_id": "1621296",
      "name": "Weekly 9-Ball Tournament",
      "start_date": "2024-03-15",
      "end_date": "2024-03-15",
      "venue_cuescore_id": "1634568"
    },
    "participants": [
      {
        "cuescore_id": "101",
        "name": "Jan Kowalski",
        "cuescore_profile_url": "https://cuescore.com/player/Jan+Kowalski/101"
      },
      {
        "cuescore_id": "102",
        "name": "Anna Nowak",
        "cuescore_profile_url": "https://cuescore.com/player/Anna+Nowak/102"
      }
    ],
    "games": [
      {
        "cuescore_match_id": "12345",
        "tournament_cuescore_id": "1621296",
        "player_a_cuescore_id": "101",
        "player_b_cuescore_id": "102",
        "winner_cuescore_id": "101",
        "played_at": "2024-03-15T19:30:00+00:00"
      }
      // ... 12 total game records (7 won by player A, 5 by player B)
    ]
  }
  // ... all other tournaments
]
```

**Invalidation Strategy**:
- **Checked first** - if exists, skip ALL fetching/parsing
- **Overwritten** whenever new tournaments are processed
- **Must be manually deleted** to force re-processing

**Cache Decision Flow**:
```
1. Check parsed cache (tournaments.json)
   ├─ EXISTS → Use it, skip everything
   └─ NOT EXISTS ↓

2. For each tournament ID:
   ├─ Check raw cache ({tournament_id}.json)
   │  ├─ EXISTS → Parse it
   │  └─ NOT EXISTS ↓
   │
   └─ Fetch from API → Save to raw cache → Parse it

3. Save all parsed data to tournaments.json
```

### Incremental Updates
The system compares discovered tournament IDs against database:
```python
existing_ids = set(t[0] for t in self.db.query(Tournament.cuescore_id).all())
new_tournament_ids = list(tournament_ids - existing_ids)
```
**Only processes new tournaments**, avoiding redundant work.

---

## 2. DATABASE SCHEMA & MAPPING

### Database Schema (PostgreSQL)

#### Players Table
```sql
CREATE TABLE players (
    id SERIAL PRIMARY KEY,
    cuescore_id VARCHAR(50) UNIQUE NOT NULL,  -- "101", "102", etc.
    name VARCHAR(255) NOT NULL,                -- "Jan Kowalski"
    cuescore_profile_url VARCHAR(500),         -- "https://cuescore.com/player/..."
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_players_cuescore_id ON players(cuescore_id);
CREATE INDEX idx_players_name ON players(name);
```

#### Venues Table
```sql
CREATE TABLE venues (
    id SERIAL PRIMARY KEY,
    cuescore_id VARCHAR(50) UNIQUE NOT NULL,  -- "1634568"
    name VARCHAR(255) NOT NULL,                -- "Klub Pictures"
    cuescore_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_venues_cuescore_id ON venues(cuescore_id);
```

#### Tournaments Table
```sql
CREATE TABLE tournaments (
    id SERIAL PRIMARY KEY,
    cuescore_id VARCHAR(50) UNIQUE NOT NULL,  -- "1621296"
    name VARCHAR(255) NOT NULL,                -- "Weekly 9-Ball"
    venue_id INTEGER REFERENCES venues(id) ON DELETE SET NULL,
    start_date DATE,
    end_date DATE,
    cuescore_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_tournaments_cuescore_id ON tournaments(cuescore_id);
```

#### Games Table (CRITICAL!)
```sql
CREATE TABLE games (
    id SERIAL PRIMARY KEY,
    cuescore_match_id VARCHAR(100) NOT NULL,  -- "12345"
    tournament_id INTEGER REFERENCES tournaments(id) ON DELETE CASCADE NOT NULL,
    player_a_id INTEGER REFERENCES players(id) ON DELETE CASCADE NOT NULL,
    player_b_id INTEGER REFERENCES players(id) ON DELETE CASCADE NOT NULL,
    winner_id INTEGER REFERENCES players(id) ON DELETE CASCADE NOT NULL,
    played_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),

    CONSTRAINT winner_is_player CHECK (
        (winner_id = player_a_id) OR (winner_id = player_b_id)
    )
);

CREATE INDEX idx_games_tournament ON games(tournament_id);
CREATE INDEX idx_games_players ON games(player_a_id, player_b_id);
CREATE INDEX idx_games_played_at ON games(played_at);
```

**Important**: This is NOT match-level data - it's GAME-level!
- Match with score 7-5 = 12 database rows (7 games + 5 games)

#### Ratings Table
```sql
CREATE TABLE ratings (
    id SERIAL PRIMARY KEY,
    player_id INTEGER REFERENCES players(id) ON DELETE CASCADE UNIQUE NOT NULL,
    rating FLOAT NOT NULL,                     -- 623.45
    games_played INTEGER NOT NULL DEFAULT 0,   -- Total games (weighted sum)
    total_wins INTEGER NOT NULL DEFAULT 0,
    total_losses INTEGER NOT NULL DEFAULT 0,
    confidence_level VARCHAR(50) NOT NULL,     -- 'unranked', 'provisional', etc.
    best_rating FLOAT,
    best_rating_date DATE,
    calculated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT rating_range CHECK (rating >= 0 AND rating <= 2000)
);

CREATE INDEX idx_ratings_player_id ON ratings(player_id);
```

#### Rating Snapshots Table (Historical)
```sql
CREATE TABLE rating_snapshots (
    id SERIAL PRIMARY KEY,
    player_id INTEGER REFERENCES players(id) ON DELETE CASCADE NOT NULL,
    week_ending DATE NOT NULL,
    rating FLOAT NOT NULL,
    games_played INTEGER NOT NULL,
    confidence_level VARCHAR(50) NOT NULL,
    calculation_version VARCHAR(10) NOT NULL DEFAULT 'v1',
    created_at TIMESTAMP DEFAULT NOW(),

    CONSTRAINT snapshot_rating_range CHECK (rating >= 0 AND rating <= 2000)
);

CREATE INDEX idx_snapshots_week_ending ON rating_snapshots(week_ending);
```

### JSON to SQL Mapping

#### Player Mapping
```python
# From parsed JSON:
{
    "cuescore_id": "101",
    "name": "Jan Kowalski",
    "cuescore_profile_url": "https://cuescore.com/player/Jan+Kowalski/101"
}

# To Database (Upsert):
def _upsert_player(self, participant: Dict):
    player = self.db.query(Player).filter(
        Player.cuescore_id == participant['cuescore_id']
    ).first()

    if not player:
        player = Player(
            cuescore_id=participant['cuescore_id'],
            name=participant['name'],
            cuescore_profile_url=participant.get('cuescore_profile_url')
        )
        self.db.add(player)
    else:
        # Update name if changed
        player.name = participant['name']

    self.db.flush()
```

#### Tournament Mapping
```python
# From parsed JSON:
{
    "cuescore_id": "1621296",
    "name": "Weekly 9-Ball Tournament",
    "start_date": "2024-03-15",
    "end_date": "2024-03-15",
    "venue_cuescore_id": "1634568"
}

# To Database:
def _upsert_tournament(self, tournament_info: Dict) -> int:
    # Get venue database ID
    venue = self.db.query(Venue).filter(
        Venue.cuescore_id == tournament_info['venue_cuescore_id']
    ).first()

    # Upsert tournament
    tournament = self.db.query(Tournament).filter(
        Tournament.cuescore_id == tournament_info['cuescore_id']
    ).first()

    if not tournament:
        tournament = Tournament(
            cuescore_id=tournament_info['cuescore_id'],
            name=tournament_info['name'],
            venue_id=venue.id if venue else None,
            start_date=tournament_info['start_date'],
            end_date=tournament_info['end_date']
        )
        self.db.add(tournament)
        self.db.flush()

    return tournament.id
```

#### Game Mapping (Critical - Match to Games Expansion)
```python
# From parsed JSON (already game-level):
{
    "cuescore_match_id": "12345",
    "tournament_cuescore_id": "1621296",
    "player_a_cuescore_id": "101",
    "player_b_cuescore_id": "102",
    "winner_cuescore_id": "101",  # Player A won THIS game
    "played_at": "2024-03-15T19:30:00+00:00"
}

# To Database:
def _insert_game(self, game_data: Dict, tournament_db_id: int):
    # Get player database IDs
    player_a = self.db.query(Player).filter(
        Player.cuescore_id == game_data['player_a_cuescore_id']
    ).first()

    player_b = self.db.query(Player).filter(
        Player.cuescore_id == game_data['player_b_cuescore_id']
    ).first()

    winner = self.db.query(Player).filter(
        Player.cuescore_id == game_data['winner_cuescore_id']
    ).first()

    # Insert game
    game = Game(
        cuescore_match_id=game_data['cuescore_match_id'],
        tournament_id=tournament_db_id,
        player_a_id=player_a.id,
        player_b_id=player_b.id,
        winner_id=winner.id,
        played_at=game_data['played_at']
    )
    self.db.add(game)
```

---

## 3. DATA PREPROCESSING PIPELINE

### Step-by-Step Transformation

#### Step 1: Raw API Response → Parsed Tournament
**Code**: `backend/app/data/parser.py`

**Input** (from API):
```json
{
  "id": "1621296",
  "name": "Weekly 9-Ball",
  "discipline": "9-ball",
  "startDate": "2024-03-15",
  "venues": [{"venueId": "1634568"}],
  "matches": [
    {
      "id": "12345",
      "playerA": {"playerId": "101", "name": "Jan Kowalski"},
      "playerB": {"playerId": "102", "name": "Anna Nowak"},
      "scoreA": 7,
      "scoreB": 5,
      "starttime": "2024-03-15T19:30:00Z"
    }
  ]
}
```

**Processing**:
1. **Discipline Filter**: Check if `discipline.lower()` is in excluded list
   - Excluded: `snooker`, `pyramid`, `piramida`, `russian pyramid`, `russian pool`
   - If excluded → Return `None` (skip tournament)

2. **Extract Tournament Info**:
   ```python
   tournament_info = {
       'cuescore_id': data['id'],                    # "1621296"
       'name': data['name'],                          # "Weekly 9-Ball"
       'start_date': parse_date(data['startDate']),  # datetime.date
       'end_date': parse_date(data['endDate']),      # datetime.date or None
       'venue_cuescore_id': data['venues'][0]['venueId']  # "1634568"
   }
   ```

3. **Extract Participants** (from matches):
   ```python
   # Deduplicate players across all matches
   players_dict = {}
   for match in data['matches']:
       player_a_id = match['playerA']['playerId']
       player_a_name = match['playerA']['name']
       players_dict[player_a_id] = {
           'cuescore_id': player_a_id,
           'name': player_a_name,
           'cuescore_profile_url': f"https://cuescore.com/player/{player_a_name.replace(' ', '+')}/{player_a_id}"
       }
       # Same for player B...

   participants = list(players_dict.values())
   ```

4. **Convert Matches to Games** (CRITICAL EXPANSION):
   ```python
   # For each match...
   match = {
       "id": "12345",
       "scoreA": 7,
       "scoreB": 5,
       "playerA": {"playerId": "101"},
       "playerB": {"playerId": "102"},
       "starttime": "2024-03-15T19:30:00Z"
   }

   # Create 7 game records for player A wins:
   for _ in range(7):  # scoreA
       games.append({
           'cuescore_match_id': "12345",
           'tournament_cuescore_id': "1621296",
           'player_a_cuescore_id': "101",
           'player_b_cuescore_id': "102",
           'winner_cuescore_id': "101",  # Player A won
           'played_at': datetime(2024, 3, 15, 19, 30)
       })

   # Create 5 game records for player B wins:
   for _ in range(5):  # scoreB
       games.append({
           'cuescore_match_id': "12345",
           'tournament_cuescore_id': "1621296",
           'player_a_cuescore_id': "101",
           'player_b_cuescore_id': "102",
           'winner_cuescore_id': "102",  # Player B won
           'played_at': datetime(2024, 3, 15, 19, 30)
       })

   # Result: 12 game records from one 7-5 match
   ```

**Output** (parsed tournament):
```json
{
  "tournament_info": {...},
  "participants": [...],
  "games": [
    // 12 game records from the 7-5 match
  ]
}
```

#### Step 2: Parsed Data → Database
**Code**: `backend/scripts/weekly_update.py:_update_database()`

For each parsed tournament:
1. Upsert all participants (create if new, update name if changed)
2. Upsert tournament (link to venue)
3. Insert all game records

**No additional transformation** - data is already in correct format.

#### Step 3: Database → DataFrame for Rating Calculation
**Code**: `backend/scripts/weekly_update.py:_run_simulation()`

```python
# Fetch all games
games = self.db.query(Game).all()

# Convert to DataFrame
games_df = pd.DataFrame([{
    'player_a_id': g.player_a_id,      # Database ID (int)
    'player_b_id': g.player_b_id,
    'winner_id': g.winner_id,
    'played_at': g.played_at           # datetime
} for g in games])

# Calculate time decay weights
time_weights = self.simulator.time_decay.calculate_weights(
    games_df['played_at'],
    reference_date=datetime.now()  # Or specific week_ending for snapshots
)

games_df['weight'] = time_weights

# Now ready for Bradley-Terry algorithm input
```

#### Step 4: Time Decay Weight Calculation
**Code**: `backend/app/rating/time_decay.py`

**Formula**: `weight = exp(-λ × days_ago)`
- Where `λ = ln(2) / half_life_days`
- Default: `half_life_days = 1095` (3 years)
- `λ = ln(2) / 1095 = 0.000633`

**Example Calculation**:
```python
reference_date = datetime(2025, 11, 22)  # Today
played_date = datetime(2024, 5, 22)      # 1.5 years ago

days_ago = (reference_date - played_date).days  # ~549 days

weight = np.exp(-0.000633 × 549)
       = np.exp(-0.347517)
       = 0.706  # About 71% weight
```

**Weight Examples** (3-year half-life):
- Today: 1.00 (100%)
- 6 months ago: 0.89 (89%)
- 1 year ago: 0.79 (79%)
- 1.5 years ago: 0.71 (71%)
- 3 years ago: 0.50 (50%)
- 6 years ago: 0.25 (25%)

#### Step 5: Bradley-Terry Input Preparation
**Code**: `backend/app/rating/calculator.py`

```python
# From DataFrame to choix comparisons format
comparisons = []
for _, game in games_df.iterrows():
    # Map database IDs to 0-indexed player indices
    winner_idx = player_id_to_idx[game['winner_id']]
    loser_idx = player_id_to_idx[game['loser_id']]
    weight = game['weight']

    comparisons.append((winner_idx, loser_idx, weight))

# choix.opt_pairwise() expects:
# - n_players: int
# - comparisons: list of (winner, loser, weight) tuples
```

#### Step 6: Confidence Level & Blending
**Code**: `backend/app/rating/confidence.py`

After Bradley-Terry calculation:
```python
ml_rating = 623.45  # From Bradley-Terry
games_played = 25

# Determine confidence level
if games_played < 10:
    confidence = ConfidenceLevel.UNRANKED
elif games_played < 50:
    confidence = ConfidenceLevel.PROVISIONAL  # ← This case
elif games_played < 200:
    confidence = ConfidenceLevel.EMERGING
else:
    confidence = ConfidenceLevel.ESTABLISHED

# Blend with starter rating (500) for non-established players
starter_weight = (200 - 25) / 200 = 0.875
ml_weight = 1 - 0.875 = 0.125

blended_rating = 0.875 × 500 + 0.125 × 623.45
               = 437.5 + 77.93
               = 515.43

# Save to database:
rating_record = {
    'player_id': player_db_id,
    'rating': 515.43,              # Blended
    'games_played': 25,
    'confidence_level': 'provisional',
    'calculated_at': datetime.now()
}
```

---

## 4. COMPLETE DATA FLOW

### Full Pipeline Visualization

```
┌─────────────────────────────────────────────────────────────────┐
│ STEP 1: TOURNAMENT DISCOVERY (Web Scraping)                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Scrape venue pages → Extract tournament IDs                   │
│                                                                 │
│  Input:  venues = [{"venue_id": "1634568", "venue_name": ...}] │
│  Output: tournament_ids = {"1621296", "1672158", ...}           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ STEP 2: FETCH & PARSE TOURNAMENTS (with 2-tier caching)        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  For each tournament ID:                                        │
│    1. Check parsed cache (tournaments.json)                     │
│       └─ If exists → Use it, DONE                               │
│                                                                 │
│    2. Check raw cache (1621296.json)                            │
│       ├─ If exists → Load raw JSON                              │
│       └─ Else → Fetch from API → Save to raw cache             │
│                                                                 │
│    3. Parse raw JSON:                                           │
│       ├─ Filter discipline (skip snooker/pyramid)               │
│       ├─ Extract tournament info                                │
│       ├─ Extract participants (unique players)                  │
│       └─ Convert matches to games:                              │
│           Match 7-5 → 7 game records (player A wins)            │
│                     + 5 game records (player B wins)            │
│                                                                 │
│  Output: tournaments_data = [parsed_tournament_1, ...]          │
│  Save:   cache/parsed/tournaments.json                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ STEP 3: UPDATE DATABASE                                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  For each parsed tournament:                                    │
│    1. Upsert players (from participants)                        │
│    2. Upsert tournament (link to venue)                         │
│    3. Insert games (all game records)                           │
│                                                                 │
│  Result: PostgreSQL database populated:                         │
│    - 32,321 players                                             │
│    - 2,104 tournaments                                          │
│    - 342,662 games                                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ STEP 4: RATING CALCULATION                                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Fetch all games from database                               │
│                                                                 │
│  2. Calculate time decay weights:                               │
│     weight = exp(-λ × days_ago)                                 │
│     where λ = ln(2) / 1095 (3-year half-life)                   │
│                                                                 │
│  3. Prepare Bradley-Terry input:                                │
│     comparisons = [(winner_idx, loser_idx, weight), ...]        │
│                                                                 │
│  4. Run Bradley-Terry ML optimization:                          │
│     choix.opt_pairwise(n_players, comparisons) → log_ratings    │
│     ⚠️  BOTTLENECK: 20-30+ minutes, often fails                 │
│                                                                 │
│  5. Post-process ratings:                                       │
│     - Convert log_ratings to ratings (exponential)              │
│     - Determine confidence level (based on games_played)        │
│     - Blend with starter rating (500) if < 200 games            │
│     - Calculate wins/losses counts                              │
│                                                                 │
│  6. Save to ratings table                                       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Time Breakdown (Full Dataset)

| Step | Operation | Time | Notes |
|------|-----------|------|-------|
| 1 | Tournament discovery (scraping) | ~2 min | Rate limited, 2104 tournaments |
| 2a | Fetch from API (if not cached) | ~3-5 min | Rate limited, 1 req/sec |
| 2b | Parse tournaments | ~30 sec | Pure CPU, no I/O |
| 3 | Database insertion | ~6-7 min | 342,662 games, sequential inserts |
| 4 | **Rating calculation** | **20-30+ min** | **FAILS - choix can't handle scale** |
| **Total** | **Failed after 30+ min** | **System can't complete** |

---

## 5. KEY ALGORITHMS

### Time Decay Algorithm

**Purpose**: Weight recent games more heavily than old games

**Formula**:
```
weight = exp(-λ × days_ago)

where:
  λ = ln(2) / half_life_days
  half_life_days = 1095 (3 years)
  λ = 0.000633
```

**Implementation** (`backend/app/rating/time_decay.py`):
```python
def calculate_weights(self, played_dates, reference_date=None):
    if reference_date is None:
        reference_date = datetime.now()

    # Calculate days since each game
    days_ago = [(reference_date - date).days for date in played_dates]

    # Exponential decay
    weights = np.exp(-self.lambda_param * np.array(days_ago))

    return weights
```

**Rust Implementation Notes**:
- Use `chrono` crate for date arithmetic
- Implement as: `(-lambda * days_ago).exp()`
- Pre-calculate lambda: `f64::ln(2.0) / 1095.0 = 0.000633`

### Confidence Level & Rating Blending

**Purpose**: New players get blended ratings until established

**Confidence Levels**:
```
< 10 games   → Unranked (don't show in rankings)
10-49 games  → Provisional
50-199 games → Emerging
200+ games   → Established (matches FargoRate)
```

**Blending Formula**:
```
If games_played >= 200:
    final_rating = ml_rating
Else:
    starter_weight = (200 - games_played) / 200
    ml_weight = 1 - starter_weight
    final_rating = starter_weight × 500 + ml_weight × ml_rating
```

**Implementation** (`backend/app/rating/confidence.py`):
```python
def blend_rating(self, ml_rating, games_played):
    if games_played >= 200:
        return ml_rating

    starter_weight = (200 - games_played) / 200
    ml_weight = 1.0 - starter_weight

    return starter_weight * 500.0 + ml_weight * ml_rating
```

**Rust Implementation Notes**:
- Simple linear interpolation
- Starter rating: 500.0
- Established threshold: 200 games

### Bradley-Terry Maximum Likelihood

**Purpose**: Calculate player ratings from pairwise game results

**Input Format**:
```python
n_players = 32321
comparisons = [
    (winner_idx, loser_idx, weight),  # Tuple for each game
    (0, 1, 1.0),        # Player 0 beat Player 1, weight 1.0
    (0, 2, 0.89),       # Player 0 beat Player 2, weight 0.89 (older game)
    (1, 2, 0.71),       # Player 1 beat Player 2, weight 0.71
    # ... 342,662 total
]
```

**Python Implementation** (FAILED):
```python
# This DOES NOT work for 32K+ players!
params = choix.opt_pairwise(
    n_players,
    comparisons,
    method='Newton-CG',  # Or 'ILSR' - both fail
    alpha=0.01,
    tol=1e-6,
    max_iter=100
)
# Returns: log-scale ratings (need exp() to get actual ratings)
```

**Rust Implementation** (WORKING):
- Custom MM algorithm in `backend-rust/src/rating.rs`
- Expected performance: 5-10 seconds vs 20-30+ minutes failure
- See `PYTHON_VS_RUST.md` for detailed comparison

---

## SUMMARY FOR RUST IMPLEMENTATION

### Must-Have Features

1. **Two-Tier Caching**:
   - `cache/raw/{tournament_id}.json` for API responses
   - `cache/parsed/tournaments.json` for processed data
   - Check parsed cache first (fastest path)

2. **Match-to-Games Expansion**:
   - **CRITICAL**: One match (7-5) = 12 game records
   - Each game record has winner_id pointing to that game's winner

3. **Time Decay Weights**:
   - `weight = exp(-0.000633 × days_ago)`
   - Apply BEFORE Bradley-Terry calculation
   - Reference date = now (or specific week for snapshots)

4. **Database Schema**:
   - Use exact same schema (compatibility with frontend)
   - Games table is game-level, not match-level
   - Foreign keys: player IDs, tournament IDs

5. **Confidence & Blending**:
   - 4 levels: unranked, provisional, emerging, established
   - Blend with starter rating (500) until 200 games
   - Formula: `blend = starter_weight × 500 + ml_weight × ml_rating`

6. **Discipline Filtering**:
   - Exclude: snooker, pyramid, piramida, russian pyramid, russian pool
   - Case-insensitive match

### Performance Targets (Rust)

- Tournament discovery: ~2 min (same as Python, rate limited)
- API fetching: ~3-5 min (same, rate limited)
- Parsing: ~5 sec (10x faster, no Python overhead)
- Database insertion: ~3-4 min (2x faster, batch operations)
- **Rating calculation: ~8 sec** (200x faster, custom MM algorithm)
- **Total: ~10-12 minutes** (vs Python's failure after 30+ min)

### Critical Differences from Naive Implementation

1. ❌ DON'T store match scores in database
   ✅ DO expand matches into individual game records

2. ❌ DON'T apply time decay after rating calculation
   ✅ DO calculate weights before Bradley-Terry

3. ❌ DON'T skip caching (API is rate-limited)
   ✅ DO implement two-tier cache

4. ❌ DON'T use final ratings directly for new players
   ✅ DO blend with starter rating based on games played

5. ❌ DON'T rely on choix or scipy
   ✅ DO implement custom MM algorithm (it's only ~100 lines)

---

This documentation should give you everything needed to replicate the Python implementation's behavior in Rust while avoiding the performance pitfalls!
