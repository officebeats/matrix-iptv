#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;

#[cfg(not(target_arch = "wasm32"))]
pub fn check_and_install_dependencies() -> Result<(), anyhow::Error> {
    print!("Checking dependencies... ");
    io::stdout().flush()?;

    if check_mpv_installed() {
        println!("✓ mpv found.");
        return Ok(());
    }

    println!("x mpv NOT found.");

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
        return Err(anyhow::anyhow!("mpv is required but not found in PATH."));
    }

    // Double check after installation
    if check_mpv_installed() {
        println!("✓ mpv install verified.");
        Ok(())
    } else {
        Err(anyhow::anyhow!("mpv was installed but still not found in PATH. You may need to restart your terminal or add it to PATH manually."))
    }
}

#[cfg(target_arch = "wasm32")]
pub fn check_and_install_dependencies() -> Result<(), anyhow::Error> {
    // No-op on wasm
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn check_mpv_installed() -> bool {
    // We check --version to ensure it's not just a broken symlink or non-runnable file
    Command::new("mpv")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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
