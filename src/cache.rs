//! Local catalog cache for instant cold starts.
//!
//! This module provides binary serialization of catalog data using bincode,
//! enabling sub-200ms cold starts from cached data while refreshing in background.

use crate::api::{Category, Stream};
use crate::config::ProcessingMode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cache version — increment when CachedCatalog struct changes to auto-invalidate old caches
const CACHE_VERSION: u32 = 1;

/// On-disk catalog cache for a single account
#[derive(Serialize, Deserialize)]
pub struct CachedCatalog {
    pub version: u32,
    pub cached_at: u64,            // Unix timestamp (seconds)
    pub account_name: String,
    pub account_url: String,       // To detect if provider changed

    // Pre-preprocessed data (already filtered by active modes at cache time)
    pub processing_modes: Vec<ProcessingMode>,

    // Live
    pub live_categories: Vec<Category>,
    pub live_streams: Vec<Stream>,

    // VOD
    pub vod_categories: Vec<Category>,
    pub vod_streams: Vec<Stream>,

    // Series
    pub series_categories: Vec<Category>,
    pub series_streams: Vec<Stream>,

    // Metadata
    pub total_channels: usize,
    pub total_movies: usize,
    pub total_series: usize,

    // Category channel counts (category_id -> count)
    pub category_counts: Vec<(String, usize)>,
}

impl CachedCatalog {
    /// Returns the cache file path for a given account.
    /// Path: <config_dir>/cache/<account_name_hash>.bin
    #[cfg(not(target_arch = "wasm32"))]
    pub fn cache_path(account_name: &str) -> Option<PathBuf> {
        use directories::ProjectDirs;
        let proj = ProjectDirs::from("com", "vibecoding", "vibe-iptv")?;
        let cache_dir = proj.cache_dir().to_path_buf();
        std::fs::create_dir_all(&cache_dir).ok()?;

        // Hash the account name to avoid filesystem issues with special characters
        let hash = simple_hash(account_name);
        Some(cache_dir.join(format!("{}.bin", hash)))
    }

    /// Save catalog to disk. Non-blocking — call from a background task.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) -> Result<(), anyhow::Error> {
        let path = Self::cache_path(&self.account_name)
            .ok_or_else(|| anyhow::anyhow!("Cannot determine cache directory"))?;
        let encoded = bincode::serialize(self)?;
        std::fs::write(&path, encoded)?;
        Ok(())
    }

    /// Load catalog from disk. Returns None if cache doesn't exist, is corrupt, or version mismatches.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(account_name: &str) -> Option<CachedCatalog> {
        let path = Self::cache_path(account_name)?;
        let data = std::fs::read(&path).ok()?;
        let catalog: CachedCatalog = bincode::deserialize(&data).ok()?;

        // Version check — reject outdated cache format
        if catalog.version != CACHE_VERSION {
            let _ = std::fs::remove_file(&path); // Clean up stale cache
            return None;
        }

        Some(catalog)
    }

    /// Check if cache is stale based on auto_refresh_hours setting.
    /// Returns true if cache should be refreshed.
    pub fn is_stale(&self, auto_refresh_hours: u32) -> bool {
        if auto_refresh_hours == 0 {
            return false; // Auto-refresh disabled
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let age_hours = (now.saturating_sub(self.cached_at)) / 3600;
        age_hours >= auto_refresh_hours as u64
    }

    /// Check if the active processing modes have changed since cache was built.
    pub fn modes_changed(&self, current_modes: &[ProcessingMode]) -> bool {
        self.processing_modes != current_modes
    }

    /// Delete cache for account
    #[cfg(not(target_arch = "wasm32"))]
    pub fn invalidate(account_name: &str) {
        if let Some(path) = Self::cache_path(account_name) {
            let _ = std::fs::remove_file(path);
        }
    }

    /// WASM stub - cache not supported in browser
    #[cfg(target_arch = "wasm32")]
    pub fn cache_path(_account_name: &str) -> Option<PathBuf> {
        None
    }

    /// WASM stub - cache not supported in browser
    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    /// WASM stub - cache not supported in browser
    #[cfg(target_arch = "wasm32")]
    pub fn load(_account_name: &str) -> Option<CachedCatalog> {
        None
    }

    /// WASM stub - cache not supported in browser
    #[cfg(target_arch = "wasm32")]
    pub fn invalidate(_account_name: &str) {}
}

fn simple_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
