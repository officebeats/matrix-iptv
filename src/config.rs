#[cfg(not(target_arch = "wasm32"))]
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]


/// DNS-over-HTTPS provider options
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub enum DnsProvider {
    Quad9,
    AdGuard,
    Cloudflare,
    Google,
    #[default]
    System,
}

impl DnsProvider {
    pub fn display_name(&self) -> &'static str {
        match self {
            DnsProvider::Quad9 => "Quad9 (Recommended)",
            DnsProvider::AdGuard => "AdGuard",
            DnsProvider::Cloudflare => "Cloudflare",
            DnsProvider::Google => "Google",
            DnsProvider::System => "System DNS",
        }
    }

    pub fn all() -> &'static [DnsProvider] {
        &[
            DnsProvider::Quad9,
            DnsProvider::AdGuard,
            DnsProvider::Cloudflare,
            DnsProvider::Google,
            DnsProvider::System,
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub name: String,
    pub base_url: String,
    pub username: String,
    pub password: String,
    pub epg_url: Option<String>,
    pub last_refreshed: Option<i64>,
    pub total_channels: Option<usize>,
    pub total_movies: Option<usize>,
    pub total_series: Option<usize>,
    pub server_timezone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Favorites {
    pub categories: std::collections::HashSet<String>, // Category IDs
    pub streams: std::collections::HashSet<String>,    // Stream IDs
    pub vod_categories: std::collections::HashSet<String>,
    pub vod_streams: std::collections::HashSet<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub accounts: Vec<Account>,
    pub last_used_account_index: Option<usize>,
    #[serde(default)]
    pub favorites: Favorites,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub american_mode: bool,
    #[serde(default)]
    pub dns_provider: DnsProvider,
}

impl AppConfig {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Result<Self, anyhow::Error> {
        let new_proj = ProjectDirs::from("com", "vibecoding", "vibe-iptv");
        let old_proj = ProjectDirs::from("com", "vibecoding", "iptv-cli");

        if let Some(proj_dirs) = new_proj {
            let config_path = proj_dirs.config_dir().join("config.json");
            if config_path.exists() {
                let content = fs::read_to_string(config_path)?;
                let config: AppConfig = serde_json::from_str(&content)?;
                return Ok(config);
            }

            // If new doesn't exist, check old
            if let Some(old_dirs) = old_proj {
                let old_path = old_dirs.config_dir().join("config.json");
                if old_path.exists() {
                    // MIGRATION: Copy old to new
                    let content = fs::read_to_string(&old_path)?;
                    let config: AppConfig = serde_json::from_str(&content)?;

                    // Save to new location
                    let _ = fs::create_dir_all(proj_dirs.config_dir());
                    let _ = fs::write(config_path, &content);

                    return Ok(config);
                }
            }
        }
        Ok(AppConfig::default())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Result<Self, anyhow::Error> {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(json)) = storage.get_item("app_config") {
                    return serde_json::from_str(&json).map_err(|e| anyhow::anyhow!(e));
                }
            }
        }
        Ok(AppConfig::default())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) -> Result<(), anyhow::Error> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "vibecoding", "vibe-iptv") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir)?;
            let config_path = config_dir.join("config.json");
            let content = serde_json::to_string_pretty(self)?;
            fs::write(config_path, content)?;
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) -> Result<(), anyhow::Error> {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let json = serde_json::to_string(self)?;
                let _ = storage.set_item("app_config", &json);
            }
        }
        Ok(())
    }

    pub fn add_account(&mut self, account: Account) {
        self.accounts.push(account);
        let _ = self.save();
    }

    pub fn update_account(&mut self, index: usize, account: Account) {
        if index < self.accounts.len() {
            self.accounts[index] = account;
            let _ = self.save();
        }
    }

    pub fn remove_account(&mut self, index: usize) {
        if index < self.accounts.len() {
            self.accounts.remove(index);
            let _ = self.save();
        }
    }

    pub fn toggle_favorite_category(&mut self, id: String) {
        if self.favorites.categories.contains(&id) {
            self.favorites.categories.remove(&id);
        } else {
            self.favorites.categories.insert(id);
        }
        let _ = self.save();
    }

    pub fn toggle_favorite_stream(&mut self, id: String) {
        if self.favorites.streams.contains(&id) {
            self.favorites.streams.remove(&id);
        } else {
            self.favorites.streams.insert(id);
        }
        let _ = self.save();
    }

    pub fn toggle_favorite_vod_category(&mut self, id: String) {
        if self.favorites.vod_categories.contains(&id) {
            self.favorites.vod_categories.remove(&id);
        } else {
            self.favorites.vod_categories.insert(id);
        }
        let _ = self.save();
    }

    pub fn toggle_favorite_vod_stream(&mut self, id: String) {
        if self.favorites.vod_streams.contains(&id) {
            self.favorites.vod_streams.remove(&id);
        } else {
            self.favorites.vod_streams.insert(id);
        }
        let _ = self.save();
    }

    pub fn get_user_timezone(&self) -> String {
        if let Some(tz) = &self.timezone {
            return tz.clone();
        }

        // Try to detect system timezone
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(tz) = iana_time_zone::get_timezone() {
                return tz;
            }
        }

        // Fallback
        "UTC".to_string()
    }

    pub fn set_timezone(&mut self, tz: String) {
        self.timezone = Some(tz);
        let _ = self.save();
    }

    pub fn set_dns_provider(&mut self, provider: DnsProvider) {
        self.dns_provider = provider;
        let _ = self.save();
    }
}
