use serde::{Deserialize, Serialize};

// --- Cuescore API Response Structures ---

/// Raw tournament API response from CueScore
#[derive(Debug, Deserialize, Serialize)]
pub struct TournamentResponse {
    #[serde(rename = "tournamentId")]
    pub id: i64,
    pub name: String,
    pub starttime: String,
    pub stoptime: Option<String>,
    #[serde(rename = "type")]
    pub tournament_type: Option<i32>,
    pub format: Option<i32>,
    pub breakrule: Option<String>,
    pub description: Option<String>,
    pub discipline: Option<String>,
    pub venues: Option<Vec<VenueInfo>>,
    #[serde(default)]
    pub banner: serde_json::Value,
    #[serde(default)]
    pub dresscode: Option<String>,
    #[serde(rename = "defaultRaceTo", default)]
    pub default_race_to: Option<i32>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(rename = "displayDate", default)]
    pub display_date: Option<String>,
    #[serde(default)]
    pub deadline: Option<String>,
    pub matches: Vec<MatchResponse>,
}

impl TournamentResponse {
    pub fn venue_id(&self) -> i64 {
        self.venues
            .as_ref()
            .and_then(|v| v.first())
            .map(|v| v.venue_id)
            .unwrap_or(0)
    }

    pub fn venue_name(&self) -> String {
        self.venues
            .as_ref()
            .and_then(|v| v.first())
            .map(|v| v.name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VenueInfo {
    #[serde(rename = "venueId")]
    pub venue_id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerInfo {
    #[serde(rename = "playerId")]
    pub player_id: Option<i64>,
    #[serde(rename = "teamId")]
    pub team_id: Option<i64>,
    pub name: String,
    pub url: Option<String>,
    pub image: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
}

/// Raw match API response from CueScore
#[derive(Debug, Deserialize, Serialize)]
pub struct MatchResponse {
    #[serde(rename = "matchId")]
    pub match_id: i64,
    #[serde(rename = "playerA")]
    pub player_a: PlayerInfo,
    #[serde(rename = "playerB")]
    pub player_b: PlayerInfo,
    #[serde(rename = "scoreA")]
    pub score_a: i32,
    #[serde(rename = "scoreB")]
    pub score_b: i32,
    #[serde(default)]
    pub starttime: String,
    #[serde(default)]
    pub stoptime: Option<String>,
}

impl MatchResponse {
    pub fn get_id(&self) -> i64 {
        self.match_id
    }

    pub fn player_a_id(&self) -> i64 {
        self.player_a.player_id.unwrap_or(0)
    }

    pub fn player_b_id(&self) -> i64 {
        self.player_b.player_id.unwrap_or(0)
    }

    pub fn player_a_name(&self) -> String {
        self.player_a.name.clone()
    }

    pub fn player_b_name(&self) -> String {
        self.player_b.name.clone()
    }

    pub fn get_score_a(&self) -> i32 {
        self.score_a
    }

    pub fn get_score_b(&self) -> i32 {
        self.score_b
    }

    pub fn get_played_at(&self) -> String {
        self.stoptime
            .as_ref()
            .filter(|s| !s.is_empty())
            .cloned()
            .or_else(|| {
                if self.starttime.is_empty() {
                    None
                } else {
                    Some(self.starttime.clone())
                }
            })
            .unwrap_or_else(|| "2024-01-01T00:00:00".to_string())
    }

    /// Check if this match has been played (has valid player IDs and scores)
    pub fn is_played(&self) -> bool {
        self.player_a.player_id.unwrap_or(0) > 0
            && self.player_b.player_id.unwrap_or(0) > 0
            && !self.starttime.is_empty()
            && (self.score_a > 0 || self.score_b > 0)
    }
}
