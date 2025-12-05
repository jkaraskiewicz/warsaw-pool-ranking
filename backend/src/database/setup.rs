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
