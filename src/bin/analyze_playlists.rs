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
            "Strong 8k3",
            "http://darkteam.vip",
            "13557354309740",
            "32203023328226",
        ),
        (
            "Strong 8k 4",
            "http://bureau88228.cdn-only.me:80",
            "0f6aa36ff19a",
            "8af89f7179",
        ),
        (
            "Premium Mega 4k FHD",
            "http://smarters.live:80",
            "ZPY7BP5A",
            "ZX2JVDNQ",
        ),
        (
            "Promax 4k OTT",
            "http://line.queen-4k.cc",
            "11D246",
            "41D1C5",
        ),
        ("Mega OTT 1", "http://line.4smart.in", "45Z88W6", "Z7PHTX3"),
        (
            "Mega OTT 2",
            "http://pwwkvdbn.qastertv.xyz",
            "CZ2FWXLS",
            "AQ9CFKL6",
        ),
        (
            "Strong 8k 5",
            "http://line.trx-ott.com",
            "10af90a352",
            "b616f33a73aa",
        ),
    ];

    println!("=== IPTV Playlist Analysis ===\n");

    for (name, url, user, pass) in accounts {
        println!("Testing: {} ({})", name, url);

        match XtreamClient::new_with_doh(url.to_string(), user.to_string(), pass.to_string()).await
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
