use std::path::PathBuf;
use std::env;

/// Get the cache directory path
///
/// Resolution order:
/// 1. CACHE_DIR environment variable (explicit override)
/// 2. Auto-detect: "backend/cache" if running from project root
/// 3. Default: "cache" (works in Docker where CWD=/app)
pub fn get_cache_dir() -> PathBuf {
    // Explicit override
    if let Ok(cache_dir) = env::var("CACHE_DIR") {
        return PathBuf::from(cache_dir);
    }

    // Auto-detect: if backend/ exists, we're at project root
    let backend_cache = PathBuf::from("backend/cache");
    if PathBuf::from("backend").is_dir() {
        return backend_cache;
    }

    // Default: works in Docker (CWD=/app, mounted at /app/cache)
    PathBuf::from("cache")
}

/// Get the database file path
///
/// Resolution order:
/// 1. DATABASE_PATH environment variable (explicit override)
/// 2. Auto-detect: "backend/data/warsaw_pool_ranking.db" if at project root
/// 3. Default: "warsaw_pool_ranking.db" (works in Docker with env var override)
pub fn get_database_path() -> String {
    // Explicit override
    if let Ok(db_path) = env::var("DATABASE_PATH") {
        return db_path;
    }

    // Auto-detect: if backend/ exists, we're at project root
    if PathBuf::from("backend").is_dir() {
        "backend/data/warsaw_pool_ranking.db".to_string()
    } else {
        // Default: current directory
        "warsaw_pool_ranking.db".to_string()
    }
}
