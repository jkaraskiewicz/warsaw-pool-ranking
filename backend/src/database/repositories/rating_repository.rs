use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::params;
use crate::database::connection::DbConn;
use crate::database::models::DbRating;

pub struct RatingRepository;

impl RatingRepository {
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
            |row| {
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
            },
        )
        .context("Failed to insert rating")
    }
}
