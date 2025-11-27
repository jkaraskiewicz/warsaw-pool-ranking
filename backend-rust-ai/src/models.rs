use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Tournament data from CueScore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tournament {
    pub id: i64,
    pub name: String,
    pub venue_id: i64,
    pub venue_name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Player data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: i64,
    pub name: String,
    pub cuescore_id: Option<i64>,
}

/// Game/Match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: i64,
    pub tournament_id: i64,
    pub player1_id: i64,
    pub player2_id: i64,
    pub player1_score: i32,
    pub player2_score: i32,
    pub date: DateTime<Utc>,
    pub weight: f64, // Time decay weight
}

/// Player rating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    pub player_id: i64,
    pub rating: f64,
    pub games_played: i32,
    pub confidence_level: ConfidenceLevel,
    pub calculated_at: DateTime<Utc>,
}

/// Confidence level based on games played
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    Unranked,      // < 10 games
    Provisional,   // 10-49 games
    Emerging,      // 50-199 games
    Established,   // 200+ games
}

/// Venue information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Venue {
    pub id: i64,
    pub name: String,
}
