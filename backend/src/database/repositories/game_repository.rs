use anyhow::{Context, Result};
use rusqlite::params;
use crate::database::connection::DbConn;
use crate::database::models::{HeadToHeadMatchRow, MatchResultRow};

pub struct GameRepository;

impl GameRepository {
    pub fn count_matches_played_for_player(
        conn: &mut DbConn,
        player_id: i32,
    ) -> Result<i32> {
        let sql = "SELECT COUNT(DISTINCT date) FROM games WHERE first_player_id = ?1 OR second_player_id = ?1";
        conn.query_row(sql, params![player_id], |row| row.get(0))
            .context("Failed to count matches played for player")
    }

    pub fn get_player_last_matches(
        conn: &mut DbConn,
        player_id: i32,
        limit: usize,
    ) -> Result<Vec<MatchResultRow>> {
        let sql = "
            SELECT
                g.date,
                t.name as tournament_name,
                CASE
                    WHEN g.first_player_id = ?1 THEN p2.name
                    ELSE p1.name
                END as opponent_name,
                CASE
                    WHEN g.first_player_id = ?1 THEN p2.id
                    ELSE p1.id
                END as opponent_id,
                SUM(CASE WHEN g.first_player_id = ?1 THEN g.first_player_score ELSE g.second_player_score END) as player_total_score,
                SUM(CASE WHEN g.first_player_id = ?1 THEN g.second_player_score ELSE g.first_player_score END) as opponent_total_score
            FROM games g
            JOIN tournaments t ON g.tournament_id = t.id
            JOIN players p1 ON g.first_player_id = p1.id
            JOIN players p2 ON g.second_player_id = p2.id
            WHERE g.first_player_id = ?1 OR g.second_player_id = ?1
            GROUP BY g.date, t.name, opponent_name, opponent_id
            ORDER BY g.date DESC
            LIMIT ?2
        ";
    
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![player_id, limit as i64], |row| {
            Ok(MatchResultRow {
                date: row.get(0)?,
                tournament_name: row.get(1)?,
                opponent_name: row.get(2)?,
                opponent_id: row.get(3)?,
                player_total_score: row.get(4)?,
                opponent_total_score: row.get(5)?,
            })
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
    
        Ok(rows)
    }

    pub fn get_head_to_head_matches(
        conn: &mut DbConn,
        player1_id: i32,
        player2_id: i32,
    ) -> Result<Vec<HeadToHeadMatchRow>> {
        let sql = "
            SELECT
                g.date,
                t.name as tournament_name,
                SUM(CASE WHEN (g.first_player_id = ?1 AND g.first_player_score > g.second_player_score) OR (g.second_player_id = ?1 AND g.second_player_score > g.first_player_score) THEN 1 ELSE 0 END) as p1_wins,
                SUM(CASE WHEN (g.first_player_id = ?2 AND g.first_player_score > g.second_player_score) OR (g.second_player_id = ?2 AND g.second_player_score > g.first_player_score) THEN 1 ELSE 0 END) as p2_wins
            FROM games g
            JOIN tournaments t ON g.tournament_id = t.id
            WHERE (g.first_player_id = ?1 AND g.second_player_id = ?2)
               OR (g.first_player_id = ?2 AND g.second_player_id = ?1)
            GROUP BY g.tournament_id, t.name, g.date
            ORDER BY g.date DESC
        ";
    
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params![player1_id, player2_id], |row| {
            Ok(HeadToHeadMatchRow {
                date: row.get(0)?,
                tournament_name: row.get(1)?,
                p1_wins: row.get(2)?,
                p2_wins: row.get(3)?,
            })
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
    
        Ok(rows)
    }
}
