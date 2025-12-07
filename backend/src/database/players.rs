use anyhow::{Context, Result};
use rusqlite::{params, OptionalExtension};

use super::connection::DbConn;
use super::models::Player;

pub fn upsert_player(
    conn: &mut DbConn,
    cuescore_id: i64,
    name: &str,
    avatar_url: Option<&str>,
) -> Result<Player> {
    if let Some(mut existing) = find_by_cuescore_id(conn, cuescore_id)? {
        // If avatar_url is now available, update it
        if existing.avatar_url.is_none() && avatar_url.is_some() {
            let sql = "UPDATE players SET avatar_url = ?1 WHERE id = ?2 RETURNING id, cuescore_id, name, avatar_url, created_at";
            existing = conn.query_row(sql, params![avatar_url, existing.id], parse_player_row)?;
        }
        return Ok(existing);
    }

    insert_new_player(conn, cuescore_id, name, avatar_url)
}

fn find_by_cuescore_id(
    conn: &mut DbConn,
    cuescore_id: i64,
) -> Result<Option<Player>> {
    let sql = "SELECT id, cuescore_id, name, avatar_url, created_at FROM players WHERE cuescore_id = ?1";

    conn.query_row(sql, params![cuescore_id], parse_player_row)
        .optional()
        .context("Failed to query player by cuescore_id")
}

fn insert_new_player(
    conn: &mut DbConn,
    cuescore_id: i64,
    name: &str,
    avatar_url: Option<&str>,
) -> Result<Player> {
    let sql = "INSERT INTO players (cuescore_id, name, avatar_url) VALUES (?1, ?2, ?3) RETURNING id, cuescore_id, name, avatar_url, created_at";

    conn.query_row(sql, params![cuescore_id, name, avatar_url], parse_player_row)
        .context("Failed to insert new player")
}

fn parse_player_row(row: &rusqlite::Row) -> rusqlite::Result<Player> {
    Ok(Player {
        id: row.get(0)?,
        cuescore_id: row.get(1)?,
        name: row.get(2)?,
        avatar_url: row.get(3)?,
        created_at: row.get(4)?,
    })
}

pub fn find_by_id(conn: &mut DbConn, id: i32) -> Result<Option<Player>> {
    let sql = "SELECT id, cuescore_id, name, avatar_url, created_at FROM players WHERE id = ?1";

    conn.query_row(sql, params![id], parse_player_row)
        .optional()
        .context("Failed to query player by id")
}

pub fn list_all(conn: &mut DbConn) -> Result<Vec<Player>> {
    let sql = "SELECT id, cuescore_id, name, avatar_url, created_at FROM players";

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map([], parse_player_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(rows)
}