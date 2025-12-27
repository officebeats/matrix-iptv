#[cfg(not(target_arch = "wasm32"))]
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;


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

/// Playlist processing mode options (Legacy - specific combinations)
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub enum PlaylistMode {
    #[default]
    Default,
    Merica,
    Sports,
    AllEnglish,
    SportsMerica,
}

impl PlaylistMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            PlaylistMode::Default => "Default",
            PlaylistMode::Merica => "'merica",
            PlaylistMode::Sports => "Sports",
            PlaylistMode::AllEnglish => "All English (US/UK/CA)",
            PlaylistMode::SportsMerica => "Sports + 'merica",
        }
    }

    pub fn is_merica_variant(&self) -> bool {
        matches!(self, PlaylistMode::Merica | PlaylistMode::SportsMerica)
    }

    pub fn all() -> &'static [PlaylistMode] {
        &[
            PlaylistMode::Default,
            PlaylistMode::Merica,
            PlaylistMode::Sports,
            PlaylistMode::AllEnglish,
            PlaylistMode::SportsMerica,
        ]
    }
}

/// Composable processing modes for multi-select
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessingMode {
    Merica,
    Sports,
    AllEnglish,
}

impl ProcessingMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            ProcessingMode::Merica => "'merica (Geo-Filter & Cleanup)",
            ProcessingMode::Sports => "Sports (Icons & Sorting)",
            ProcessingMode::AllEnglish => "All English (US/UK/CA Only)",
        }
    }

    pub fn all() -> &'static [ProcessingMode] {
        &[
            ProcessingMode::Merica,
            ProcessingMode::Sports,
            ProcessingMode::AllEnglish,
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

/// A user-defined channel group
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChannelGroup {
    pub name: String,
    #[serde(default)]
    pub icon: Option<String>,  // Emoji or icon name
    #[serde(default)]
    pub stream_ids: Vec<String>,  // Ordered list of stream IDs
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Favorites {
    pub categories: std::collections::HashSet<String>, // Category IDs
    pub streams: std::collections::HashSet<String>,    // Stream IDs
    pub vod_categories: std::collections::HashSet<String>,
    pub vod_streams: std::collections::HashSet<String>,
    #[serde(default)]
    pub groups: Vec<ChannelGroup>,  // Custom user groups
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub accounts: Vec<Account>,
    pub last_used_account_index: Option<usize>,
    #[serde(default)]
    pub favorites: Favorites,
    #[serde(default)]
    pub timezone: Option<String>,
    
    // Legacy support
    #[serde(default)] // Don't skip serializing yet if we want external tools to see it, but prefer migration
    pub playlist_mode: PlaylistMode, 

    #[serde(default)]
    pub processing_modes: Vec<ProcessingMode>,

    #[serde(default)]
    pub dns_provider: DnsProvider,
    #[serde(default)]
    pub use_default_mpv: bool,  // Use default MPV settings instead of optimized
    
    /// Auto-refresh playlist if older than this many hours. 0 = disabled.
    #[serde(default = "default_auto_refresh_hours")]
    pub auto_refresh_hours: u32,
}

fn default_auto_refresh_hours() -> u32 { 12 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            last_used_account_index: None,
            favorites: Favorites::default(),
            timezone: None,
            playlist_mode: PlaylistMode::default(),
            processing_modes: Vec::new(),
            dns_provider: DnsProvider::default(),
            use_default_mpv: false,
            auto_refresh_hours: 12,
        }
    }
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
                let mut config: AppConfig = serde_json::from_str(&content)?;
                
                // MIGRATION: V3.0.4 - Convert legacy playlist_mode to processing_modes
                if config.processing_modes.is_empty() && config.playlist_mode != PlaylistMode::Default {
                    match config.playlist_mode {
                        PlaylistMode::Merica => config.processing_modes.push(ProcessingMode::Merica),
                        PlaylistMode::Sports => config.processing_modes.push(ProcessingMode::Sports),
                        PlaylistMode::AllEnglish => config.processing_modes.push(ProcessingMode::AllEnglish),
                        PlaylistMode::SportsMerica => {
                            config.processing_modes.push(ProcessingMode::Merica);
                            config.processing_modes.push(ProcessingMode::Sports);
                        }
                        _ => {}
                    }
                }
                
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

    // Group management methods
    pub fn create_group(&mut self, name: String, icon: Option<String>) -> usize {
        let group = ChannelGroup {
            name,
            icon,
            stream_ids: Vec::new(),
        };
        self.favorites.groups.push(group);
        let _ = self.save();
        self.favorites.groups.len() - 1
    }

    pub fn delete_group(&mut self, index: usize) {
        if index < self.favorites.groups.len() {
            self.favorites.groups.remove(index);
            let _ = self.save();
        }
    }

    pub fn add_to_group(&mut self, group_index: usize, stream_id: String) {
        if let Some(group) = self.favorites.groups.get_mut(group_index) {
            if !group.stream_ids.contains(&stream_id) {
                group.stream_ids.push(stream_id);
                let _ = self.save();
            }
        }
    }

    pub fn remove_from_group(&mut self, group_index: usize, stream_id: &str) {
        if let Some(group) = self.favorites.groups.get_mut(group_index) {
            group.stream_ids.retain(|id| id != stream_id);
            let _ = self.save();
        }
    }

    pub fn rename_group(&mut self, group_index: usize, new_name: String) {
        if let Some(group) = self.favorites.groups.get_mut(group_index) {
            group.name = new_name;
            let _ = self.save();
        }
    }
}
