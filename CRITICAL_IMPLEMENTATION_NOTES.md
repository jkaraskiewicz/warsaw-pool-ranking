# Critical Implementation Notes - Quick Reference

**READ THIS FIRST** before implementing the Rust backend!

---

## 🔥 Most Critical Gotcha: Match vs Game Records

### ❌ WRONG - Storing Matches
```sql
-- DON'T DO THIS
CREATE TABLE matches (
    player_a_id INT,
    player_b_id INT,
    score_a INT,  -- 7
    score_b INT   -- 5
);
```

### ✅ CORRECT - Storing Individual Games
```sql
-- DO THIS
CREATE TABLE games (
    player_a_id INT,
    player_b_id INT,
    winner_id INT  -- Either player_a_id or player_b_id
);
```

**Why**: Bradley-Terry needs individual game results, not match scores.

**Conversion Example**:
```
Match: Player A vs Player B, score 7-5

Becomes 12 database rows:
- 7 rows with winner_id = player_a_id
- 5 rows with winner_id = player_b_id
```

**Code Location**: `backend/app/data/parser.py:_parse_single_match()`

---

## 🗂️ Cache File Naming

### Raw Cache (API Responses)
```
backend/cache/raw/
├── 1621296.json   ← Tournament ID 1621296
├── 1672158.json   ← Tournament ID 1672158
└── 1751987.json   ← Tournament ID 1751987
```

**Pattern**: `{tournament_id}.json`

**Content**: Unmodified CueScore API response

**Invalidation**: NEVER (unless manually deleted)

### Parsed Cache (Processed Data)
```
backend/cache/parsed/
└── tournaments.json  ← ALL tournaments
```

**Pattern**: Single file for all data

**Content**: Array of parsed tournaments (tournament_info + participants + games)

**Invalidation**: Overwritten on each run

**Check Order**:
1. Check `tournaments.json` - if exists, use it (skip everything)
2. Else, for each tournament, check `raw/{id}.json`
3. If raw cache miss → fetch from API

---

## ⏰ Time Decay Formula

```rust
// EXACT formula from Python
let lambda = f64::ln(2.0) / 1095.0;  // 0.000633
let weight = (-lambda * days_ago as f64).exp();
```

**Values**:
- Half-life: 1095 days (3 years)
- Lambda (λ): 0.000633

**Example Weights**:
```
Today:       1.00 (100%)
6 months:    0.89
1 year:      0.79
1.5 years:   0.71
3 years:     0.50
6 years:     0.25
```

**Apply BEFORE Bradley-Terry**, not after!

---

## 🎯 Confidence Levels & Blending

### Confidence Thresholds
```rust
match games_played {
    0..=9   => Unranked,      // Don't show in rankings
    10..=49 => Provisional,
    50..=199 => Emerging,
    200..   => Established    // Matches FargoRate
}
```

### Rating Blending Formula
```rust
const STARTER_RATING: f64 = 500.0;
const ESTABLISHED_THRESHOLD: usize = 200;

fn blend_rating(ml_rating: f64, games_played: usize) -> f64 {
    if games_played >= ESTABLISHED_THRESHOLD {
        return ml_rating;  // Fully established
    }

    let starter_weight = (ESTABLISHED_THRESHOLD - games_played) as f64
                       / ESTABLISHED_THRESHOLD as f64;
    let ml_weight = 1.0 - starter_weight;

    starter_weight * STARTER_RATING + ml_weight * ml_rating
}
```

**Example**:
```
Player with 25 games, ML rating 623.45:
  starter_weight = (200 - 25) / 200 = 0.875
  ml_weight = 0.125
  blended = 0.875 × 500 + 0.125 × 623.45 = 515.43
```

---

## 🗄️ Database Foreign Keys

### CRITICAL: Use Database IDs, Not CueScore IDs

❌ **WRONG**:
```rust
struct Game {
    player_a_cuescore_id: String,  // "101"
    player_b_cuescore_id: String,  // "102"
}
```

✅ **CORRECT**:
```rust
struct Game {
    player_a_id: i64,  // Database ID: 1, 2, 3...
    player_b_id: i64,
}
```

**Why**: Database relations use auto-increment IDs, not CueScore IDs.

**Mapping Required**:
```rust
// When inserting games:
let player_a_db_id = get_player_by_cuescore_id("101")?;  // Returns 1
let player_b_db_id = get_player_by_cuescore_id("102")?;  // Returns 2

Game {
    player_a_id: player_a_db_id,  // 1
    player_b_id: player_b_db_id,  // 2
    winner_id: player_a_db_id,    // 1
    ...
}
```

---

## 🔍 Discipline Filtering

**Exclude these disciplines** (case-insensitive):
```rust
const EXCLUDED_DISCIPLINES: &[&str] = &[
    "snooker",
    "pyramid",
    "piramida",         // Polish spelling
    "russian pyramid",
    "russian pool",
];

fn is_pool_tournament(discipline: &str) -> bool {
    let lower = discipline.to_lowercase();
    !EXCLUDED_DISCIPLINES.iter().any(|&excl| lower.contains(excl))
}
```

