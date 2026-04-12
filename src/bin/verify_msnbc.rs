use matrix_iptv_lib::api::XtreamClient;
use matrix_iptv_lib::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = AppConfig::load()?;
    let acc = config
        .accounts
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("No active account found"))?;

    println!("📡 Fetching Live Channels for account: {}", acc.name);
    println!("🔗 URL: {}", acc.base_url);

    let base_url_8080 = format!("{}:8080", acc.base_url);
    println!("🔗 Testing Stream on URL: {}", base_url_8080);

    let client = XtreamClient::new(
        base_url_8080.clone(),
        acc.username.clone(),
        acc.password.clone(),
    );

    let streams = client.get_live_streams("0", None).await?;
    println!("✅ Fetched {} streams total", streams.len());

    let msnbc_streams: Vec<_> = streams
        .iter()
        .filter(|s| s.name.to_lowercase().contains("msnbc"))
        .collect();

    if msnbc_streams.is_empty() {
        println!("❌ No MSNBC channels found in the playlist!");
        return Ok(());
    }

    println!("📺 Found {} MSNBC channels:", msnbc_streams.len());
    for s in &msnbc_streams {
        println!("  - [{}] {}", s.stream_id, s.name);
    }

    // Try to verify the first one
    let target = msnbc_streams[0];
    let stream_url = format!(
        "{}/live/{}/{}/{}.ts",
        base_url_8080, acc.username, acc.password, target.stream_id
    );

    let client_http = reqwest::Client::builder()
        .user_agent("IPTVSmartersPlayer")
        .build()?;

    println!("🔗 Verifying stream URL: {}", stream_url);

    let mut resp = client_http.get(&stream_url).send().await;

    // Resilience: Fallback to DoH if DNS fails for the stream as well
    if let Err(ref e) = resp {
        let err_str = e.to_string().to_lowercase();
        println!("DEBUG: Stream request error: {}", err_str);

        if err_str.contains("dns")
            || err_str.contains("resolution")
            || err_str.contains("resolve")
            || err_str.contains("no such host")
            || err_str.contains("11004")
            || err_str.contains("no data")
        {
            println!("⚠️ DNS Error on stream! Trying DoH fallback...");
            // Manual DoH resolve for the stream host
            // Using Quad9 IP-based DoH to avoid resolving the DoH host itself
            let doh_url = "https://9.9.9.9/dns-query?name=zfruvync.duperab.xyz";
            println!("🔗 Resolving via DoH: {}", doh_url);
            let doh_resp = client_http
                .get(doh_url)
                .header("Accept", "application/dns-json")
                .send()
                .await?;
            let doh_json: serde_json::Value = doh_resp.json().await?;

            if let Some(answers) = doh_json.get("Answer").and_then(|a| a.as_array()) {
                if let Some(ip) = answers[0].get("data").and_then(|d| d.as_str()) {
                    println!("✅ DoH Resolved stream host to: {}", ip);
                    let resolved_url = stream_url.replace("zfruvync.duperab.xyz", ip);
                    println!("🔗 Using resolved stream URL: {}", resolved_url);
                    resp = client_http
                        .get(&resolved_url)
                        .header("Host", "zfruvync.duperab.xyz")
                        .send()
                        .await;
                }
            }
        }
    }

    let resp = resp?;

    if resp.status().is_success() {
        println!("✅ Stream URL is reachable (Status: {})", resp.status());
        let bytes = resp.bytes().await?;
        println!("📦 Stream content size: {} bytes", bytes.len());
        if bytes.len() > 0 {
            println!("🚀 Playback verified: Stream is providing data!");
        } else {
            println!("⚠️ Stream is reachable but returned 0 bytes.");
        }
    } else {
        println!("❌ Stream URL failed (Status: {})", resp.status());
    }

    Ok(())
}
