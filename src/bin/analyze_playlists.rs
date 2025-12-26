use matrix_iptv_lib::api::XtreamClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let accounts = vec![
        (
            "Example Provider 1",
            "http://example.com",
            "username",
            "password",
        ),
        (
            "Example Provider 2",
            "http://example.org",
            "username",
            "password",
        ),
    ];

    println!("=== IPTV Playlist Analysis ===\n");

    for (name, url, user, pass) in accounts {
        println!("Testing: {} ({})", name, url);

        match XtreamClient::new_with_doh(url.to_string(), user.to_string(), pass.to_string(), matrix_iptv_lib::config::DnsProvider::System).await
        {
            Ok(client) => {
                match client.authenticate().await {
                    Ok((true, _, si)) => {
                        println!(
                            "  ✅ Connected! (Timezone: {:?})",
                            si.and_then(|s| s.timezone)
                        );

                        // Get categories
                        match client.get_live_categories().await {
                            Ok(cats) => {
                                println!("  📂 {} Live Categories", cats.len());

                                // Sample first 20 categories for pattern analysis
                                println!("  Sample categories:");
                                for cat in cats.iter().take(20) {
                                    println!("    - {}", cat.category_name);
                                }
                                if cats.len() > 20 {
                                    println!("    ... and {} more", cats.len() - 20);
                                }
                            }
                            Err(e) => println!("  ❌ Failed to get categories: {}", e),
                        }
                    }
                    Ok((false, _, _)) => println!("  ❌ Auth failed"),
                    Err(e) => println!("  ❌ Auth error: {}", e),
                }
            }
            Err(e) => println!("  ❌ Connection error: {}", e),
        }
        println!();
    }

    Ok(())
}
