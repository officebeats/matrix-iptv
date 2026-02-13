use reqwest::{Client, redirect};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "http://zfruvync.duperab.xyz/live/PE1S9S8U/11EZZUMW/53504.ts";
    println!("Diagnosing Stream URL: {}", url);

    // Test Cases
    let scenarios = vec![
        ("No Headers", None, None),
        ("Chrome UA", Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"), None),
        ("VLC UA", Some("VLC/3.0.20 LibVLC/3.0.20"), None),
        ("Chrome UA + Referer", Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"), Some("http://zfruvync.duperab.xyz/")),
    ];

    for (name, ua, referer) in scenarios {
        println!("\n--- Testing Scenario: {} ---", name);
        
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(redirect::Policy::none()) // We want to see the 302, not follow it blindly yet
            .danger_accept_invalid_certs(true)
            .build()?;

        let mut req = client.get(url);
        if let Some(agent) = ua {
            req = req.header("User-Agent", agent);
        }
        if let Some(ref_url) = referer {
            req = req.header("Referer", ref_url);
        }

        match req.send().await {
            Ok(resp) => {
                println!("Status: {}", resp.status());
                println!("Headers:");
                for (k, v) in resp.headers() {
                    println!("  {}: {:?}", k, v);
                }
                
                if resp.status().is_redirection() {
                    if let Some(loc) = resp.headers().get("location") {
                        if let Ok(loc_str) = loc.to_str() {
                            println!("-> Redirects to: {}", loc_str);
                            // Verify if redirect target exists
                            verify_host(loc_str).await;
                        }
                    }
                } else if resp.status().is_success() {
                    println!("SUCCESS: Stream reachable directly!");
                }
            },
            Err(e) => println!("Request Failed: {}", e),
        }
    }
    
    Ok(())
}

async fn verify_host(url: &str) {
    let host = if let Some(start) = url.find("://") {
        let rest = &url[start + 3..];
        if let Some(end) = rest.find('/') {
            &rest[..end]
        } else {
            // Case where url is just http://host
            if let Some(port) = rest.find(':') {
                &rest[..port]
            } else {
                rest
            }
        }
    } else {
        return;
    };

    println!("  Checking DNS for host: {}", host);
    // Simple DNS lookup check via Google DoH to avoid local system DNS bias
    let doh_url = format!("https://dns.google/resolve?name={}", host);
    let client = Client::new();
    match client.get(&doh_url).send().await {
        Ok(resp) => {
                if let Ok(text) = resp.text().await {
                    if text.contains("\"Status\": 0") {
                        println!("  [✓] Host ({}) exists (Google DoH).", host);
                    } else {
                        println!("  [X] Host ({}) returned error/NXDOMAIN from Google DoH: {}", host, text);
                    }
                }
        },
        Err(_) => println!("  [?] Could not check DoH for host."),
    }
}
