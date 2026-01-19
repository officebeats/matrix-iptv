use matrix_iptv_lib::api::XtreamClient;
use matrix_iptv_lib::config::DnsProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing User Playlist with 8080 Approach ===\n");
    
    let base_url = "http://zfruvync.rmtil.com:8080".to_string();
    let username = "PE1S9S8U".to_string();
    let password = "11EZZUMW".to_string();
    
    println!("URL: {}", base_url);
    println!("User: {}", username);
    println!();
    
    // Create client with System DNS first (to see if it hits the AT&T block)
    println!("--- Testing with System DNS ---");
    let client = XtreamClient::new(base_url.clone(), username.clone(), password.clone());
    
    match client.authenticate().await {
        Ok((success, ui, _)) => {
            if success {
                println!("✓ PASS - Authenticated successfully!");
                if let Some(info) = ui {
                    println!("Status: {:?}", info.status);
                    println!("Expiry: {:?}", info.exp_date);
                }
            } else {
                println!("✗ FAIL - Authentication failed (Invalid credentials?)");
            }
        }
        Err(e) => {
            println!("✗ FAIL - Error: {}", e);
        }
    }

    println!("\n--- Testing with Quad9 DoH (Bypasses local DNS blocks) ---");
    match XtreamClient::new_with_doh(base_url, username, password, DnsProvider::Quad9).await {
        Ok(client) => {
             match client.authenticate().await {
                Ok((success, ui, _)) => {
                    if success {
                        println!("✓ PASS - Authenticated successfully via Quad9!");
                        if let Some(info) = ui {
                            println!("Status: {:?}", info.status);
                        }
                    } else {
                        println!("✗ FAIL - Authentication failed via Quad9");
                    }
                }
                Err(e) => {
                    println!("✗ FAIL - Error via Quad9: {}", e);
                }
            }
        }
        Err(e) => println!("✗ FAIL - Could not initialize DoH client: {}", e),
    }
    
    println!("\n=== Test Complete ===");
    Ok(())
}
