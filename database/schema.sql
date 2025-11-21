-- Warsaw Pool Ranking System - Database Schema
-- PostgreSQL Database Schema
-- Version: 1.0
-- Date: 2025-11-20

-- ============================================================
-- ENUMS
-- ============================================================

CREATE TYPE confidence_level_enum AS ENUM ('unranked', 'provisional', 'emerging', 'established');

-- ============================================================
-- TABLES
-- ============================================================

-- Players Table
-- Stores player information with CueScore IDs for linking
CREATE TABLE players (
    id SERIAL PRIMARY KEY,
    cuescore_id VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    cuescore_profile_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Venues Table
-- Stores Warsaw/Masovian pool venues
CREATE TABLE venues (
    id SERIAL PRIMARY KEY,
    cuescore_id VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    cuescore_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Tournaments Table
-- Stores tournament metadata
CREATE TABLE tournaments (
    id SERIAL PRIMARY KEY,
    cuescore_id VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    venue_id INTEGER REFERENCES venues(id) ON DELETE SET NULL,
    start_date DATE,
    end_date DATE,
    cuescore_url VARCHAR(500),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Games Table
-- Stores individual game results (match scores converted to game-level records)
-- E.g., a 7-5 match becomes 12 game records
-- All game types (8-ball, 9-ball, 10-ball) treated equally in unified ranking
CREATE TABLE games (
    id SERIAL PRIMARY KEY,
    cuescore_match_id VARCHAR(100) NOT NULL,
    tournament_id INTEGER NOT NULL REFERENCES tournaments(id) ON DELETE CASCADE,
    player_a_id INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    player_b_id INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    winner_id INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    played_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Constraint to ensure winner is one of the players
    CONSTRAINT winner_is_player CHECK (winner_id = player_a_id OR winner_id = player_b_id)
);

-- Ratings Table (Current Ratings)
-- Stores the current/latest rating for each player
-- Updated during weekly recalculation
CREATE TABLE ratings (
    id SERIAL PRIMARY KEY,
    player_id INTEGER NOT NULL UNIQUE REFERENCES players(id) ON DELETE CASCADE,
    rating FLOAT NOT NULL,
    games_played INTEGER NOT NULL DEFAULT 0,
    total_wins INTEGER NOT NULL DEFAULT 0,
    total_losses INTEGER NOT NULL DEFAULT 0,
    confidence_level confidence_level_enum NOT NULL,
    best_rating FLOAT,
    best_rating_date DATE,
    calculated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Ensure rating is reasonable
    CONSTRAINT rating_range CHECK (rating >= 0 AND rating <= 2000)
);

-- Rating Snapshots Table (Historical Ratings)
-- Stores weekly rating snapshots for history charts
-- THIS TABLE IS REPLACED ENTIRELY EACH WEEK during simulation
CREATE TABLE rating_snapshots (
    id SERIAL PRIMARY KEY,
    player_id INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    week_ending DATE NOT NULL,
    rating FLOAT NOT NULL,
    games_played INTEGER NOT NULL,
    confidence_level confidence_level_enum NOT NULL,
    calculation_version VARCHAR(10) NOT NULL DEFAULT 'v1',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Ensure rating is reasonable
    CONSTRAINT snapshot_rating_range CHECK (rating >= 0 AND rating <= 2000)
);

-- ============================================================
-- INDEXES
-- ============================================================

-- Players indexes
CREATE INDEX idx_players_cuescore_id ON players(cuescore_id);
CREATE INDEX idx_players_name ON players(name);

-- Venues indexes
CREATE INDEX idx_venues_cuescore_id ON venues(cuescore_id);

-- Tournaments indexes
CREATE INDEX idx_tournaments_cuescore_id ON tournaments(cuescore_id);
CREATE INDEX idx_tournaments_venue_id ON tournaments(venue_id);
CREATE INDEX idx_tournaments_start_date ON tournaments(start_date);

-- Games indexes
CREATE INDEX idx_games_tournament_id ON games(tournament_id);
CREATE INDEX idx_games_cuescore_match_id ON games(cuescore_match_id);
CREATE INDEX idx_games_player_a_id ON games(player_a_id);
CREATE INDEX idx_games_player_b_id ON games(player_b_id);
CREATE INDEX idx_games_winner_id ON games(winner_id);
CREATE INDEX idx_games_played_at ON games(played_at);

-- Ratings indexes
CREATE INDEX idx_ratings_player_id ON ratings(player_id);
CREATE INDEX idx_ratings_rating ON ratings(rating DESC); -- For leaderboard queries
CREATE INDEX idx_ratings_confidence ON ratings(confidence_level);

-- Rating Snapshots indexes
CREATE INDEX idx_snapshots_player_id ON rating_snapshots(player_id);
CREATE INDEX idx_snapshots_week_ending ON rating_snapshots(week_ending);
-- Composite index for fast history queries (player's rating over time)
CREATE INDEX idx_snapshots_player_week ON rating_snapshots(player_id, week_ending);

-- ============================================================
-- FUNCTIONS & TRIGGERS
-- ============================================================

-- Function to update 'updated_at' timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for updated_at
CREATE TRIGGER update_players_updated_at
    BEFORE UPDATE ON players
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_venues_updated_at
    BEFORE UPDATE ON venues
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tournaments_updated_at
    BEFORE UPDATE ON tournaments
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================
-- COMMENTS
-- ============================================================

COMMENT ON TABLE players IS 'Pool players with CueScore IDs';
COMMENT ON TABLE venues IS 'Warsaw/Masovian pool venues';
COMMENT ON TABLE tournaments IS 'Tournaments held at tracked venues';
COMMENT ON TABLE games IS 'Individual game results (match scores expanded to games)';
COMMENT ON TABLE ratings IS 'Current player ratings (updated weekly)';
COMMENT ON TABLE rating_snapshots IS 'Weekly rating history (replaced entirely each week)';

COMMENT ON COLUMN games.cuescore_match_id IS 'CueScore match ID for deduplication and linking';
COMMENT ON COLUMN rating_snapshots.calculation_version IS 'Algorithm version used (e.g., v1, v2) for tracking changes';
COMMENT ON COLUMN rating_snapshots.week_ending IS 'Sunday date ending the week this snapshot represents';
