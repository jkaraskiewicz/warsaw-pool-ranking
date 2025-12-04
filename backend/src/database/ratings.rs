use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::{params, OptionalExtension};

use super::connection::DbConn;
use super::models::Rating;

pub fn insert_rating(
    conn: &mut DbConn,
    player_id: i32,
    rating: f64,
    games_played: i32,
    confidence_level: &str,
    calculated_at: NaiveDateTime,
) -> Result<Rating> {
    let sql = "INSERT INTO ratings (player_id, rating, games_played, confidence_level, calculated_at) VALUES (?1, ?2, ?3, ?4, ?5) RETURNING id, player_id, rating, games_played, confidence_level, calculated_at, created_at";

    conn.query_row(
        sql,
        params![player_id, rating, games_played, confidence_level, calculated_at],
        parse_rating_row,
    )
    .context("Failed to insert rating")
}

fn parse_rating_row(row: &rusqlite::Row) -> rusqlite::Result<Rating> {
    Ok(Rating {
        id: row.get(0)?,
        player_id: row.get(1)?,
        rating: row.get(2)?,
        games_played: row.get(3)?,
        confidence_level: row.get(4)?,
        calculated_at: row.get(5)?,
        created_at: row.get(6)?,
    })
}

pub fn list_by_player(
    conn: &mut DbConn,
    player_id: i32,
) -> Result<Vec<Rating>> {
    let sql = "SELECT id, player_id, rating, games_played, confidence_level, calculated_at, created_at FROM ratings WHERE player_id = ?1 ORDER BY calculated_at DESC";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(params![player_id], parse_rating_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(rows)
}

pub fn get_latest_for_player(
    conn: &mut DbConn,
    player_id: i32,
) -> Result<Option<Rating>> {
    let sql = "SELECT id, player_id, rating, games_played, confidence_level, calculated_at, created_at FROM ratings WHERE player_id = ?1 ORDER BY calculated_at DESC LIMIT 1";

    conn.query_row(sql, params![player_id], parse_rating_row)
        .optional()
        .context("Failed to get latest rating for player")
}
