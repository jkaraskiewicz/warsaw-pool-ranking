use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::{params, OptionalExtension};

use super::connection::DbConn;
use super::models::Tournament;

pub fn upsert_tournament(
    conn: &mut DbConn,
    cuescore_id: i64,
    name: &str,
    venue_id: i64,
    venue_name: &str,
    start_date: NaiveDateTime,
    end_date: Option<NaiveDateTime>,
) -> Result<Tournament> {
    if let Some(existing) = find_by_cuescore_id(conn, cuescore_id)? {
        return Ok(existing);
    }

    insert_new_tournament(
        conn,
        cuescore_id,
        name,
        venue_id,
        venue_name,
        start_date,
        end_date,
    )
}

fn find_by_cuescore_id(
    conn: &mut DbConn,
    cuescore_id: i64,
) -> Result<Option<Tournament>> {
    let sql = "SELECT id, cuescore_id, name, venue_id, venue_name, start_date, end_date, created_at FROM tournaments WHERE cuescore_id = ?1";

    conn.query_row(sql, params![cuescore_id], parse_tournament_row)
        .optional()
        .context("Failed to query tournament by cuescore_id")
}

fn insert_new_tournament(
    conn: &mut DbConn,
    cuescore_id: i64,
    name: &str,
    venue_id: i64,
    venue_name: &str,
    start_date: NaiveDateTime,
    end_date: Option<NaiveDateTime>,
) -> Result<Tournament> {
    let sql = "INSERT INTO tournaments (cuescore_id, name, venue_id, venue_name, start_date, end_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6) RETURNING id, cuescore_id, name, venue_id, venue_name, start_date, end_date, created_at";

    conn.query_row(
        sql,
        params![cuescore_id, name, venue_id, venue_name, start_date, end_date],
        parse_tournament_row,
    )
    .context("Failed to insert new tournament")
}

fn parse_tournament_row(row: &rusqlite::Row) -> rusqlite::Result<Tournament> {
    Ok(Tournament {
        id: row.get(0)?,
        cuescore_id: row.get(1)?,
        name: row.get(2)?,
        venue_id: row.get(3)?,
        venue_name: row.get(4)?,
        start_date: row.get(5)?,
        end_date: row.get(6)?,
        created_at: row.get(7)?,
    })
}

pub fn find_by_id(conn: &mut DbConn, id: i32) -> Result<Option<Tournament>> {
    let sql = "SELECT id, cuescore_id, name, venue_id, venue_name, start_date, end_date, created_at FROM tournaments WHERE id = ?1";

    conn.query_row(sql, params![id], parse_tournament_row)
        .optional()
        .context("Failed to query tournament by id")
}
