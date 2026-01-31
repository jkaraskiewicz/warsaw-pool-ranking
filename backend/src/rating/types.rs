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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_level_unranked_below_10() {
        assert_eq!(ConfidenceLevel::from_games_played(0), ConfidenceLevel::Unranked);
        assert_eq!(ConfidenceLevel::from_games_played(5), ConfidenceLevel::Unranked);
        assert_eq!(ConfidenceLevel::from_games_played(9), ConfidenceLevel::Unranked);
    }

    #[test]
    fn test_confidence_level_provisional_10_to_49() {
        assert_eq!(ConfidenceLevel::from_games_played(10), ConfidenceLevel::Provisional);
        assert_eq!(ConfidenceLevel::from_games_played(25), ConfidenceLevel::Provisional);
        assert_eq!(ConfidenceLevel::from_games_played(49), ConfidenceLevel::Provisional);
    }

    #[test]
    fn test_confidence_level_emerging_50_to_199() {
        assert_eq!(ConfidenceLevel::from_games_played(50), ConfidenceLevel::Emerging);
        assert_eq!(ConfidenceLevel::from_games_played(100), ConfidenceLevel::Emerging);
        assert_eq!(ConfidenceLevel::from_games_played(199), ConfidenceLevel::Emerging);
    }

    #[test]
    fn test_confidence_level_established_200_plus() {
        assert_eq!(ConfidenceLevel::from_games_played(200), ConfidenceLevel::Established);
        assert_eq!(ConfidenceLevel::from_games_played(500), ConfidenceLevel::Established);
        assert_eq!(ConfidenceLevel::from_games_played(1000), ConfidenceLevel::Established);
    }

    #[test]
    fn test_confidence_level_as_str() {
        assert_eq!(ConfidenceLevel::Unranked.as_str(), "unranked");
        assert_eq!(ConfidenceLevel::Provisional.as_str(), "provisional");
        assert_eq!(ConfidenceLevel::Emerging.as_str(), "emerging");
        assert_eq!(ConfidenceLevel::Established.as_str(), "established");
    }
}
