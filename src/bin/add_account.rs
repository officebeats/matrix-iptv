use matrix_iptv_lib::config::{AppConfig, Account, AccountType, DnsProvider};
use directories::ProjectDirs;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proj_dirs = ProjectDirs::from("com", "vibecoding", "vibe-iptv").ok_or("Could not find project dirs")?;
    let config_dir = proj_dirs.config_dir();
    let config_path = config_dir.join("config.json");
    
    fs::create_dir_all(config_dir)?;
    
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        serde_json::from_str(&content)?
    } else {
        AppConfig::default()
    };
    
    // Add new account
    let new_account = Account {
        name: "Strong8k2-PC".to_string(),
        base_url: "http://zfruvync.rmtil.com:8080".to_string(),
        username: "PE1S9S8U".to_string(),
        password: "11EZZUMW".to_string(),
        account_type: AccountType::Xtream,
        epg_url: None,
        last_refreshed: None,
        total_channels: None,
        total_movies: None,
        total_series: None,
        server_timezone: None,
    };
    
    // Remove if exists
    config.accounts.retain(|a| a.name != new_account.name);
    config.accounts.push(new_account);
    
    // Set DNS to system (for best compatibility with 8080 port bypass)
    config.dns_provider = DnsProvider::System;
    
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, json)?;
    
    println!("Successfully added account to {:?}", config_path);
    Ok(())
}
