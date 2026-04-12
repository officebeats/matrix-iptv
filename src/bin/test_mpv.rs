use matrix_iptv_lib::api::XtreamClient;
use matrix_iptv_lib::config::AppConfig;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    println!("Loading config...");
    let config = AppConfig::load().unwrap();
    let account = config.accounts.first().expect("No accounts found");

    println!("Authenticating with account: {}", account.name);
    let client = XtreamClient::new(
        account.base_url.clone(),
        account.username.clone(),
        account.password.clone(),
    );
    let _ = client.authenticate().await?;

    println!("Fetching Live Categories...");
    let categories = client.get_live_categories().await?;
    let target_cat = categories
        .iter()
        .find(|c| c.category_name.contains("NBA"))
        .expect("Could not find NBA category");

    println!(
        "Fetching Streams for Category: {} (ID: {})",
        target_cat.category_name, target_cat.category_id
    );
    let streams = client
        .get_live_streams(&target_cat.category_id, None)
        .await?;

    let target_stream = streams
        .iter()
        .find(|s| s.name.contains("NBA") || s.name.contains("Utah") || s.name.contains("Warriors"))
        .or_else(|| streams.first())
        .expect("Could not find any stream in this category")
        .clone();

    let stream_id = target_stream.stream_id.to_string();

    let url = client.get_stream_url(&stream_id, "ts");
    println!(
        "\nFound Stream!\nTitle: {}\nURL: [HIDDEN FOR OUTPUT SECURITY]\n",
        target_stream.name
    );

    println!("Launching MPV...");

    // Find mpv executable
    let mpv_path = matrix_iptv_lib::setup::get_mpv_path().expect("Could not find mpv");

    // Start mpv
    let mut cmd = std::process::Command::new(mpv_path);
    cmd.arg(&url);
    cmd.arg("--force-window=immediate");
    cmd.arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");

    // Add referrer matching the base url (very common IPTV requirement)
    if let Ok(parsed_url) = reqwest::Url::parse(&url) {
        let origin = format!(
            "{}://{}",
            parsed_url.scheme(),
            parsed_url.host_str().unwrap_or("")
        );
        cmd.arg(format!("--referrer={}/", origin));
    } else {
        cmd.arg(format!("--referrer={}", url));
    }

    // v4.0.17: 'Anti-Loop' Profile
    cmd.arg("--demuxer-max-bytes=128MiB");
    cmd.arg("--demuxer-max-back-bytes=50MiB");
    cmd.arg("--demuxer-readahead-secs=20");
    cmd.arg("--demuxer-thread=yes");
    cmd.arg("--network-timeout=60");
    cmd.arg("--cache-pause=yes");
    cmd.arg("--keep-open=yes");
    cmd.arg("--stream-lavf-o=reconnect=1,reconnect_at_eof=1,reconnect_streamed=1,reconnect_delay_max=5,multiple_requests=1");
    cmd.arg("--demuxer-lavf-o=analyzeduration=3000000,probesize=3000000,fflags=+genpts+igndts");
    cmd.arg("--tls-verify=no");
    cmd.arg("--hwdec=auto-safe");
    cmd.arg("--ytdl=no");
    cmd.arg("--msg-level=all=v");
    cmd.arg("--log-file=mpv_trace.log");

    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn()?;

    println!("Testing stability... waiting 130 seconds...");
    let start_time = std::time::Instant::now();

    while start_time.elapsed().as_secs() < 130 {
        sleep(Duration::from_secs(1)).await;
        if let Ok(Some(status)) = child.try_wait() {
            println!(
                "\n[FAILURE] Player died after {} seconds! Status: {}",
                start_time.elapsed().as_secs(),
                status
            );
            let mut stdout = String::new();
            let mut stderr = String::new();
            if let Some(mut out) = child.stdout.take() {
                use std::io::Read;
                out.read_to_string(&mut stdout).unwrap_or_default();
            }
            if let Some(mut err) = child.stderr.take() {
                use std::io::Read;
                err.read_to_string(&mut stderr).unwrap_or_default();
            }
            println!("STDOUT:\n{}", stdout);
            println!("STDERR:\n{}", stderr);
            return Ok(());
        } else {
            if start_time.elapsed().as_secs() % 5 == 0 {
                print!(".");
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
        }
    }

    println!("\n[SUCCESS] Player is still running after >2 mins of playback!");
    let _ = child.kill();
    Ok(())
}
