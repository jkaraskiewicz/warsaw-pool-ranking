-- SQLite Schema

DROP TABLE IF EXISTS ratings;
DROP TABLE IF EXISTS games;
DROP TABLE IF EXISTS tournaments;
DROP TABLE IF EXISTS players;

CREATE TABLE players (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cuescore_id INTEGER UNIQUE,
    name TEXT NOT NULL,
    avatar_url TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_players_cuescore_id ON players(cuescore_id);

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

CREATE TABLE ratings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    player_id INTEGER NOT NULL REFERENCES players(id),
    rating_type TEXT NOT NULL,
    rating REAL NOT NULL,
    games_played INTEGER NOT NULL,
    confidence_level TEXT NOT NULL,
    calculated_at TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ratings_player ON ratings(player_id);
CREATE INDEX idx_ratings_type_rank ON ratings(rating_type, rating DESC);
CREATE INDEX idx_ratings_calculated_at ON ratings(calculated_at);