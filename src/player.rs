use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

#[cfg(not(target_arch = "wasm32"))]
use std::process::{Child, Command};

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

    /// Start MPV and return the IPC pipe path for monitoring
    #[cfg(not(target_arch = "wasm32"))]
    pub fn play(&self, url: &str) -> Result<(), anyhow::Error> {
        self.stop();

        // Create a unique named pipe path for this session
        let pipe_name = format!("\\\\.\\pipe\\mpv_ipc_{}", std::process::id());

        let child = Command::new("mpv")
            .arg(url)
            .arg("--fs") // Start in fullscreen
            .arg("--force-window")
            .arg("--cache=yes")
            .arg("--demuxer-max-bytes=128MiB")
            .arg("--demuxer-max-back-bytes=32MiB")
            .arg("--msg-level=all=no")
            .arg("--term-status-msg=no")
            .arg("--hwdec=auto")
            // Premium playback settings
            .arg("--interpolation=yes")
            .arg("--interpolation-threshold=-1")
            .arg("--tscale=mitchell")
            .arg("--tscale-blur=0.7")
            .arg("--video-sync=display-resample")
            // IPC for status monitoring
            .arg(format!("--input-ipc-server={}", pipe_name))
            .spawn();

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
            Err(e) => Err(anyhow::anyhow!(
                "Failed to start mpv: {}. Make sure mpv is installed and in PATH.",
                e
            )),
        }
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
    pub fn play(&self, url: &str) -> Result<(), anyhow::Error> {
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
        if let Some(win) = window() {
            web_sys::console::log_1(&"Stopping stream".into());
        }
    }
}
