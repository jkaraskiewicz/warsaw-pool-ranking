use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: i32,
    pub cuescore_id: Option<i64>,
    pub name: String,
    pub avatar_url: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct Tournament {
    pub id: i32,
    pub cuescore_id: i64,
    pub name: String,
    pub venue_id: i64,
    pub venue_name: String,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub id: i32,
    pub tournament_id: i32,
    pub first_player_id: i32,
    pub second_player_id: i32,
    pub first_player_score: i32,
    pub second_player_score: i32,
    pub date: NaiveDateTime,
    pub weight: f64,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone)]
pub struct DbRating {
    pub id: i32,
    pub player_id: i32,
    pub rating_type: String,
    pub rating: f64,
    pub games_played: i32,
    pub confidence_level: String,
    pub calculated_at: NaiveDateTime,
    pub created_at: Option<NaiveDateTime>,
}

// DTOs for joined queries
#[derive(Debug, Clone)]
pub struct PlayerWithRating {
    pub player_id: i32,
    pub cuescore_id: Option<i64>,
    pub name: String,
    pub avatar_url: Option<String>,
    pub rating: f64,
    pub games_played: i32,
    pub confidence_level: String,
}

#[derive(Debug, Clone)]
pub enum SortColumn {
    Name,
    Rating,
    GamesPlayed,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct PlayerFilter {
    pub name_contains: Option<String>,
    pub min_games: Option<i32>,
    pub rating_type: String,
    pub sort_by: SortColumn,
    pub sort_order: SortOrder,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct HeadToHeadMatchRow {
    pub date: NaiveDateTime,
    pub tournament_name: String,
    pub p1_wins: i32,
    pub p2_wins: i32,
}