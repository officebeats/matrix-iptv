#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

/// Common locations where mpv might be installed on macOS/Linux
#[cfg(not(target_arch = "wasm32"))]
const MPV_CANDIDATE_PATHS: &[&str] = &[
    "/opt/homebrew/bin/mpv",    // macOS Apple Silicon Homebrew
    "/usr/local/bin/mpv",       // macOS Intel Homebrew / some Linux
    "/usr/bin/mpv",             // Common system path (Linux)
    "/snap/bin/mpv",            // Snap on Linux
];

/// Checks if a given path points to an executable mpv binary
#[cfg(not(target_arch = "wasm32"))]
fn is_valid_mpv_at_path(path: &str) -> bool {
    let p = Path::new(path);
    if !p.exists() || !p.is_file() {
        return false;
    }
    
    // On Unix, check if it's executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = p.metadata() {
            let mode = metadata.permissions().mode();
            if mode & 0o111 == 0 {
                return false; // No execute bits
            }
        }
    }
    
    // Verify it actually runs
    Command::new(path)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns the path to mpv executable, searching common locations if not in PATH
/// This is the primary function for finding mpv across platforms
#[cfg(not(target_arch = "wasm32"))]
pub fn get_mpv_path() -> Option<String> {
    // First, try standard PATH lookup
    if Command::new("mpv")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("mpv".to_string());
    }
    
    // Fallback: search common installation paths (primarily for macOS Homebrew)
    for candidate in MPV_CANDIDATE_PATHS {
        if is_valid_mpv_at_path(candidate) {
            return Some(candidate.to_string());
        }
    }
    
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub fn check_and_install_dependencies() -> Result<(), anyhow::Error> {
    print!("Checking dependencies... ");
    io::stdout().flush()?;

    if let Some(mpv_path) = get_mpv_path() {
        if mpv_path == "mpv" {
            println!("✓ mpv found in PATH.");
        } else {
            println!("✓ mpv found at: {}", mpv_path);
        }
        return Ok(());
    }

    println!("✗ mpv NOT found.");

    if cfg!(target_os = "windows") {
        println!("Attempting to install mpv using winget...");
        install_mpv_windows()?;
    } else if cfg!(target_os = "macos") {
        println!("Attempting to install mpv using homebrew...");
        install_mpv_macos()?;
    } else {
        println!(
            "Please install mpv manually (e.g., 'sudo apt install mpv')."
        );
        return Err(anyhow::anyhow!(
            "mpv is required but not found.\n\n\
            Searched locations:\n  - PATH\n  {}\n\n\
            On macOS with Homebrew, try: brew install mpv\n\
            On Linux, try: sudo apt install mpv",
            MPV_CANDIDATE_PATHS.join("\n  - ")
        ));
    }

    // Double check after installation
    if get_mpv_path().is_some() {
        println!("✓ mpv install verified.");
        Ok(())
    } else {
        let hint = if cfg!(target_os = "macos") {
            "\n\nHint: On Apple Silicon, Homebrew installs to /opt/homebrew/bin.\n\
             You may need to add this to your shell profile:\n\
             export PATH=\"/opt/homebrew/bin:$PATH\""
        } else {
            ""
        };
        Err(anyhow::anyhow!(
            "mpv was installed but still not found. You may need to restart your terminal or add it to PATH manually.{}", 
            hint
        ))
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get_mpv_path() -> Option<String> {
    None
}

#[cfg(target_arch = "wasm32")]
pub fn check_and_install_dependencies() -> Result<(), anyhow::Error> {
    // No-op on wasm
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn install_mpv_macos() -> Result<(), anyhow::Error> {
    // Check if brew is installed
    let brew_check = Command::new("brew")
        .arg("--version")
        .output()
        .is_ok();

    if !brew_check {
        return Err(anyhow::anyhow!("Homebrew ('brew') is required for headless installation on macOS. Please install it first from https://brew.sh/"));
    }

    println!("Running: brew install mpv");
    let status = Command::new("brew")
        .args(&["install", "mpv"])
        .status()?;

    if status.success() {
        println!("✓ mpv installed successfully via brew.");
        Ok(())
    } else {
        println!("brew install mpv failed, trying cask...");
        let status_cask = Command::new("brew")
            .args(&["install", "--cask", "mpv"])
            .status()?;
        
        if status_cask.success() {
            println!("✓ mpv installed successfully via brew cask.");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to install mpv via homebrew."))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn install_mpv_windows() -> Result<(), anyhow::Error> {
    // Try installing specific ID first
    println!("Running: winget install -e --id \"9P3JFR0CLLL6\" --accept-source-agreements --accept-package-agreements");
    let status = Command::new("winget")
        .args(&[
            "install",
            "-e",
            "--id",
            "9P3JFR0CLLL6",
            "--accept-source-agreements",
            "--accept-package-agreements",
        ])
        .status();

    if let Ok(s) = status {
        if s.success() {
            println!("✓ mpv installed successfully.");
            return Ok(());
        }
    }

    println!("Specific ID failed, trying generic 'mpv'...");
    let status_generic = Command::new("winget")
        .args(&[
            "install",
            "-e",
            "mpv",
            "--accept-source-agreements",
            "--accept-package-agreements",
        ])
        .status()?;

    if status_generic.success() {
        println!("✓ mpv installed successfully.");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to install mpv via winget."))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn check_for_updates(tx: tokio::sync::mpsc::Sender<crate::app::AsyncAction>, manual: bool) {
    let current_version = env!("CARGO_PKG_VERSION");
    let client = reqwest::Client::builder()
        .user_agent("matrix-iptv-cli-updater")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    // Use a GET request to the latest release to check the tag
    // GitHub redirects /latest to the specific tag URL
    if let Ok(resp) = client.get("https://github.com/officebeats/matrix-iptv/releases/latest")
        .send()
        .await 
    {
        let final_url = resp.url().to_string();
        // URL is likely https://github.com/officebeats/matrix-iptv/releases/tag/v3.0.9
        if let Some(tag) = final_url.split("/tag/").last() {
            let tag = tag.trim_start_matches('v');
            if tag != current_version && !tag.is_empty() {
                let _ = tx.send(crate::app::AsyncAction::UpdateAvailable(tag.to_string())).await;
            } else if manual {
                let _ = tx.send(crate::app::AsyncAction::NoUpdateFound).await;
            }
        }
    } else if manual {
        let _ = tx.send(crate::app::AsyncAction::Error("Failed to check for updates. Please check your connection.".to_string())).await;
    }
}
