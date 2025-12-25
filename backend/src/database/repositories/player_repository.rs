use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::{params, OptionalExtension};
use crate::database::connection::DbConn;
use crate::database::models::{Player, PlayerFilter, PlayerWithRating, SortColumn, SortOrder};

pub struct PlayerRepository;

impl PlayerRepository {
    pub fn list_ranked_players(
        conn: &mut DbConn,
        filter: &PlayerFilter,
    ) -> Result<(Vec<PlayerWithRating>, usize)> {
        let mut where_clauses = Vec::new();
        let mut count_params: Vec<&dyn rusqlite::ToSql> = Vec::new();
        let mut query_params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    
        // Add DSL-generated filters
        if !filter.sql_filter.is_empty() {
            where_clauses.push(filter.sql_filter.where_clause.as_str());
            for param in &filter.sql_filter.params {
                count_params.push(param.as_ref());
                query_params.push(param.as_ref());
            }
        }
    
        // Store min_games for later use (can't push reference to local variable)
        let min_games_holder = filter.min_games;
        if let Some(ref min_games) = min_games_holder {
            where_clauses.push("r.games_played >= ?");
            count_params.push(min_games);
            query_params.push(min_games);
        }

        // Add last_played_cutoff filter for active players
        let last_played_cutoff_holder = filter.last_played_cutoff;
        if let Some(ref cutoff) = last_played_cutoff_holder {
            where_clauses.push("p.last_played >= ?");
            count_params.push(cutoff);
            query_params.push(cutoff);
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
        let total: usize = conn.query_row(&count_sql, rusqlite::params_from_iter(&count_params), |row| row.get(0))?;
    
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
            "SELECT p.id, p.cuescore_id, p.name, p.avatar_url, r.rating, r.games_played, r.confidence_level
             FROM players p
             JOIN ratings r ON p.id = r.player_id
             {}
             ORDER BY {} {}
             LIMIT ? OFFSET ?",
            where_sql, sort_col, sort_dir
        );
    
        let limit_value = filter.limit as i64;
        let offset_value = filter.offset as i64;
        query_params.push(&limit_value);
        query_params.push(&offset_value);
    
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(&query_params), |row| {
            Ok(PlayerWithRating {
                player_id: row.get(0)?,
                cuescore_id: row.get(1)?,
                name: row.get(2)?,
                avatar_url: row.get(3)?,
                rating: row.get(4)?,
                games_played: row.get(5)?,
                confidence_level: row.get(6)?,
            })
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
    
        Ok((rows, total))
    }

    pub fn get_player_rating_detail(
        conn: &mut DbConn,
        player_id: i32,
        rating_type: &str,
    ) -> Result<Option<PlayerWithRating>> {
        let sql = "
            SELECT p.id, p.cuescore_id, p.name, p.avatar_url, r.rating, r.games_played, r.confidence_level
            FROM players p
            JOIN ratings r ON p.id = r.player_id
            WHERE p.id = ?1 AND r.rating_type = ?2
        ";
    
        conn.query_row(sql, params![player_id, rating_type], |row| {
            Ok(PlayerWithRating {
                player_id: row.get(0)?,
                cuescore_id: row.get(1)?,
                name: row.get(2)?,
                avatar_url: row.get(3)?,
                rating: row.get(4)?,
                games_played: row.get(5)?,
                confidence_level: row.get(6)?,
            })
        }).optional().context("Failed to get player rating detail")
    }

    pub fn get_player_last_match_date(
        conn: &mut DbConn,
        player_id: i32,
    ) -> Result<Option<String>> {
        conn.query_row(
            "SELECT MAX(date) FROM games WHERE first_player_id = ?1 OR second_player_id = ?1",
            params![player_id],
            |r| r.get(0)
        ).optional().context("Failed to get player last match date")
    }

    pub fn find_by_id(conn: &mut DbConn, id: i32) -> Result<Option<Player>> {
        let sql = "SELECT id, cuescore_id, name, avatar_url, last_played, created_at FROM players WHERE id = ?1";

        conn.query_row(sql, params![id], |row| {
            Ok(Player {
                id: row.get(0)?,
                cuescore_id: row.get(1)?,
                name: row.get(2)?,
                avatar_url: row.get(3)?,
                last_played: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).optional().context("Failed to query player by id")
    }

    pub fn list_all(conn: &mut DbConn) -> Result<Vec<Player>> {
        let sql = "SELECT id, cuescore_id, name, avatar_url, last_played, created_at FROM players";

        let mut stmt = conn.prepare(sql)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Player {
                    id: row.get(0)?,
                    cuescore_id: row.get(1)?,
                    name: row.get(2)?,
                    avatar_url: row.get(3)?,
                    last_played: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(rows)
    }

    pub fn update_player_last_played(
        conn: &mut DbConn,
        player_id: i32,
        last_played: NaiveDateTime,
    ) -> Result<()> {
        let sql = "UPDATE players SET last_played = ?1 WHERE id = ?2";
        conn.execute(sql, params![last_played, player_id])
            .context("Failed to update player last_played date")?;
        Ok(())
    }

    pub fn upsert_player(
        conn: &mut DbConn,
        cuescore_id: i64,
        name: &str,
        avatar_url: Option<&str>,
    ) -> Result<Player> {
        // Check if player exists
        let existing: Option<Player> = conn.query_row(
            "SELECT id, cuescore_id, name, avatar_url, last_played, created_at FROM players WHERE cuescore_id = ?1",
            params![cuescore_id],
            |row| {
                Ok(Player {
                    id: row.get(0)?,
                    cuescore_id: row.get(1)?,
                    name: row.get(2)?,
                    avatar_url: row.get(3)?,
                    last_played: row.get(4)?,
                    created_at: row.get(5)?,
                })
            }
        ).optional().context("Failed to query player by cuescore_id")?;

        if let Some(mut existing_player) = existing {
            // If avatar_url is now available, update it
            if existing_player.avatar_url.is_none() && avatar_url.is_some() {
                let sql = "UPDATE players SET avatar_url = ?1 WHERE id = ?2 RETURNING id, cuescore_id, name, avatar_url, last_played, created_at";
                existing_player = conn.query_row(sql, params![avatar_url, existing_player.id], |row| {
                    Ok(Player {
                        id: row.get(0)?,
                        cuescore_id: row.get(1)?,
                        name: row.get(2)?,
                        avatar_url: row.get(3)?,
                        last_played: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                })?;
            }
            return Ok(existing_player);
        }

        // Insert new player
        let sql = "INSERT INTO players (cuescore_id, name, avatar_url, last_played) VALUES (?1, ?2, ?3, NULL) RETURNING id, cuescore_id, name, avatar_url, last_played, created_at";

        conn.query_row(sql, params![cuescore_id, name, avatar_url], |row| {
            Ok(Player {
                id: row.get(0)?,
                cuescore_id: row.get(1)?,
                name: row.get(2)?,
                avatar_url: row.get(3)?,
                last_played: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).context("Failed to insert new player")
    }
}