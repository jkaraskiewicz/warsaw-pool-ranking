use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerListItem {
    pub rank: usize,
    pub player_id: i64,
    pub cuescore_id: Option<i64>,
    pub name: String,
    pub rating: f64,
    pub games_played: i32,
    pub confidence_level: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerDetail {
    pub player_id: i64,
    pub cuescore_id: Option<i64>,
    pub name: String,
    pub rating: f64,
    pub games_played: i32,
    pub confidence_level: String,
    pub ml_rating: f64,
    pub starter_weight: f64,
    pub ml_weight: f64,
    pub effective_games: i32,
    pub last_played: Option<String>,
}
