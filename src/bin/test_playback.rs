use matrix_iptv_lib::config::AppConfig;
use matrix_iptv_lib::api::XtreamClient;
use matrix_iptv_lib::player::Player;
use matrix_iptv_lib::config::PlayerEngine;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("=== Matrix IPTV CLI Playback Diagnostic Tool ===\n");

    let player = Player::new();
    
    println!("[1] Checking MPV installation...");
    match matrix_iptv_lib::setup::get_mpv_path() {
        Some(path) => println!("    ✓ MPV found at: {}", path),
        None => {
            println!("    ✗ MPV not found!");
            println!("    Please install MPV:");
            println!("      - Windows: winget install mpv");
            println!("      - Mac: brew install mpv");
            println!("      - Linux: sudo apt install mpv");
        }
    }

    println!("\n[2] Checking for VLC fallback...");
    match matrix_iptv_lib::setup::get_vlc_path() {
        Some(path) => println!("    ✓ VLC found at: {}", path),
        None => println!("    ℹ VLC not found (will use MPV only)"),
    }

    println!("\n[3] Loading config...");
    let config = match AppConfig::load() {
        Ok(c) => c,
        Err(e) => {
            println!("    ✗ Failed to load config: {}", e);
            return;
        }
    };

    if config.accounts.is_empty() {
        println!("    ✗ No accounts configured. Run matrix-iptv first to add an account.");
        return;
    }

    println!("    Found {} account(s)", config.accounts.len());
    
    for (i, account) in config.accounts.iter().enumerate() {
        println!("      {}) {} - {}", i + 1, account.name, account.base_url);
    }

    let account = &config.accounts[config.last_used_account_index.unwrap_or(0)];
    println!("\n[4] Testing connection to: {}...", account.name);

    let client = XtreamClient::new(
        account.base_url.clone(),
        account.username.clone(), 
        account.password.clone()
    );

    match client.authenticate().await {
        Ok((auth_ok, user_info, _)) => {
            if auth_ok {
                println!("    ✓ Authentication successful!");
                if let Some(info) = user_info {
                    println!("    ✓ User status: {:?}", info.status);
                }
            } else {
                println!("    ✗ Authentication failed");
                return;
            }
        }
        Err(e) => {
            println!("    ✗ Authentication failed: {}", e);
            return;
        }
    }

    println!("\n[5] Fetching channel list...");
    match client.get_live_categories().await {
        Ok(categories) => {
            println!("    ✓ Found {} categories", categories.len());
            
            let target_cat = categories.iter().find(|c| c.category_name.contains("NBA") || c.category_name.contains("Sports"))
                .or_else(|| categories.iter().find(|c| c.category_name.contains("USA") || c.category_name.contains(" Entertainment")))
                .or_else(|| categories.first());
                
            if let Some(cat) = target_cat {
                println!("    Testing with category: {}", cat.category_name);
                
                match client.get_live_streams(&cat.category_id, None).await {
                    Ok(streams) => {
                        println!("    ✓ Found {} streams in this category", streams.len());
                        
                        let stream = streams.iter()
                            .find(|s| s.name.contains("NBA") || s.name.contains("CBS") || s.name.contains("ABC"))
                            .or_else(|| streams.first());
                            
                        if let Some(s) = stream {
                            println!("\n[6] Testing playback...");
                            println!("    Channel: {}", s.name);
                            
                            let stream_id = matrix_iptv_lib::api::get_id_str(&s.stream_id);
                            
                            println!("    URL: [hidden for security]");
                            
                            let exts = ["ts", "m3u8", "mp4"];
                            let mut success = false;
                            
                            for ext in exts {
                                let test_url = client.get_stream_url(&stream_id, ext);
                                println!("\n    Trying format: {}...", ext);
                                println!("    URL: {}", test_url);
                                
                                let engine = config.preferred_player;
                                let use_default = config.use_default_mpv;
                                let smooth = config.smooth_motion;
                                
                                match player.play(&test_url, engine, use_default, smooth).await {
                                    Ok(_) => {
                                        println!("    ✓ Player launched!");
                                        
                                        // Wait longer to see if it actually plays
                                        for i in 1..=6 {
                                            sleep(Duration::from_secs(2)).await;
                                            println!("    Checking after {} seconds...", i * 2);
                                            
                                            if !player.is_running() {
                                                if let Some(log_err) = player.get_last_error_from_log() {
                                                    println!("    ✗ Player exited. Log error: {}", log_err);
                                                } else {
                                                    println!("    ✗ Player exited (no error in log)");
                                                }
                                                break;
                                            }
                                            
                                            if i >= 3 {
                                                println!("    ✓ Player is running after {} seconds - SUCCESS!", i * 2);
                                                success = true;
                                                break;
                                            }
                                        }
                                        
                                        player.stop();
                                        if success { break; }
                                    }
                                    Err(e) => {
                                        println!("    ✗ Failed to start player: {}", e);
                                        let diagnosis = player.diagnose_playback_failure(&e.to_string());
                                        if let Some(hint) = diagnosis.hint {
                                            println!("      Hint: {}", hint);
                                        }
                                    }
                                }
                            }
                            
                            // If all failed, try VLC directly
                            if !success {
                                println!("\n    Trying direct VLC fallback...");
                                if let Some(vlc_path) = matrix_iptv_lib::setup::get_vlc_path() {
                                    println!("    VLC found at: {}", vlc_path);
                                    let vlc_url = client.get_stream_url(&stream_id, "ts");
                                    // Test with simple vlc command
                                    let result = std::process::Command::new(&vlc_path)
                                        .arg(&vlc_url)
                                        .spawn();
                                    
                                    match result {
                                        Ok(mut child) => {
                                            sleep(Duration::from_secs(5)).await;
                                            match child.try_wait() {
                                                Ok(Some(status)) => {
                                                    println!("    ✗ VLC exited with status: {}", status);
                                                }
                                                Ok(None) => {
                                                    println!("    ✓ VLC is running!");
                                                    success = true;
                                                    let _ = child.kill();
                                                }
                                                Err(e) => println!("    Error checking VLC: {}", e),
                                            }
                                        }
                                        Err(e) => println!("    ✗ Failed to start VLC: {}", e),
                                    }
                                }
                            }
                            
                            if success {
                                println!("\n=== PLAYBACK SUCCESS ===");
                            } else {
                                println!("\n=== PLAYBACK FAILED ===");
                                println!("\nDiagnostic suggestions:");
                                for suggestion in player.check_and_suggest_fixes() {
                                    println!("  - {}", suggestion);
                                }
                            }
                        }
                    }
                    Err(e) => println!("    ✗ Failed to get streams: {}", e),
                }
            }
        }
        Err(e) => println!("    ✗ Failed to get categories: {}", e),
    }
}