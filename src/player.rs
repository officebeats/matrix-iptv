#![allow(dead_code)]
use crate::config::PlayerEngine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

#[cfg(not(target_arch = "wasm32"))]
use std::process::{Child, Command, Stdio};

#[cfg(not(target_arch = "wasm32"))]
use tokio::time::{sleep, Duration};

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(target_arch = "wasm32"))]
#[cfg(target_arch = "wasm32")]
use web_sys::window;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StreamFormat {
    Ts,
    M3u8,
    Mp4,
    Json,
}

impl StreamFormat {
    pub fn extension(&self) -> &str {
        match self {
            Self::Ts => "ts",
            Self::M3u8 => "m3u8",
            Self::Mp4 => "mp4",
            Self::Json => "json",
        }
    }
}

#[derive(Clone)]
pub struct Player {
    #[cfg(not(target_arch = "wasm32"))]
    process: Arc<Mutex<Option<Child>>>,
    #[cfg(not(target_arch = "wasm32"))]
    ipc_path: Arc<Mutex<Option<PathBuf>>>,
    #[cfg(not(target_arch = "wasm32"))]
    last_error: Arc<Mutex<Option<String>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaybackError {
    pub error_type: PlaybackErrorType,
    pub message: String,
    pub hint: Option<String>,
    pub recoverable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlaybackErrorType {
    MpvNotFound,
    StreamUnreachable,
    InvalidFormat,
    NetworkTimeout,
    AuthExpired,
    ProviderBlocked,
    Unknown,
}

impl PlaybackError {
    pub fn new(error_type: PlaybackErrorType, message: String) -> Self {
        let (hint, recoverable) = match &error_type {
            PlaybackErrorType::MpvNotFound => (
                Some("Please install MPV: winget install mpv (Windows), brew install mpv (Mac), or sudo apt install mpv (Linux)".to_string()),
                false
            ),
            PlaybackErrorType::StreamUnreachable => (
                Some("Try a different stream format or check your internet connection. The stream may be offline.".to_string()),
                true
            ),
            PlaybackErrorType::InvalidFormat => (
                Some("Trying different stream format (m3u8/ts/mp4)...".to_string()),
                true
            ),
            PlaybackErrorType::NetworkTimeout => (
                Some("Network timeout. Trying with increased buffer settings...".to_string()),
                true
            ),
            PlaybackErrorType::AuthExpired => (
                Some("Your IPTV subscription may have expired. Please check your account.".to_string()),
                false
            ),
            PlaybackErrorType::ProviderBlocked => (
                Some("Provider may be blocking the connection. Try enabling VPN or contact your provider.".to_string()),
                true
            ),
            PlaybackErrorType::Unknown => (None, true),
        };

        Self {
            error_type,
            message,
            hint,
            recoverable,
        }
    }
}

impl Player {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                process: Arc::new(Mutex::new(None)),
                ipc_path: Arc::new(Mutex::new(None)),
                last_error: Arc::new(Mutex::new(None)),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {}
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_last_error(&self, error: Option<String>) {
        if let Ok(mut guard) = self.last_error.lock() {
            *guard = error;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_last_error(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|g| g.clone())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn get_last_error(&self) -> Option<String> {
        None
    }

    /// Start the selected player engine with automatic retry and fallback
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn play(
        &self,
        url: &str,
        engine: PlayerEngine,
        use_default_mpv: bool,
        smooth_motion: bool,
    ) -> Result<(), anyhow::Error> {
        self.stop();

        match engine {
            PlayerEngine::Mpv => {
                match self
                    .play_mpv_with_retry(url, use_default_mpv, smooth_motion, 0)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        if crate::setup::get_vlc_path().is_some() {
                            self.play_vlc(url, smooth_motion)
                        } else {
                            Err(e)
                        }
                    }
                }
            }
            PlayerEngine::Vlc => self.play_vlc(url, smooth_motion),
        }
    }

    /// Try playing with different stream formats as fallback
    #[cfg(not(target_arch = "wasm32"))]
    async fn play_mpv_with_retry(
        &self,
        url: &str,
        use_default_mpv: bool,
        smooth_motion: bool,
        attempt: u32,
    ) -> Result<(), anyhow::Error> {
        let result = self.play_mpv(url, use_default_mpv, smooth_motion);

        if result.is_err() && attempt < 3 {
            if let Some(base_url) = self.extract_stream_base_url(url) {
                let formats: Vec<&str> = if url.contains(".m3u8") {
                    vec!["ts", "mp4"]
                } else {
                    vec!["m3u8", "ts", "mp4"]
                };

                if let Some(format) = formats.get(attempt as usize) {
                    let new_url = format!("{}.{}", base_url, format);
                    return Box::pin(self.play_mpv_with_retry(
                        &new_url,
                        use_default_mpv,
                        smooth_motion,
                        attempt + 1,
                    ))
                    .await;
                }
            }
        }

        result
    }

    /// Extract base URL without extension for format fallback
    #[cfg(not(target_arch = "wasm32"))]
    fn extract_stream_base_url(&self, url: &str) -> Option<String> {
        let base = url.rsplit('.').last()?;
        let formats = ["ts", "m3u8", "mp4", "json"];
        if formats.contains(&base) {
            let pos = url.len() - base.len() - 1;
            Some(url[..pos].to_string())
        } else {
            None
        }
    }

    /// Diagnose playback failure and return helpful error message
    pub fn diagnose_playback_failure(&self, error: &str) -> PlaybackError {
        let error_lower = error.to_lowercase();

        if error_lower.contains("mpv not found") || error_lower.contains("cannot find") {
            return PlaybackError::new(PlaybackErrorType::MpvNotFound, error.to_string());
        }

        if error_lower.contains("connection")
            || error_lower.contains("refused")
            || error_lower.contains("unreachable")
        {
            return PlaybackError::new(PlaybackErrorType::StreamUnreachable, error.to_string());
        }

        if error_lower.contains("timeout") || error_lower.contains("timed out") {
            return PlaybackError::new(PlaybackErrorType::NetworkTimeout, error.to_string());
        }

        if error_lower.contains("403")
            || error_lower.contains("forbidden")
            || error_lower.contains("blocked")
        {
            return PlaybackError::new(PlaybackErrorType::ProviderBlocked, error.to_string());
        }

        if error_lower.contains("401")
            || error_lower.contains("unauthorized")
            || error_lower.contains("expired")
        {
            return PlaybackError::new(PlaybackErrorType::AuthExpired, error.to_string());
        }

        if error_lower.contains("format")
            || error_lower.contains("invalid")
            || error_lower.contains("unsupported")
        {
            return PlaybackError::new(PlaybackErrorType::InvalidFormat, error.to_string());
        }

        PlaybackError::new(PlaybackErrorType::Unknown, error.to_string())
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
                    Err(anyhow::anyhow!(
                        "Stream returned error status: {} (Server might be offline/blocking)",
                        resp.status()
                    ))
                }
            }
            Err(e) => {
                // Provide a user-friendly error description using shared DNS detection
                if crate::doh::is_dns_error(&e) {
                    Err(anyhow::anyhow!("Stream Server Unreachable. The host likely does not exist or is blocked (DNS Error). Details: {}", e))
                } else if e.is_connect() || e.is_timeout() {
                    Err(anyhow::anyhow!(
                        "Stream Connection Failed. Server may be slow or offline. Details: {}",
                        e
                    ))
                } else {
                    Err(anyhow::anyhow!("Stream Check Failed: {}", e))
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn play_mpv(
        &self,
        url: &str,
        use_default_mpv: bool,
        smooth_motion: bool,
    ) -> Result<(), anyhow::Error> {
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
           .arg("--force-window=immediate")
           .arg("--no-fs")
           .arg("--osc=yes")
           .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");

        // Apply smooth motion interpolation if enabled
        if smooth_motion {
            cmd.arg("--video-sync=display-resample") // Smooth motion sync (required for interpolation)
                .arg("--interpolation=yes") // Frame generation / motion smoothing
                .arg("--tscale=linear") // Soap opera effect - smooth motion blending
                .arg("--tscale-clamp=0.0"); // Allow full blending for maximum smoothness
        }

        // Only apply optimizations if not using default MPV settings
        // Safe defaults that work across platforms (macOS, Windows, Linux)
        if !use_default_mpv {
            cmd.arg("--cache=yes")
               .arg("--demuxer-max-bytes=128MiB") // Increased cache size
               .arg("--demuxer-max-back-bytes=50MiB") // Keep backward buffer
               .arg("--demuxer-readahead-secs=20") // Read more ahead
               .arg("--demuxer-thread=yes")
               .arg("--cache-pause=yes") // Let MPV buffer gracefully instead of stuttering
               .arg("--network-timeout=60")
               .arg("--keep-open=yes")
               .arg("--video-sync=audio")
               .arg("--stream-lavf-o=reconnect=1,reconnect_at_eof=1,reconnect_streamed=1,reconnect_delay_max=5,multiple_requests=1")
               .arg("--demuxer-lavf-o=analyzeduration=3000000,probesize=3000000,fflags=+genpts+igndts")
               .arg("--ytdl=no")
               .arg("--tls-verify=no")
               .arg("--hwdec=auto-safe"); // Enable hardware decoding to reduce CPU stuttering

            if cfg!(target_os = "windows") {
                cmd.arg("--d3d11-flip=yes").arg("--gpu-api=d3d11");
            } else if cfg!(target_os = "macos") {
                // Don't force gpu-api on macOS - let mpv auto-detect
                // Some mpv builds don't support explicit opengl
            }
        }

        // Common settings for both modes
        cmd.arg("--msg-level=all=no")
           .arg("--term-status-msg=no")
           .arg("--input-terminal=no")
           .arg("--terminal=no")
           .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
           .arg("--keep-open=no")
           .arg("--log-file=mpv_playback.log")
           .arg(format!("--input-ipc-server={}", pipe_name));

        // Disconnect from terminal input/output to prevent hotkey conflicts
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let child = cmd.spawn();

        match child {
            Ok(child) => {
                {
                    let mut guard = self
                        .process
                        .lock()
                        .map_err(|e| anyhow::anyhow!("Failed to lock process mutex: {}", e))?;
                    *guard = Some(child);
                }
                {
                    let mut ipc_guard = self
                        .ipc_path
                        .lock()
                        .map_err(|e| anyhow::anyhow!("Failed to lock IPC path mutex: {}", e))?;
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
                Err(anyhow::anyhow!("Failed to start mpv: {}.{}", e, hint))
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn play_vlc(&self, url: &str, smooth_motion: bool) -> Result<(), anyhow::Error> {
        // Find vlc executable
        let vlc_path = crate::setup::get_vlc_path()
            .ok_or_else(|| anyhow::anyhow!("VLC not found. Please install VLC."))?;

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
            let mut guard = self
                .process
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock process mutex: {}", e))?;
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
                    if lower.contains("error")
                        || lower.contains("failed")
                        || lower.contains("fatal")
                    {
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

    /// Monitor MPV IPC socket for error events
    /// This provides real-time error detection during playback
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "windows")))]
    pub async fn monitor_ipc_errors(&self, duration_ms: u64) -> Option<String> {
        let ipc_path = self.ipc_path.lock().ok()?.clone()?;
        let duration = Duration::from_millis(duration_ms);
        let start = std::time::Instant::now();

        while start.elapsed() < duration {
            if !self.is_running() {
                return self.get_last_error_from_log();
            }

            // Try to read from IPC socket
            if let Ok(ipc_data) = self.read_ipc_socket(&ipc_path).await {
                if let Some(error) = self.parse_ipc_error(&ipc_data) {
                    return Some(error);
                }
            }

            sleep(Duration::from_millis(500)).await;
        }
        None
    }

    /// Read from MPV IPC socket
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "windows")))]
    async fn read_ipc_socket(&self, path: &PathBuf) -> Result<String, std::io::Error> {
        use tokio::io::AsyncReadExt;
        use tokio::net::UnixStream;

        let mut stream = match UnixStream::connect(path).await {
            Ok(s) => s,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "IPC not ready",
                ))
            }
        };

        let mut buf = [0u8; 4096];
        match stream.read(&mut buf).await {
            Ok(0) => Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "IPC closed",
            )),
            Ok(n) => Ok(String::from_utf8_lossy(&buf[..n]).to_string()),
            Err(e) => Err(e),
        }
    }

    /// Windows stub for read_ipc_socket
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    async fn read_ipc_socket(&self, _path: &PathBuf) -> Result<String, std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unix sockets not available on Windows",
        ))
    }

    /// Windows stub for monitor_ipc_errors
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    pub async fn monitor_ipc_errors(&self, _duration_ms: u64) -> Option<String> {
        None
    }

    /// Parse IPC data for error messages
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "windows")))]
    fn parse_ipc_error(&self, data: &str) -> Option<String> {
        let error_patterns = ["error", "failed", "abort", "network"];
        let lower = data.to_lowercase();

        for pattern in error_patterns {
            if lower.contains(pattern) {
                if let Some(line) = data.lines().find(|l| l.to_lowercase().contains(pattern)) {
                    return Some(line.to_string());
                }
            }
        }
        None
    }

    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    fn parse_ipc_error(&self, _data: &str) -> Option<String> {
        None
    }

    /// Enhanced playback check with IPC error monitoring
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "windows")))]
    pub async fn wait_for_playback_with_monitoring(
        &self,
        timeout_ms: u64,
    ) -> Result<bool, anyhow::Error> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        sleep(Duration::from_millis(500)).await;

        while start.elapsed() < timeout {
            // Check if process died
            if !self.is_running() {
                // Try to get error from logs
                if let Some(log_error) = self.get_last_error_from_log() {
                    self.set_last_error(Some(log_error.clone()));
                    return Err(anyhow::anyhow!("Playback failed: {}", log_error));
                }
                return Ok(false);
            }

            // Check for IPC errors after initial startup
            if start.elapsed() > Duration::from_millis(3000) {
                if let Some(ipc_error) = self.monitor_ipc_errors(2000).await {
                    self.set_last_error(Some(ipc_error.clone()));
                    return Err(anyhow::anyhow!("Playback error: {}", ipc_error));
                }

                return Ok(true);
            }

            sleep(Duration::from_millis(200)).await;
        }

        Ok(self.is_running())
    }

    /// Simplified playback check for Windows (no IPC monitoring)
    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    pub async fn wait_for_playback_with_monitoring(
        &self,
        timeout_ms: u64,
    ) -> Result<bool, anyhow::Error> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);

        sleep(Duration::from_millis(500)).await;

        while start.elapsed() < timeout {
            if !self.is_running() {
                if let Some(log_error) = self.get_last_error_from_log() {
                    return Err(anyhow::anyhow!("Playback failed: {}", log_error));
                }
                return Ok(false);
            }

            if start.elapsed() > Duration::from_millis(3000) {
                return Ok(true);
            }

            sleep(Duration::from_millis(200)).await;
        }

        Ok(self.is_running())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn play(
        &self,
        url: &str,
        _engine: PlayerEngine,
        _use_default_mpv: bool,
        _smooth_motion: bool,
    ) -> Result<(), anyhow::Error> {
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

    /// Check for common playback issues and return self-healing suggestions
    #[cfg(not(target_arch = "wasm32"))]
    pub fn check_and_suggest_fixes(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Check if mpv is installed
        if crate::setup::get_mpv_path().is_none() {
            suggestions.push("MPV not found. Install with: winget install mpv (Windows), brew install mpv (Mac), or sudo apt install mpv (Linux)".to_string());
        }

        // Check if old log files exist and are large (might indicate recurring issues)
        if let Ok(meta) = std::fs::metadata("mpv_playback.log") {
            if meta.len() > 1_000_000 {
                suggestions.push(
                    "Large log file detected. Consider deleting mpv_playback.log to free space"
                        .to_string(),
                );
            }
        }

        // Check player config
        if let Ok(config) = crate::config::AppConfig::load() {
            if config.preferred_player == PlayerEngine::Mpv && config.use_default_mpv {
                suggestions.push("Using default MPV settings. Try enabling custom settings for better stream compatibility".to_string());
            }
        }

        suggestions
    }

    /// Auto-detect stream URL issues and suggest fixes
    pub fn analyze_stream_url(url: &str) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for http vs https
        if url.starts_with("http://") {
            issues.push("Stream URL uses HTTP (insecure). Trying HTTPS may help".to_string());
        }

        // Check for missing referrer
        if !url.contains("/live/") && !url.contains(".m3u8") {
            issues.push(
                "Non-standard URL format detected. Stream may require special player settings"
                    .to_string(),
            );
        }

        // Check for query parameters that might indicate auth issues
        if url.contains("&token=") || url.contains("?token=") {
            issues.push("URL contains token - ensure it hasn't expired".to_string());
        }

        issues
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        self.stop();
    }
}
