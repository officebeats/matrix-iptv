use directories::ProjectDirs;
use matrix_iptv_lib::config::{Account, AppConfig};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "vibecoding", "vibe-iptv") {
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;
        let config_path = config_dir.join("config.json");
        println!("Config path: {:?}", config_path);

        let mut config: AppConfig = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppConfig::default()
        };

        let trex_account = Account {
            name: "Trex Placeholder".to_string(),
            base_url: "http://your-provider-url.com".to_string(),
            username: "YOUR_USERNAME".to_string(),
            password: "YOUR_PASSWORD".to_string(),
            epg_url: None,
            last_refreshed: None,
            total_channels: None,
            total_movies: None,
            total_series: None,
            server_timezone: None,
        };

        let strong8k_account = Account {
            name: "Strong 8K Placeholder".to_string(),
            base_url: "http://your-provider-url.com".to_string(),
            username: "YOUR_USERNAME".to_string(),
            password: "YOUR_PASSWORD".to_string(),
            epg_url: None,
            last_refreshed: None,
            total_channels: None,
            total_movies: None,
            total_series: None,
            server_timezone: Some("Europe/Amsterdam".to_string()),
        };

        // Remove existing if name matches
        config
            .accounts
            .retain(|a| a.name != "Trex Placeholder" && a.name != "Strong 8K Placeholder");
        config.accounts.push(trex_account);
        config.accounts.push(strong8k_account);

        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, content)?;
        println!("Updated config with Trex and Strong 8K accounts.");
    } else {
        println!("Could not determine config path.");
    }
    Ok(())
}

