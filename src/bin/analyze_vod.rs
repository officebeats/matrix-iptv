use matrix_iptv_lib::api::XtreamClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let accounts = vec![
        (
            "Trex",
            "http://line.offcial-trex.pro",
            "3a6aae52fb",
            "39c165888139",
        ),
        ("TREX 2", "http://line.4smart.in", "02C298D", "QPA233U"),
        (
            "Strong 8k 2",
            "http://susan47491.cdn-akm.me",
            "5d267aa26217",
            "934f8e20cc",
        ),
        (
            "Promax 4k OTT",
            "http://line.queen-4k.cc",
            "11D246",
            "41D1C5",
        ),
    ];

    println!("=== VOD Content Analysis ===\n");

    for (name, url, user, pass) in accounts {
        println!("Testing: {} ({})", name, url);

        match XtreamClient::new_with_doh(url.to_string(), user.to_string(), pass.to_string()).await
        {
            Ok(client) => {
                match client.authenticate().await {
                    Ok((true, _, _)) => {
                        println!("  ✅ Connected!");

                        // Get VOD categories
                        match client.get_vod_categories().await {
                            Ok(cats) => {
                                println!("  🎬 {} VOD Categories", cats.len());

                                // Sample categories
                                println!("  Sample VOD categories:");
                                for cat in cats.iter().take(15) {
                                    println!("    - {}", cat.category_name);
                                }

                                // Get some movies from first category
                                if let Some(first_cat) = cats.first() {
                                    if let Ok(movies) =
                                        client.get_vod_streams(&first_cat.category_id).await
                                    {
                                        println!(
                                            "\n  Sample movies from '{}':",
                                            first_cat.category_name
                                        );
                                        for movie in movies.iter().take(10) {
                                            println!("    - {}", movie.name);
                                        }
                                    }
                                }
                            }
                            Err(e) => println!("  ❌ Failed to get VOD categories: {}", e),
                        }
                    }
                    Ok((false, _, _)) => println!("  ❌ Auth failed"),
                    Err(e) => println!("  ❌ Auth error: {}", e),
                }
            }
            Err(e) => println!("  ❌ Connection error: {}", e),
        }
        println!("\n{}", "=".repeat(60));
        println!();
    }

    Ok(())
}
