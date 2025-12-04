use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::{params, OptionalExtension};

use super::connection::DbConn;
use super::models::DbRating;

pub fn insert_rating(
    conn: &mut DbConn,
    player_id: i32,
    rating_type: &str,
    rating: f64,
    games_played: i32,
    confidence_level: &str,
    calculated_at: NaiveDateTime,
) -> Result<DbRating> {
    let sql = "INSERT INTO ratings (player_id, rating_type, rating, games_played, confidence_level, calculated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6) RETURNING id, player_id, rating_type, rating, games_played, confidence_level, calculated_at, created_at";

    conn.query_row(
        sql,
        params![player_id, rating_type, rating, games_played, confidence_level, calculated_at],
        parse_db_rating_row,
    )
    .context("Failed to insert rating")
}

fn parse_db_rating_row(row: &rusqlite::Row) -> rusqlite::Result<DbRating> {
    Ok(DbRating {
        id: row.get(0)?,
        player_id: row.get(1)?,
        rating_type: row.get(2)?,
        rating: row.get(3)?,
        games_played: row.get(4)?,
        confidence_level: row.get(5)?,
        calculated_at: row.get(6)?,
        created_at: row.get(7)?,
    })
}

pub fn list_by_player(
    conn: &mut DbConn,
    player_id: i32,
    rating_type: &str,
) -> Result<Vec<DbRating>> {
    let sql = "SELECT id, player_id, rating_type, rating, games_played, confidence_level, calculated_at, created_at FROM ratings WHERE player_id = ?1 AND rating_type = ?2 ORDER BY calculated_at DESC";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map(params![player_id, rating_type], parse_db_rating_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(rows)
}

pub fn get_latest_for_player(
    conn: &mut DbConn,
    player_id: i32,
    rating_type: &str,
) -> Result<Option<DbRating>> {
    let sql = "SELECT id, player_id, rating_type, rating, games_played, confidence_level, calculated_at, created_at FROM ratings WHERE player_id = ?1 AND rating_type = ?2 ORDER BY calculated_at DESC LIMIT 1";

    conn.query_row(sql, params![player_id, rating_type], parse_db_rating_row)
        .optional()
        .context("Failed to get latest rating for player")
}