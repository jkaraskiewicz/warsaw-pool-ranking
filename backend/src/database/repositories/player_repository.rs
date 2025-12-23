use anyhow::{Context, Result};
use rusqlite::{params, OptionalExtension};
use crate::database::connection::DbConn;
use crate::database::models::{PlayerFilter, PlayerWithRating, SortColumn, SortOrder};

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

    pub fn count_player_distinct_matches_played(
        conn: &mut DbConn,
        player_id: i32,
    ) -> Result<i32> {
        conn.query_row(
            "SELECT COUNT(DISTINCT date) FROM games WHERE first_player_id = ?1 OR second_player_id = ?1",
            params![player_id],
            |r| r.get(0)
        ).context("Failed to count player distinct matches played")
    }
}
