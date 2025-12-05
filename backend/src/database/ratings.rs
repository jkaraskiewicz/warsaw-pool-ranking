use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::{params, OptionalExtension};

use super::connection::DbConn;
use super::models::{DbRating, PlayerWithRating, PlayerFilter, SortColumn, SortOrder};

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

pub fn get_player_rating_detail(
    conn: &mut DbConn,
    player_id: i32,
    rating_type: &str,
) -> Result<Option<PlayerWithRating>> {
    let sql = "
        SELECT p.id, p.cuescore_id, p.name, r.rating, r.games_played, r.confidence_level 
        FROM players p 
        JOIN ratings r ON p.id = r.player_id 
        WHERE p.id = ?1 AND r.rating_type = ?2
    ";

    conn.query_row(sql, params![player_id, rating_type], |row| {
        Ok(PlayerWithRating {
            player_id: row.get(0)?,
            cuescore_id: row.get(1)?,
            name: row.get(2)?,
            rating: row.get(3)?,
            games_played: row.get(4)?,
            confidence_level: row.get(5)?,
        })
    }).optional().context("Failed to get player rating detail")
}

pub fn list_ranked_players(
    conn: &mut DbConn,
    filter: &PlayerFilter,
) -> Result<(Vec<PlayerWithRating>, usize)> {
    let mut where_clauses = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    // Mandatory filters
    where_clauses.push("r.rating_type = ?");
    params.push(Box::new(filter.rating_type.clone()));

    if let Some(min_games) = filter.min_games {
        where_clauses.push("r.games_played >= ?");
        params.push(Box::new(min_games));
    }

    if let Some(name_filter) = &filter.name_contains {
        where_clauses.push("p.name LIKE ?");
        params.push(Box::new(format!("%{}%", name_filter)));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Count
    let count_sql = format!(
        "SELECT COUNT(*) FROM players p JOIN ratings r ON p.id = r.player_id {}",
        where_sql
    );
    let total: usize = conn.query_row(&count_sql, rusqlite::params_from_iter(params.iter()), |row| row.get(0))?;

    // Sort
    let sort_col = match filter.sort_by {
        SortColumn::Name => "p.name",
        SortColumn::Rating => "r.rating",
        SortColumn::GamesPlayed => "r.games_played",
    };
    let sort_dir = match filter.sort_order {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };

    let sql = format!(
        "SELECT p.id, p.cuescore_id, p.name, r.rating, r.games_played, r.confidence_level 
         FROM players p 
         JOIN ratings r ON p.id = r.player_id 
         {} 
         ORDER BY {} {} 
         LIMIT ? OFFSET ?",
        where_sql, sort_col, sort_dir
    );

    params.push(Box::new(filter.limit as i64));
    params.push(Box::new(filter.offset as i64));

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(PlayerWithRating {
            player_id: row.get(0)?,
            cuescore_id: row.get(1)?,
            name: row.get(2)?,
            rating: row.get(3)?,
            games_played: row.get(4)?,
            confidence_level: row.get(5)?,
        })
    })?.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok((rows, total))
}