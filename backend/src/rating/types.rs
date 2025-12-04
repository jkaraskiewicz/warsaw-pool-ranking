use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub type PlayerId = i32;
pub type RatingValue = f64;
pub type RatingMap = HashMap<PlayerId, RatingValue>;

#[derive(Debug, Clone)]
pub struct PlayerRating {
    pub player_id: PlayerId,
    pub rating_type: String,
    pub rating: RatingValue,
    pub games_played: i32,
    pub confidence_level: ConfidenceLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    Unranked,    // < 10 games
    Provisional, // 10-49 games
    Emerging,    // 50-199 games
    Established, // 200+ games
}

impl ConfidenceLevel {
    pub fn from_games_played(games: i32) -> Self {
        // These thresholds match FargoRate's logic
        if games < 10 {
            ConfidenceLevel::Unranked
        } else if games < 50 {
            ConfidenceLevel::Provisional
        } else if games < 200 {
            ConfidenceLevel::Emerging
        } else {
            ConfidenceLevel::Established
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ConfidenceLevel::Unranked => "unranked",
            ConfidenceLevel::Provisional => "provisional",
            ConfidenceLevel::Emerging => "emerging",
            ConfidenceLevel::Established => "established",
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameResult {
    pub winner_id: PlayerId,
    pub loser_id: PlayerId,
    pub weight: f64,
}
