use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use rusqlite::{params, OptionalExtension};
use crate::database::connection::DbConn;
use crate::database::models::{Player, PlayerFilter, PlayerWithRating, SortColumn, SortOrder};

/// Helper struct to build WHERE clause and collect parameters
struct WhereClauseBuilder<'a> {
    clauses: Vec<&'a str>,
    params: Vec<&'a dyn rusqlite::ToSql>,
}

impl<'a> WhereClauseBuilder<'a> {
    fn new() -> Self {
        Self {
            clauses: Vec::new(),
            params: Vec::new(),
        }
    }

    fn add_clause(&mut self, clause: &'a str, param: &'a dyn rusqlite::ToSql) {
        self.clauses.push(clause);
        self.params.push(param);
    }

    fn build(&self) -> String {
        if self.clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", self.clauses.join(" AND "))
        }
    }
}

/// Build WHERE clause and parameters from filter
fn build_where_clause<'a>(
    filter: &'a PlayerFilter,
    min_games_holder: &'a Option<i32>,
    last_played_cutoff_holder: &'a Option<chrono::NaiveDateTime>,
) -> (String, Vec<&'a dyn rusqlite::ToSql>) {
    let mut builder = WhereClauseBuilder::new();

    // Add DSL-generated filters
    if !filter.sql_filter.is_empty() {
        builder.clauses.push(filter.sql_filter.where_clause.as_str());
        for param in &filter.sql_filter.params {
            builder.params.push(param.as_ref());
        }
    }

    // Add min_games filter
    if let Some(min_games) = min_games_holder {
        builder.add_clause("r.games_played >= ?", min_games);
    }

    // Add last_played_cutoff filter
    if let Some(cutoff) = last_played_cutoff_holder {
        builder.add_clause("p.last_played >= ?", cutoff);
    }

    (builder.build(), builder.params)
}

pub struct PlayerRepository;

impl PlayerRepository {
    pub fn list_ranked_players(
        conn: &mut DbConn,
        filter: &PlayerFilter,
    ) -> Result<(Vec<PlayerWithRating>, usize)> {
        // Store holders for filter values (needed for lifetime)
        let min_games_holder = filter.min_games;
        let last_played_cutoff_holder = filter.last_played_cutoff;

        // Build WHERE clause and collect parameters
        let (where_sql, params) = build_where_clause(filter, &min_games_holder, &last_played_cutoff_holder);
    
        // Count total matching records
        let count_sql = format!(
            "SELECT COUNT(*) FROM players p JOIN ratings r ON p.id = r.player_id {}",
            where_sql
        );
        let total: usize = conn.query_row(&count_sql, rusqlite::params_from_iter(&params), |row| row.get(0))?;

        // Build query with sorting and pagination
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

        // Add pagination parameters
        let limit_value = filter.limit as i64;
        let offset_value = filter.offset as i64;
        let mut query_params = params;
        query_params.push(&limit_value);
        query_params.push(&offset_value);

        // Execute query
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

    pub fn find_by_cuescore_id(conn: &mut DbConn, cuescore_id: i64) -> Result<Option<Player>> {
        let sql = "SELECT id, cuescore_id, name, avatar_url, last_played, created_at FROM players WHERE cuescore_id = ?1";

        conn.query_row(sql, params![cuescore_id], |row| {
            Ok(Player {
                id: row.get(0)?,
                cuescore_id: row.get(1)?,
                name: row.get(2)?,
                avatar_url: row.get(3)?,
                last_played: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).optional().context("Failed to query player by cuescore_id")
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

    /// Helper function to map row to Player
    fn map_row_to_player(row: &rusqlite::Row) -> rusqlite::Result<Player> {
        Ok(Player {
            id: row.get(0)?,
            cuescore_id: row.get(1)?,
            name: row.get(2)?,
            avatar_url: row.get(3)?,
            last_played: row.get(4)?,
            created_at: row.get(5)?,
        })
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
            Self::map_row_to_player
        ).optional().context("Failed to query player by cuescore_id")?;

        // Handle existing player - early return pattern
        if let Some(existing_player) = existing {
            // Only update if avatar_url is newly available
            if existing_player.avatar_url.is_none() && avatar_url.is_some() {
                let sql = "UPDATE players SET avatar_url = ?1 WHERE id = ?2 RETURNING id, cuescore_id, name, avatar_url, last_played, created_at";
                return conn.query_row(sql, params![avatar_url, existing_player.id], Self::map_row_to_player)
                    .context("Failed to update player avatar");
            }
            return Ok(existing_player);
        }

        // Insert new player
        let sql = "INSERT INTO players (cuescore_id, name, avatar_url, last_played) VALUES (?1, ?2, ?3, NULL) RETURNING id, cuescore_id, name, avatar_url, last_played, created_at";
        conn.query_row(sql, params![cuescore_id, name, avatar_url], Self::map_row_to_player)
            .context("Failed to insert new player")
    }
}