**Why**: CueScore has all cue sports, we only want pool (8-ball, 9-ball, 10-ball, etc.)

---

## 📊 Bradley-Terry Input Format

### What choix expects (Python):
```python
n_players = 32321
comparisons = [
    (winner_idx, loser_idx, weight),
    (0, 1, 1.0),      # Player 0 beat Player 1
    (0, 2, 0.89),     # Player 0 beat Player 2 (older game)
    # ... 342,662 tuples
]
```

### What Rust MM algorithm needs:
```rust
// Build comparison matrix (n_players × n_players)
let mut comparison_matrix = Array2::<f64>::zeros((n_players, n_players));
let mut wins = Array1::<f64>::zeros(n_players);

for game in games {
    let i = player_to_idx[game.player_a_id];
    let j = player_to_idx[game.player_b_id];
    let weight = game.weight;

    // How many times i and j played
    comparison_matrix[[i, j]] += weight;
    comparison_matrix[[j, i]] += weight;

    // Who won
    if game.winner_id == game.player_a_id {
        wins[i] += weight;
    } else {
        wins[j] += weight;
    }
}
```

**Output**: Log-scale ratings (use `.exp()` to convert to actual ratings)

---

## 🔢 Date/Time Parsing

### Date Formats to Support
```rust
// CueScore uses multiple formats
const DATE_FORMATS: &[&str] = &[
    "%Y-%m-%d",      // 2024-03-15
    "%d-%m-%Y",      // 15-03-2024
    "%m/%d/%Y",      // 03/15/2024
];

// DateTime formats
const DATETIME_FORMATS: &[&str] = &[
    "%Y-%m-%dT%H:%M:%S",         // ISO without timezone
    "%Y-%m-%dT%H:%M:%S%z",       // ISO with timezone
    "%Y-%m-%dT%H:%M:%S%.3fZ",    // ISO with milliseconds
];
```

### Timestamp Handling
```rust
// CueScore may send Unix timestamps
fn parse_datetime(input: &str) -> Option<DateTime<Utc>> {
    // Try parsing as timestamp first
    if let Ok(timestamp) = input.parse::<i64>() {
        return Some(Utc.timestamp(timestamp, 0));
    }

    // Try ISO formats...
}
```

---

## 🚀 Performance Expectations

| Operation | Python | Rust | Notes |
|-----------|--------|------|-------|
| Scraping | 2 min | 2 min | Rate limited, same |
| API fetching | 3-5 min | 3-5 min | Rate limited, same |
| Parsing | 30 sec | 5 sec | 6x faster, no Python overhead |
| DB insertion | 6-7 min | 3-4 min | 2x faster, batch ops |
| **Rating calc** | **FAILED** | **8 sec** | **Custom MM vs choix** |
| **TOTAL** | **30+ min (FAIL)** | **10-12 min** | **Actually works** |

---

## ⚠️ Common Mistakes to Avoid

1. **Storing match scores instead of game records**
   - Match 7-5 → 12 game records, not one match record

2. **Using CueScore IDs in foreign keys**
   - Use database auto-increment IDs, map from CueScore IDs

3. **Applying time decay after rating calculation**
   - Calculate weights BEFORE Bradley-Terry input

4. **Forgetting to blend new player ratings**
   - Players < 200 games need blending with starter rating (500)

5. **Not checking parsed cache first**
   - Check `tournaments.json` before checking individual raw files

6. **Trusting choix/scipy for large datasets**
   - 30K+ players requires custom MM implementation

7. **Including non-pool disciplines**
   - Filter out snooker, pyramid variants

---

## 📝 Testing Checklist

Before deploying Rust implementation, verify:

- [ ] Match 7-5 creates 12 game records (not 1)
- [ ] Time decay weight for 3-year-old game ≈ 0.50
- [ ] Player with 25 games gets blended rating (not raw ML rating)
- [ ] Confidence level for 150 games = "emerging"
- [ ] Snooker tournaments are filtered out
- [ ] Cache check order: parsed → raw → API
- [ ] Database uses integer IDs, not CueScore string IDs
- [ ] Rating calculation completes in < 30 seconds (not 30 minutes)
- [ ] Winner constraint: `winner_id IN (player_a_id, player_b_id)`
- [ ] Ratings range: 0-2000 (constraint enforced)

---

## 🎯 Priority Order for Rust Implementation

1. **Database schema** - Must match Python exactly (frontend compatibility)
2. **Match-to-games expansion** - Critical for correct data
3. **Two-tier caching** - Performance essential
4. **Time decay** - Must be identical formula
5. **Custom MM algorithm** - Can't rely on choix
6. **Confidence & blending** - UI depends on this
7. **API parsing** - Can start simple, refine later

---

Ready to implement? Start with the database schema and match-to-games logic. Everything else builds on that foundation.
