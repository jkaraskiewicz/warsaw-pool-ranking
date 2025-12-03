use anyhow::Result;
use chrono::NaiveDateTime;

use crate::domain::models::{MatchResponse, TournamentResponse};

#[derive(Debug, Clone)]
pub struct ExpandedGame {
    pub tournament_id: i64,
    pub winner_id: i64,
    pub loser_id: i64,
    pub date: NaiveDateTime,
    pub weight: f64,
}

pub fn expand_tournament_to_games(
    tournament: &TournamentResponse,
) -> Result<Vec<ExpandedGame>> {
    let mut expanded = Vec::new();

    for match_data in &tournament.matches {
        // Skip unplayed/future matches
        if !match_data.is_played() {
            continue;
        }

        let games = expand_match_to_games(tournament.id, match_data)?;
        expanded.extend(games);
    }

    Ok(expanded)
}

fn expand_match_to_games(
    tournament_id: i64,
    match_data: &MatchResponse,
) -> Result<Vec<ExpandedGame>> {
    let mut games = Vec::new();
    let date_str = match_data.get_played_at();
    let date = parse_date_string(&date_str)?;

    let first_wins = create_games_for_winner(
        tournament_id,
        match_data.player_a_id(),
        match_data.player_b_id(),
        match_data.get_score_a(),
        date,
    );

    let second_wins = create_games_for_winner(
        tournament_id,
        match_data.player_b_id(),
        match_data.player_a_id(),
        match_data.get_score_b(),
        date,
    );

    games.extend(first_wins);
    games.extend(second_wins);

    Ok(games)
}

fn parse_date_string(date_str: &str) -> Result<NaiveDateTime> {
    use chrono::{DateTime, NaiveDateTime as ND};

    // Try RFC3339 format (with timezone)
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.naive_utc());
    }

    // Try naive datetime format (without timezone)
    if let Ok(dt) = ND::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt);
    }

    // Try with fractional seconds
    if let Ok(dt) = ND::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.f") {
        return Ok(dt);
    }

    anyhow::bail!("Failed to parse date: {}", date_str)
}

fn create_games_for_winner(
    tournament_id: i64,
    winner_id: i64,
    loser_id: i64,
    win_count: i32,
    date: NaiveDateTime,
) -> Vec<ExpandedGame> {
    (0..win_count)
        .map(|_| build_game(tournament_id, winner_id, loser_id, date))
        .collect()
}

fn build_game(
    tournament_id: i64,
    winner_id: i64,
    loser_id: i64,
    date: NaiveDateTime,
) -> ExpandedGame {
    ExpandedGame {
        tournament_id,
        winner_id,
        loser_id,
        date,
        weight: 1.0,
    }
}

pub fn count_total_games(tournament: &TournamentResponse) -> usize {
    tournament
        .matches
        .iter()
        .map(|m| count_games_in_match(m))
        .sum()
}

fn count_games_in_match(match_data: &MatchResponse) -> usize {
    (match_data.get_score_a() + match_data.get_score_b()) as usize
}
