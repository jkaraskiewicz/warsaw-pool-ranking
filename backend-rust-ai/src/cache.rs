use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// File-based cache for tournament data
pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    /// Create a new cache instance
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Create cache directory if it doesn't exist
        fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;

        Ok(Self { cache_dir })
    }

    /// Save data to cache
    pub fn save<T: Serialize>(&self, key: &str, data: &T) -> Result<()> {
        let file_path = self.cache_dir.join(format!("{}.json", key));

        let json = serde_json::to_string_pretty(data)
            .context("Failed to serialize data")?;

        fs::write(&file_path, json)
            .context("Failed to write cache file")?;

        info!("Saved data to cache: {}", file_path.display());
        Ok(())
    }

    /// Load data from cache
    pub fn load<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let file_path = self.cache_dir.join(format!("{}.json", key));

        if !file_path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&file_path)
            .context("Failed to read cache file")?;

        let data = serde_json::from_str(&json)
            .context("Failed to deserialize cache data")?;

        info!("Loaded data from cache: {}", file_path.display());
        Ok(Some(data))
    }

    /// Check if cached data exists
    pub fn exists(&self, key: &str) -> bool {
        self.cache_dir.join(format!("{}.json", key)).exists()
    }

    /// Clear all cached data
    pub fn clear(&self) -> Result<()> {
        fs::remove_dir_all(&self.cache_dir)
            .context("Failed to clear cache")?;

        fs::create_dir_all(&self.cache_dir)
            .context("Failed to recreate cache directory")?;

        info!("Cleared cache directory");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        value: String,
    }

    #[test]
    fn test_cache_save_and_load() {
        let temp_dir = std::env::temp_dir().join("warsaw_pool_test_cache");
        let cache = Cache::new(&temp_dir).unwrap();

        let data = TestData {
            value: "test".to_string(),
        };

        cache.save("test_key", &data).unwrap();
        let loaded: Option<TestData> = cache.load("test_key").unwrap();

        assert_eq!(loaded, Some(data));

        // Cleanup
        cache.clear().unwrap();
    }
}
