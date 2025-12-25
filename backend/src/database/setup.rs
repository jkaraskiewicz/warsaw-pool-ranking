use anyhow::{Context, Result};

use super::connection::DbConn;

pub fn reset_database(conn: &mut DbConn) -> Result<()> {
    let schema_sql = include_str!("schema.sql");
    let statements = split_sql_statements(schema_sql);

    for (idx, statement) in statements.iter().enumerate() {
        if !statement.trim().is_empty() {
            execute_sql(conn, statement)
                .with_context(|| format!("Failed to execute statement {}", idx + 1))?;
        }
    }

    log::info!("Database schema reset successfully");
    Ok(())
}

pub fn create_avatars_table_if_missing(conn: &mut DbConn) -> Result<()> {
    let create_sql = "
        CREATE TABLE IF NOT EXISTS avatars (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            player_id INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
            size TEXT NOT NULL CHECK(size IN ('small', 'medium', 'large')),
            image_data BLOB NOT NULL,
            format TEXT NOT NULL DEFAULT 'webp',
            source_url TEXT NOT NULL,
            source_url_hash TEXT NOT NULL,
            width INTEGER NOT NULL,
            height INTEGER NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(player_id, size)
        );
        CREATE INDEX IF NOT EXISTS idx_avatars_player_size ON avatars(player_id, size);
        CREATE INDEX IF NOT EXISTS idx_avatars_source_hash ON avatars(source_url_hash);
    ";

    // Split statements if there are multiple (CREATE TABLE and CREATE INDEX)
    let statements = split_sql_statements(create_sql);

    for (idx, statement) in statements.iter().enumerate() {
        if !statement.trim().is_empty() {
            execute_sql(conn, statement)
                .with_context(|| format!("Failed to execute avatar statement {}", idx + 1))?;
        }
    }

    log::info!("Avatars table ensured to exist.");
    Ok(())
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn execute_sql(conn: &mut DbConn, sql: &str) -> Result<()> {
    conn.execute(sql, [])
        .context("Failed to execute SQL statement")
        .map(|_| ())
}