use anyhow::{Context, Result};
use std::fs;

use super::connection::DbConn;

pub fn reset_database(conn: &mut DbConn) -> Result<()> {
    let schema_sql = read_schema_file()?;
    let statements = split_sql_statements(&schema_sql);

    for (idx, statement) in statements.iter().enumerate() {
        if !statement.trim().is_empty() {
            execute_sql(conn, statement)
                .with_context(|| format!("Failed to execute statement {}", idx + 1))?;
        }
    }

    log::info!("Database schema reset successfully");
    Ok(())
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn read_schema_file() -> Result<String> {
    let possible_paths = [
        "src/database/schema.sql",
        "backend-rust/src/database/schema.sql",
        "../src/database/schema.sql",
    ];

    for path in &possible_paths {
        if let Ok(content) = fs::read_to_string(path) {
            return Ok(content);
        }
    }

    anyhow::bail!("Could not find schema.sql in any expected location")
}

fn execute_sql(conn: &mut DbConn, sql: &str) -> Result<()> {
    conn.execute(sql, [])
        .context("Failed to execute SQL statement")
        .map(|_| ())
}
