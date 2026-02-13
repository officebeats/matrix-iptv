use matrix_iptv_lib::api::XtreamClient;
use matrix_iptv_lib::config::{AppConfig, DnsProvider};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Matrix IPTV Cross-Playlist Audit ===\n");

    let config = AppConfig::load()?;
    if config.accounts.is_empty() {
        println!("❌ No accounts found in config.json");
        return Ok(());
    }

    println!("Found {} accounts. Starting audit...\n", config.accounts.len());

    for (i, acc) in config.accounts.iter().enumerate() {
        println!("[{}/{}] Account: {} ({})", i + 1, config.accounts.len(), acc.name, acc.base_url);
        
        // 1. Connection & Auth
        let start = Instant::now();
        match XtreamClient::new_with_doh(
            acc.base_url.clone(),
            acc.username.clone(),
            acc.password.clone(),
            DnsProvider::System,
        ).await {
            Ok(client) => {
                match client.authenticate().await {
                    Ok((true, _, _)) => {
                        let auth_dur = start.elapsed();
                        println!("  ✅ Auth: Success in {:.2}s", auth_dur.as_secs_f32());

                        // 2. Categories
                        let cat_start = Instant::now();
                        match client.get_live_categories().await {
                            Ok(cats) => {
                                let cat_dur = cat_start.elapsed();
                                println!("  📂 Categories: {} items in {:.2}s", cats.len(), cat_dur.as_secs_f32());

                                // 3. Full Stream Fetch (Testing Resilience & Speed)
                                println!("  🔍 Fetching ALL live streams (Testing Resilience)...");
                                let stream_start = Instant::now();
                                match client.get_live_streams("ALL").await {
                                    Ok(streams) => {
                                        let stream_dur = stream_start.elapsed();
                                        println!("  ✅ Streams: {} items in {:.2}s", streams.len(), stream_dur.as_secs_f32());

                                        // 4. MSNBC Search
                                        let msnbc: Vec<_> = streams.iter()
                                            .filter(|s| s.name.to_uppercase().contains("MSNBC"))
                                            .collect();
                                        
                                        if !msnbc.is_empty() {
                                            println!("  📍 Found {} MSNBC streams:", msnbc.len());
                                            for s in msnbc.iter().take(3) {
                                                println!("    - [{}] {}", s.stream_id, s.name);
                                            }
                                        } else {
                                            println!("  ⚠️ MSNBC not found in this playlist.");
                                        }
                                    }
                                    Err(e) => println!("  ❌ Stream Fetch Error: {}", e),
                                }
                            }
                            Err(e) => println!("  ❌ Categories Error: {}", e),
                        }
                    }
                    Ok((false, _, _)) => println!("  ❌ Auth Failed: Invalid credentials"),
                    Err(e) => println!("  ❌ Auth Connection Error: {}", e),
                }
            }
            Err(e) => println!("  ❌ Client Creation Error: {}", e),
        }
        println!();
    }

    println!("=== Audit Complete ===");
    Ok(())
}
