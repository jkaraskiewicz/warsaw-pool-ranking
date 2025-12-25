use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::{params, OptionalExtension};
use crate::database::connection::DbConn;
use crate::database::models::Tournament;

pub struct TournamentRepository;

impl TournamentRepository {
    pub fn upsert_tournament(
        conn: &mut DbConn,
        cuescore_id: i64,
        name: &str,
        venue_id: i64,
        venue_name: &str,
        start_date: NaiveDateTime,
        end_date: Option<NaiveDateTime>,
    ) -> Result<Tournament> {
        // Check if tournament exists
        let existing: Option<Tournament> = conn.query_row(
            "SELECT id, cuescore_id, name, venue_id, venue_name, start_date, end_date, created_at FROM tournaments WHERE cuescore_id = ?1",
            params![cuescore_id],
            |row| {
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
        ).optional().context("Failed to query tournament by cuescore_id")?;

        if let Some(existing_tournament) = existing {
            return Ok(existing_tournament);
        }

        // Insert new tournament
        let sql = "INSERT INTO tournaments (cuescore_id, name, venue_id, venue_name, start_date, end_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6) RETURNING id, cuescore_id, name, venue_id, venue_name, start_date, end_date, created_at";

        conn.query_row(
            sql,
            params![cuescore_id, name, venue_id, venue_name, start_date, end_date],
            |row| {
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
            },
        )
        .context("Failed to insert new tournament")
    }
}
