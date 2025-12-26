use matrix_iptv_lib::api::XtreamClient;
use matrix_iptv_lib::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Strong8K Playlist ===\n");
    
    let config = AppConfig::load()?;
    
    // Find Strong8K account
    let strong_account = config.accounts.iter()
        .find(|a| a.name.to_lowercase().contains("strong"))
        .expect("Strong8K account not found");
    
    println!("Account: {}", strong_account.name);
    println!("URL: {}", strong_account.base_url);
    println!();
    
    // Create client
    let client = XtreamClient::new_with_doh(
        strong_account.base_url.clone(),
        strong_account.username.clone(),
        strong_account.password.clone(),
        config.dns_provider,
    ).await?;
    
    // Test authentication
    print!("Testing authentication... ");
    match client.authenticate().await {
        Ok((true, _, _)) => println!("✓ PASS"),
        Ok((false, _, _)) => {
            println!("✗ FAIL - Invalid credentials");
            return Ok(());
        }
        Err(e) => {
            println!("✗ FAIL - {}", e);
            return Ok(());
        }
    }
    
    // Test Live TV
    print!("Testing Live TV categories... ");
    match client.get_live_categories().await {
        Ok(cats) => println!("✓ PASS - {} categories", cats.len()),
        Err(e) => println!("✗ FAIL - {}", e),
    }
    
    // Test VOD/Movies
    print!("Testing VOD/Movies categories... ");
    match client.get_vod_categories().await {
        Ok(cats) => println!("✓ PASS - {} categories", cats.len()),
        Err(e) => println!("✗ FAIL - {}", e),
    }
    
    // Test Series
    print!("Testing Series categories... ");
    match client.get_series_categories().await {
        Ok(cats) => {
            println!("✓ PASS - {} categories", cats.len());
            
            // Test loading streams from first category
            if let Some(first_cat) = cats.first() {
                print!("  Testing streams in '{}' category... ", first_cat.category_name);
                match client.get_series_streams(&first_cat.category_id).await {
                    Ok(streams) => println!("✓ PASS - {} series", streams.len()),
                    Err(e) => println!("✗ FAIL - {}", e),
                }
            }
        }
        Err(e) => println!("✗ FAIL - {}", e),
    }
    
    println!("\n=== Strong8K Test Complete ===");
    Ok(())
}
