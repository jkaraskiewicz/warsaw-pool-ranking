use anyhow::{Context, Result};
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = r2d2::Pool<SqliteConnectionManager>;
pub type DbConn = r2d2::PooledConnection<SqliteConnectionManager>;

pub fn create_pool(database_path: &str) -> Result<DbPool> {
    let manager = build_manager(database_path);
    build_pool(manager)
}

fn build_manager(path: &str) -> SqliteConnectionManager {
    SqliteConnectionManager::file(path)
}

fn build_pool(manager: SqliteConnectionManager) -> Result<DbPool> {
    r2d2::Pool::builder()
        .build(manager)
        .context("Failed to create database connection pool")
}

pub fn get_connection(pool: &DbPool) -> Result<DbConn> {
    pool.get()
        .context("Failed to get database connection from pool")
}
