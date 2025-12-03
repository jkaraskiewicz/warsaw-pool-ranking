use std::collections::HashMap;

pub type PlayerId = i32;
pub type RatingValue = f64;
pub type RatingMap = HashMap<PlayerId, RatingValue>;

#[derive(Debug, Clone)]
pub struct PlayerRating {
    pub player_id: PlayerId,
    pub rating: RatingValue,
    pub games_played: i32,
    pub confidence_level: ConfidenceLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
}

impl ConfidenceLevel {
    pub fn from_games_played(games: i32) -> Self {
        if games < 10 {
            ConfidenceLevel::Low
        } else if games < 30 {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::High
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ConfidenceLevel::Low => "low",
            ConfidenceLevel::Medium => "medium",
            ConfidenceLevel::High => "high",
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameResult {
    pub winner_id: PlayerId,
    pub loser_id: PlayerId,
    pub weight: f64,
}
