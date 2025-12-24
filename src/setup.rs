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
    } else {
        println!(
            "Please install mpv manually (e.g., 'brew install mpv' or 'sudo apt install mpv')."
        );
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn check_and_install_dependencies() -> Result<(), anyhow::Error> {
    // No-op on wasm
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn check_mpv_installed() -> bool {
    Command::new("mpv")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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
