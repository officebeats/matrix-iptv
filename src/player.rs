use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use crate::config::PlayerEngine;

#[cfg(not(target_arch = "wasm32"))]
use std::process::{Child, Command, Stdio};

#[cfg(not(target_arch = "wasm32"))]


#[cfg(not(target_arch = "wasm32"))]


#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[derive(Clone)]
pub struct Player {
    #[cfg(not(target_arch = "wasm32"))]
    process: Arc<Mutex<Option<Child>>>,
    #[cfg(not(target_arch = "wasm32"))]
    ipc_path: Arc<Mutex<Option<PathBuf>>>,
}

impl Player {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                process: Arc::new(Mutex::new(None)),
                ipc_path: Arc::new(Mutex::new(None)),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {}
        }
    }

    /// Start the selected player engine and return the IPC pipe path for monitoring
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn play(&self, url: &str, engine: PlayerEngine, use_default_mpv: bool, smooth_motion: bool) -> Result<(), anyhow::Error> {
        // We SKIP the pre-flight `check_stream_health(url)` here because doing a GET request
        // immediately before launching the player can trigger the IPTV provider's
        // "Max 1 Connection" rule (the health check leaves a ghost connection open
        // for 30-60 seconds on their backend). This would cause the provider
        // to gracefully kill the stream ~45 seconds in!
        
        self.stop();

        match engine {
            PlayerEngine::Mpv => self.play_mpv(url, use_default_mpv, smooth_motion),
            PlayerEngine::Vlc => self.play_vlc(url, smooth_motion),
        }
    }

    async fn check_stream_health(&self, url: &str) -> Result<(), anyhow::Error> {
        // Build a client that mimics the player's behavior (Chrome UA)
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        // We use a stream request but abort immediately to check connectivity/headers.
        // HEAD often fails on some IPTV servers, so a started GET is safer logic-wise.
        let mut result = client.get(url).send().await;
        
        // Resilience: Fallback to DoH if DNS fails for the stream health check
        if let Err(ref e) = result {
            if crate::doh::is_dns_error(e) {
                #[cfg(debug_assertions)]
                println!("DEBUG: Health check DNS error detected for {}. Trying DoH fallback...", url);

                if let Some(resp) = crate::doh::try_doh_fallback(&client, url).await {
                    result = Ok(resp);
                }
            }
        }

        match result {
            Ok(resp) => {
                if resp.status().is_success() || resp.status().is_redirection() {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Stream returned error status: {} (Server might be offline/blocking)", resp.status()))
                }
            },
            Err(e) => {
                // Provide a user-friendly error description using shared DNS detection
                if crate::doh::is_dns_error(&e) {
                     Err(anyhow::anyhow!("Stream Server Unreachable. The host likely does not exist or is blocked (DNS Error). Details: {}", e))
                } else if e.is_connect() || e.is_timeout() {
                     Err(anyhow::anyhow!("Stream Connection Failed. Server may be slow or offline. Details: {}", e))
                } else {
                     Err(anyhow::anyhow!("Stream Check Failed: {}", e))
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn play_mpv(&self, url: &str, use_default_mpv: bool, smooth_motion: bool) -> Result<(), anyhow::Error> {

        // Find mpv executable, checking PATH and common installation locations
        let mpv_path = crate::setup::get_mpv_path().ok_or_else(|| {
            let hint = if cfg!(target_os = "macos") {
                "\n\nHint: On macOS with Homebrew:\n\
                 - Apple Silicon: mpv is typically at /opt/homebrew/bin/mpv\n\
                 - Intel Mac: mpv is typically at /usr/local/bin/mpv\n\n\
                 To fix, add Homebrew to your PATH:\n\
                   export PATH=\"/opt/homebrew/bin:$PATH\"\n\
                 (Add this line to ~/.zshrc or ~/.bash_profile)"
            } else {
                ""
            };
            anyhow::anyhow!(
                "mpv not found. Please install mpv and ensure it's in your PATH.{}",
                hint
            )
        })?;

        // Create a unique IPC path for this session
        let pipe_name = if cfg!(target_os = "windows") {
            format!("\\\\.\\pipe\\mpv_ipc_{}", std::process::id())
        } else {
            format!("/tmp/mpv_ipc_{}", std::process::id())
        };

        let mut cmd = Command::new(&mpv_path);
        
        // Add Referrer validation (Common anti-scraping measure)
        if let Some(scheme_end) = url.find("://") {
            let rest = &url[scheme_end + 3..];
            if let Some(path_start) = rest.find('/') {
                let host = &rest[..path_start];
                let base = format!("{}://{}/", &url[..scheme_end], host);
                cmd.arg(format!("--referrer={}", base));
            }
        }
        
        let _is_live = url.contains("/live/") || url.contains(".m3u8");

        cmd.arg(url)
           .arg("--geometry=1280x720") // Start in 720p window (user preference)
           .arg("--force-window")      // Ensure window opens even if audio-only initially
           .arg("--no-fs")             // DISABLING FULLSCREEN - Force Windowed Mode
           .arg("--osc=yes")           // Enable On Screen Controller for usability
           .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"); // Mimic Chrome to bypass scraping blocks

        // Apply smooth motion interpolation if enabled
        if smooth_motion {
            cmd.arg("--video-sync=display-resample") // Smooth motion sync (required for interpolation)
               .arg("--interpolation=yes") // Frame generation / motion smoothing
               .arg("--tscale=linear")     // Soap opera effect - smooth motion blending (GPU friendly)
               .arg("--tscale-clamp=0.0"); // Allow full blending for maximum smoothness
        }

        // Only apply optimizations if not using default MPV settings
        if !use_default_mpv {
            cmd.arg("--cache=yes")
               // Emulate VLC's small, steady network caching (~15 seconds) to avoid triggering "rip-bot" bans.
               // Large buffers cause burst-then-idle read patterns which IPTV firewalls punish by dropping the socket. 
               .arg("--demuxer-max-bytes=32MiB")
               .arg("--demuxer-max-back-bytes=16MiB")
               .arg("--demuxer-readahead-secs=15") 
               .arg("--cache-pause=no")                // Don't pause rendering while catching up
               .arg("--network-timeout=20")            // Tolerate network blips
               // Force Lavf to unconditionally auto-reconnect on dropped TCP sockets without exiting player
               .arg("--stream-lavf-o=reconnect=1,reconnect_at_eof=1,reconnect_streamed=1,reconnect_delay_max=5")
               // Lavf specific options for better HTTP behavior
               .arg("--tls-verify=no");

            if cfg!(target_os = "windows") {
                cmd.arg("--d3d11-flip=yes")            // Modern Windows presentation (faster)
                   .arg("--gpu-api=d3d11");             // Force D3D11 (faster than OpenGL on Windows)
            } else if cfg!(target_os = "macos") {
                cmd.arg("--gpu-api=opengl");            // Generally safe default for macOS mpv
            }
        }

        // Common settings for both modes
        cmd.arg("--msg-level=all=no")
           .arg("--term-status-msg=no")
           .arg("--input-terminal=no") // Ignore terminal for input
           .arg("--terminal=no")       // Completely disable terminal interactions
           // USER AGENT MASQUERADE: Modern Chrome to avoid throttling
           .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
           // Keep window open if playback fails to see error (optional, maybe off for prod)
           .arg("--keep-open=no")
           // Logging for troubleshooting
           .arg("--log-file=mpv_playback.log")
           // IPC for status monitoring
           .arg(format!("--input-ipc-server={}", pipe_name));

        // Disconnect from terminal input/output to prevent hotkey conflicts
        cmd.stdin(Stdio::null())
           .stdout(Stdio::null())
           .stderr(Stdio::null());

        let child = cmd.spawn();

        match child {
            Ok(child) => {
                {
                    let mut guard = self.process.lock().map_err(|e| {
                        anyhow::anyhow!("Failed to lock process mutex: {}", e)
                    })?;
                    *guard = Some(child);
                }
                {
                    let mut ipc_guard = self.ipc_path.lock().map_err(|e| {
                        anyhow::anyhow!("Failed to lock IPC path mutex: {}", e)
                    })?;
                    *ipc_guard = Some(PathBuf::from(&pipe_name));
                }
                Ok(())
            }
            Err(e) => {
                let hint = if cfg!(target_os = "macos") {
                    format!(
                        "\n\nSearched for mpv at: {}\n\
                        Hint: On Apple Silicon, Homebrew installs to /opt/homebrew/bin.\n\
                        Add this to your ~/.zshrc: export PATH=\"/opt/homebrew/bin:$PATH\"",
                        mpv_path
                    )
                } else {
                    String::new()
                };
                Err(anyhow::anyhow!(
                    "Failed to start mpv: {}.{}",
                    e, hint
                ))
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn play_vlc(&self, url: &str, smooth_motion: bool) -> Result<(), anyhow::Error> {
        // Find vlc executable
        let vlc_path = crate::setup::get_vlc_path().ok_or_else(|| {
            anyhow::anyhow!("VLC not found. Please install VLC.")
        })?;

        let mut cmd = Command::new(&vlc_path);
        
        // Add Referrer validation (Common anti-scraping measure)
        // Manual parsing to avoid adding 'url' crate dependency
        if let Some(scheme_end) = url.find("://") {
            let rest = &url[scheme_end + 3..];
            if let Some(path_start) = rest.find('/') {
                let host = &rest[..path_start];
                let base = format!("{}://{}/", &url[..scheme_end], host);
                cmd.arg(format!("--http-referrer={}", base));
            }
        }

        cmd.arg(url)
           .arg("--no-video-title-show") 
           .arg("--http-user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
           .arg("--http-reconnect")
           .arg("--http-continuous")
           .arg("--clock-jitter=500")           // Allow more jitter in stream clock
           .arg("--network-caching=15000")      // 15 second buffer for TS streams
           .arg("--gnutls-verify-trust-ee=no"); // For VLC HTTPS stability

        // Apply smooth motion (deinterlacing) if enabled
        if smooth_motion {
            cmd.arg("--video-filter=deinterlace")
               .arg("--deinterlace-mode=bob");
        }
        // DISCONNECT from terminal
        cmd.stdin(Stdio::null())
           .stdout(Stdio::null())
           .stderr(Stdio::null());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x00000008;
            cmd.creation_flags(DETACHED_PROCESS);
        }

        let child = cmd.spawn()?;
        
        {
            let mut guard = self.process.lock().map_err(|e| {
                anyhow::anyhow!("Failed to lock process mutex: {}", e)
            })?;
            *guard = Some(child);
        }
        
        Ok(())
    }

    /// Read the last few lines of the player logs to find errors
    pub fn get_last_error_from_log(&self) -> Option<String> {
        let logs = ["mpv_playback.log", "vlc_playback.log"];
        
        for log_file in logs {
            if let Ok(content) = std::fs::read_to_string(log_file) {
                let lines: Vec<&str> = content.lines().rev().take(15).collect();
                for line in lines {
                    let lower = line.to_lowercase();
                    if lower.contains("error") || lower.contains("failed") || lower.contains("fatal") {
                        // Clean up common VLC/MPV prefixes for cleaner UI display
                        let cleaned = line.split("]: ").last().unwrap_or(line);
                        return Some(cleaned.to_string());
                    }
                }
            }
        }
        None
    }

    /// Check if MPV is still running (process alive)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn is_running(&self) -> bool {
        if let Ok(mut guard) = self.process.lock() {
            if let Some(ref mut child) = *guard {
                // try_wait returns Ok(Some(status)) if exited, Ok(None) if still running
                match child.try_wait() {
                    Ok(Some(_)) => false, // Process has exited
                    Ok(None) => true,     // Still running
                    Err(_) => false,      // Error, assume not running
                }
            } else {
                false
            }
        } else {
            false // Mutex poisoned, assume not running
        }
    }

    /// Wait for MPV to actually start playing by polling process status
    /// Returns Ok(true) if playback confirmed, Ok(false) if process died, Err on timeout
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn wait_for_playback(&self, timeout_ms: u64) -> Result<bool, anyhow::Error> {
        use tokio::time::{sleep, Duration, Instant};

        let start = Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        // Give MPV a moment to initialize
        sleep(Duration::from_millis(500)).await;

        // Poll until process is confirmed running and has had time to buffer
        while start.elapsed() < timeout {
            if !self.is_running() {
                // Process died, playback failed
                return Ok(false);
            }

            // Check if process has been alive for at least 2 seconds
            // This indicates MPV successfully connected and is playing
            if start.elapsed() > Duration::from_millis(2000) {
                return Ok(true);
            }

            sleep(Duration::from_millis(200)).await;
        }

        // If we reached here and process is still running, consider it a success
        Ok(self.is_running())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn play(&self, url: &str, _engine: PlayerEngine, _use_default_mpv: bool, _smooth_motion: bool) -> Result<(), anyhow::Error> {
        self.stop();
        if let Some(win) = window() {
            let _ = win.alert_with_message(&format!("Play stream: {}", url));
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn stop(&self) {
        if let Ok(mut guard) = self.process.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }

        if let Ok(mut ipc_guard) = self.ipc_path.lock() {
            *ipc_guard = None;
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn stop(&self) {
        if let Some(_win) = window() {
            web_sys::console::log_1(&"Stopping stream".into());
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.stop();
    }
}
