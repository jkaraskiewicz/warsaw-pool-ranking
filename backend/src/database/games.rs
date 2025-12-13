use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::params;

use super::connection::DbConn;
use super::models::Game;

#[allow(clippy::too_many_arguments)]
pub fn insert_game(
    conn: &mut DbConn,
    tournament_id: i32,
    first_player_id: i32,
    second_player_id: i32,
    first_player_score: i32,
    second_player_score: i32,
    date: NaiveDateTime,
    weight: f64,
) -> Result<Game> {
    let sql = "INSERT INTO games (tournament_id, first_player_id, second_player_id, first_player_score, second_player_score, date, weight) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) RETURNING id, tournament_id, first_player_id, second_player_id, first_player_score, second_player_score, date, weight, created_at";

    conn.query_row(
        sql,
        params![
            tournament_id,
            first_player_id,
            second_player_id,
            first_player_score,
            second_player_score,
            date,
            weight
        ],
        parse_game_row,
    )
    .context("Failed to insert game")
}

fn parse_game_row(row: &rusqlite::Row) -> rusqlite::Result<Game> {
    Ok(Game {
        id: row.get(0)?,
        tournament_id: row.get(1)?,
        first_player_id: row.get(2)?,
        second_player_id: row.get(3)?,
        first_player_score: row.get(4)?,
        second_player_score: row.get(5)?,
        date: row.get(6)?,
        weight: row.get(7)?,
        created_at: row.get(8)?,
    })
}

pub fn list_all(conn: &mut DbConn) -> Result<Vec<Game>> {
    let sql = "SELECT id, tournament_id, first_player_id, second_player_id, first_player_score, second_player_score, date, weight, created_at FROM games";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map([], parse_game_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(rows)
}

pub fn list_by_tournament(
    conn: &mut DbConn,
    tournament_id: i32,
) -> Result<Vec<Game>> {
    let sql = "SELECT id, tournament_id, first_player_id, second_player_id, first_player_score, second_player_score, date, weight, created_at FROM games WHERE tournament_id = ?1";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(params![tournament_id], parse_game_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(rows)
}

pub fn get_head_to_head_matches(
    conn: &mut DbConn,
    player1_id: i32,
    player2_id: i32,
) -> Result<Vec<super::models::HeadToHeadMatchRow>> {
    let sql = "
        SELECT
            g.date,
            t.name as tournament_name,
            SUM(CASE WHEN (g.first_player_id = ?1 AND g.first_player_score > g.second_player_score) OR (g.second_player_id = ?1 AND g.second_player_score > g.first_player_score) THEN 1 ELSE 0 END) as p1_wins,
            SUM(CASE WHEN (g.first_player_id = ?2 AND g.first_player_score > g.second_player_score) OR (g.second_player_id = ?2 AND g.second_player_score > g.first_player_score) THEN 1 ELSE 0 END) as p2_wins
        FROM games g
        JOIN tournaments t ON g.tournament_id = t.id
        WHERE (g.first_player_id = ?1 AND g.second_player_id = ?2)
           OR (g.first_player_id = ?2 AND g.second_player_id = ?1)
        GROUP BY g.tournament_id, t.name, g.date
        ORDER BY g.date DESC
    ";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![player1_id, player2_id], |row| {
        Ok(super::models::HeadToHeadMatchRow {
            date: row.get(0)?,
            tournament_name: row.get(1)?,
            p1_wins: row.get(2)?,
            p2_wins: row.get(3)?,
        })
    })?.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(rows)
}

pub fn count_matches_played_for_player(
    conn: &mut DbConn,
    player_id: i32,
) -> Result<i32> {
    let sql = "SELECT COUNT(DISTINCT date) FROM games WHERE first_player_id = ?1 OR second_player_id = ?1";
    conn.query_row(sql, params![player_id], |row| row.get(0))
        .context("Failed to count matches played for player")
}

pub fn get_player_last_matches(
    conn: &mut DbConn,
    player_id: i32,
    limit: usize,
) -> Result<Vec<super::models::MatchResultRow>> {
    let sql = "
        SELECT
            g.date,
            t.name as tournament_name,
            CASE
                WHEN g.first_player_id = ?1 THEN p2.name
                ELSE p1.name
            END as opponent_name,
            CASE
                WHEN g.first_player_id = ?1 THEN p2.id
                ELSE p1.id
            END as opponent_id,
            SUM(CASE WHEN g.first_player_id = ?1 THEN g.first_player_score ELSE g.second_player_score END) as player_total_score,
            SUM(CASE WHEN g.first_player_id = ?1 THEN g.second_player_score ELSE g.first_player_score END) as opponent_total_score
        FROM games g
        JOIN tournaments t ON g.tournament_id = t.id
        JOIN players p1 ON g.first_player_id = p1.id
        JOIN players p2 ON g.second_player_id = p2.id
        WHERE g.first_player_id = ?1 OR g.second_player_id = ?1
        GROUP BY g.date, t.name, opponent_name, opponent_id
        ORDER BY g.date DESC
        LIMIT ?2
    ";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![player_id, limit as i64], |row| {
        Ok(super::models::MatchResultRow {
            date: row.get(0)?,
            tournament_name: row.get(1)?,
            opponent_name: row.get(2)?,
            opponent_id: row.get(3)?,
            player_total_score: row.get(4)?,
            opponent_total_score: row.get(5)?,
        })
    })?.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(rows)
}
