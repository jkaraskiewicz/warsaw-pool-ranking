use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// File-based cache for tournament data with two-tier system
pub struct Cache {
    cache_dir: PathBuf,
    raw_dir: PathBuf,
    parsed_dir: PathBuf,
}

impl Cache {
    /// Create a new cache instance
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        let raw_dir = cache_dir.join("raw");
        let parsed_dir = cache_dir.join("parsed");

        // Create cache directories
        fs::create_dir_all(&raw_dir).context("Failed to create raw cache directory")?;
        fs::create_dir_all(&parsed_dir).context("Failed to create parsed cache directory")?;

        Ok(Self {
            cache_dir,
            raw_dir,
            parsed_dir,
        })
    }

    /// Save data to cache
    pub fn save<T: Serialize>(&self, key: &str, data: &T) -> Result<()> {
        let file_path = self.cache_dir.join(format!("{}.json", key));

        let json = serde_json::to_string_pretty(data).context("Failed to serialize data")?;

        fs::write(&file_path, json).context("Failed to write cache file")?;

        info!("Saved data to cache: {}", file_path.display());
        Ok(())
    }

    /// Load data from cache
    pub fn load<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let file_path = self.cache_dir.join(format!("{}.json", key));

        if !file_path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&file_path).context("Failed to read cache file")?;

        let data = serde_json::from_str(&json).context("Failed to deserialize cache data")?;

        info!("Loaded data from cache: {}", file_path.display());
        Ok(Some(data))
    }

    /// Check if cached data exists
    pub fn exists(&self, key: &str) -> bool {
        self.cache_dir.join(format!("{}.json", key)).exists()
    }

    /// Clear all cached data
    pub fn clear(&self) -> Result<()> {
        fs::remove_dir_all(&self.cache_dir).context("Failed to clear cache")?;

        fs::create_dir_all(&self.cache_dir).context("Failed to recreate cache directory")?;

        info!("Cleared cache directory");
        Ok(())
    }

    // --- Two-Tier Cache Methods ---

    /// Save raw API response to cache
    pub fn save_raw(&self, id: &str, data: &Value) -> Result<()> {
        let file_path = self.build_raw_path(id);
        self.write_json(&file_path, data)?;
        info!("Saved raw data to cache: {}", file_path.display());
        Ok(())
    }

    /// Load raw API response from cache
    pub fn load_raw(&self, id: &str) -> Result<Option<Value>> {
        let file_path = self.build_raw_path(id);
        self.read_json_opt(&file_path)
    }

    /// Save parsed data to cache
    pub fn save_parsed<T: Serialize>(&self, key: &str, data: &T) -> Result<()> {
        let file_path = self.build_parsed_path(key);
        self.write_json(&file_path, data)?;
        info!("Saved parsed data to cache: {}", file_path.display());
        Ok(())
    }

    /// Load parsed data from cache
    pub fn load_parsed<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let file_path = self.build_parsed_path(key);
        self.read_json_opt(&file_path)
    }

    // --- Helper Methods ---

    fn build_raw_path(&self, id: &str) -> PathBuf {
        self.raw_dir.join(format!("{}.json", id))
    }

    fn build_parsed_path(&self, key: &str) -> PathBuf {
        self.parsed_dir.join(format!("{}.json", key))
    }

    fn write_json<T: Serialize>(&self, path: &Path, data: &T) -> Result<()> {
        let json = serde_json::to_string_pretty(data)?;
        fs::write(path, json).context("Failed to write cache file")?;
        Ok(())
    }

    fn read_json_opt<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<Option<T>> {
        if !path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(path)?;
        let data = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse JSON from {:?}. First 200 chars: {}",
                path,
                &json[..json.len().min(200)]))?;
        Ok(Some(data))
    }
}
