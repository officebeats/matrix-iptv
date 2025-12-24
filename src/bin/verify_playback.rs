use directories::ProjectDirs;
use matrix_iptv_lib::api::XtreamClient;
use matrix_iptv_lib::config::AppConfig;
use std::fs;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load Config
    let proj_dirs = ProjectDirs::from("com", "vibecoding", "vibe-iptv").ok_or("No proj dirs")?;
    let config_path = proj_dirs.config_dir().join("config.json");

    // Use hardcoded credentials for testing
    let base_url = "http://your-provider.com".to_string();
    let username = "username".to_string();
    let password = "password".to_string();

    println!("Testing with: {} / {}", base_url, username);

    let client = XtreamClient::new_with_doh(base_url, username, password).await?;

    // 3. Login
    if client.authenticate().await?.0 {
        println!("Authenticated!");

        // 4. Get a Live Stream
        let cats = client.get_live_categories().await?;
        if let Some(cat) = cats.first() {
            println!("Category: {}", cat.category_name);
            let streams = client
                .get_live_streams(&cat.category_id.to_string())
                .await?;
            if let Some(stream) = streams.first() {
                println!("Stream: {}", stream.name);
                let url = client.get_stream_url(&stream.stream_id.to_string(), "ts");
                println!("Generated URL: {}", url);

                // 5. Test Reachability (HEAD)
                let client_http = reqwest::Client::new();
                let resp = client_http.head(&url).send().await?;
                println!("Stream URL Status: {}", resp.status());

                if resp.status().is_success() {
                    println!("Stream is accessible! Launching MPV for 5 seconds test...");
                    // 6. Launch MPV (dry run or short run)
                    // We use --end=5 to stop after 5 seconds
                    let status = Command::new("mpv").arg(&url).arg("--end=5").status()?;
                    println!("MPV exited with: {}", status);
                } else {
                    println!("Stream URL returned error.");
                }
            }
        }
    } else {
        println!("Authentication failed.");
    }

    Ok(())
}
