#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;

/// Common locations where mpv might be installed on macOS/Linux
#[cfg(not(target_arch = "wasm32"))]
const MPV_CANDIDATE_PATHS: &[&str] = &[
    "/opt/homebrew/bin/mpv", // macOS Apple Silicon Homebrew
    "/usr/local/bin/mpv",    // macOS Intel Homebrew / some Linux
    "/usr/bin/mpv",          // Common system path (Linux)
    "/snap/bin/mpv",         // Snap on Linux
];

/// Common locations where vlc might be installed on macOS/Linux
#[cfg(not(target_arch = "wasm32"))]
const VLC_CANDIDATE_PATHS: &[&str] = &[
    "/Applications/VLC.app/Contents/MacOS/VLC", // macOS Standard
    "/usr/bin/vlc",                             // Linux Standard
    "/usr/local/bin/vlc",                       // Manual Install
    "/snap/bin/vlc",                            // Snap on Linux
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

/// Returns the path to vlc executable, searching common locations if not in PATH
#[cfg(not(target_arch = "wasm32"))]
pub fn get_vlc_path() -> Option<String> {
    // First, try standard PATH lookup
    if Command::new("vlc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("vlc".to_string());
    }

    // Windows specific common paths
    if cfg!(windows) {
        let win_paths = [
            "C:\\Program Files\\VideoLAN\\VLC\\vlc.exe",
            "C:\\Program Files (x86)\\VideoLAN\\VLC\\vlc.exe",
        ];
        for path in win_paths {
            if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
    }

    // Fallback: search common installation paths
    for candidate in VLC_CANDIDATE_PATHS {
        let p = Path::new(candidate);
        if p.exists() && p.is_file() {
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
            print!("✓ mpv found. ");
        } else {
            print!("✓ mpv found at: {}. ", mpv_path);
        }
    } else {
        println!("\n✗ mpv NOT found.");
        if cfg!(target_os = "windows") {
            println!("Attempting to install mpv using winget...");
            install_mpv_windows()?;
        } else if cfg!(target_os = "macos") {
            println!("Attempting to install mpv using homebrew...");
            install_mpv_macos()?;
        }
    }

    if let Some(vlc_path) = get_vlc_path() {
        if vlc_path == "vlc" {
            println!("✓ vlc found.");
        } else {
            println!("✓ vlc found at: {}", vlc_path);
        }
    } else {
        println!("\n✗ vlc NOT found.");
        if cfg!(target_os = "windows") {
            println!("Attempting to install vlc using winget...");
            install_vlc_windows()?;
        } else if cfg!(target_os = "macos") {
            println!("Attempting to install vlc using homebrew...");
            install_vlc_macos()?;
        }
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn get_vlc_path() -> Option<String> {
    None
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

/// Find the Homebrew executable, checking common locations
#[cfg(not(target_arch = "wasm32"))]
fn find_brew() -> Option<String> {
    // Try PATH first
    if Command::new("brew")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Some("brew".to_string());
    }

    // Check common Homebrew locations
    let candidates = [
        "/opt/homebrew/bin/brew",              // Apple Silicon
        "/usr/local/bin/brew",                 // Intel Mac
        "/home/linuxbrew/.linuxbrew/bin/brew", // Linux Homebrew
    ];

    for candidate in candidates {
        let path = Path::new(candidate);
        if path.exists()
            && path.is_file()
            && Command::new(candidate)
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        {
            return Some(candidate.to_string());
        }
    }

    None
}

/// Install Homebrew headlessly on macOS
#[cfg(not(target_arch = "wasm32"))]
fn install_homebrew() -> Result<String, anyhow::Error> {
    println!("Installing Homebrew...");
    println!("This may take a few minutes. Please wait...");

    // Run the official Homebrew installer script with NONINTERACTIVE flag
    let status = Command::new("/bin/bash")
        .args([
            "-c",
            "NONINTERACTIVE=1 /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        ])
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Failed to install Homebrew. Please install manually from https://brew.sh/"
        ));
    }

    // Find brew after installation
    if let Some(brew_path) = find_brew() {
        println!("✓ Homebrew installed successfully.");
        Ok(brew_path)
    } else {
        Err(anyhow::anyhow!(
            "Homebrew was installed but could not be found. \
            You may need to add it to your PATH and restart the application."
        ))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn install_mpv_macos() -> Result<(), anyhow::Error> {
    // Find or install Homebrew
    let brew_path = match find_brew() {
        Some(path) => {
            println!("✓ Found Homebrew at: {}", path);
            path
        }
        None => {
            println!("Homebrew not found. Installing...");
            install_homebrew()?
        }
    };

    println!("Installing mpv via Homebrew...");
    println!("Running: {} install mpv", brew_path);

    let status = Command::new(&brew_path).args(["install", "mpv"]).status()?;

    if status.success() {
        println!("✓ mpv installed successfully via Homebrew.");
        return Ok(());
    }

    // Try cask as fallback
    println!("Formula install failed, trying cask...");
    let status_cask = Command::new(&brew_path)
        .args(["install", "--cask", "mpv"])
        .status()?;

    if status_cask.success() {
        println!("✓ mpv installed successfully via Homebrew Cask.");
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to install mpv via Homebrew. \
            Please try manually: brew install mpv"
        ))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn install_mpv_windows() -> Result<(), anyhow::Error> {
    // Try installing specific ID first
    println!("Running: winget install -e --id \"9P3JFR0CLLL6\" --accept-source-agreements --accept-package-agreements");
    let status = Command::new("winget")
        .args([
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
        .args([
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
fn install_vlc_macos() -> Result<(), anyhow::Error> {
    let brew_path = match find_brew() {
        Some(path) => path,
        None => install_homebrew()?,
    };

    println!("Installing vlc via Homebrew Cask...");
    let status = Command::new(&brew_path)
        .args(["install", "--cask", "vlc"])
        .status()?;

    if status.success() {
        println!("✓ vlc installed successfully.");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to install vlc via Homebrew."))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn install_vlc_windows() -> Result<(), anyhow::Error> {
    println!("Running: winget install -e --id VideoLAN.VLC --accept-source-agreements --accept-package-agreements");
    let status = Command::new("winget")
        .args([
            "install",
            "-e",
            "--id",
            "VideoLAN.VLC",
            "--accept-source-agreements",
            "--accept-package-agreements",
        ])
        .status()?;

    if status.success() {
        println!("✓ vlc installed successfully.");
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to install vlc via winget."))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn is_newer_version(current: &str, tag: &str) -> bool {
    let parse_version =
        |s: &str| -> Vec<u32> { s.split('.').filter_map(|p| p.parse::<u32>().ok()).collect() };

    let cur_parts = parse_version(current);
    let tag_parts = parse_version(tag);

    for i in 0..std::cmp::max(cur_parts.len(), tag_parts.len()) {
        let cur = cur_parts.get(i).unwrap_or(&0);
        let tgt = tag_parts.get(i).unwrap_or(&0);

        if tgt > cur {
            return true;
        } else if tgt < cur {
            return false;
        }
    }
    false
}

/// Returns the path to the update cooldown file (stored next to config)
#[cfg(not(target_arch = "wasm32"))]
fn get_update_cooldown_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("", "", "matrix-iptv")
        .map(|dirs| dirs.config_dir().join(".update_cooldown"))
}

/// Check if an update for the given version was recently dismissed (within cooldown_hours)
#[cfg(not(target_arch = "wasm32"))]
fn is_update_dismissed(version: &str, cooldown_hours: u64) -> bool {
    let path = match get_update_cooldown_path() {
        Some(p) => p,
        None => return false,
    };
    if let Ok(content) = std::fs::read_to_string(&path) {
        // Format: "version|unix_timestamp"
        let parts: Vec<&str> = content.trim().split('|').collect();
        if parts.len() == 2 {
            let dismissed_version = parts[0];
            if let Ok(ts) = parts[1].parse::<u64>() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let elapsed_hours = (now.saturating_sub(ts)) / 3600;
                // If same version was dismissed within cooldown window, skip
                if dismissed_version == version && elapsed_hours < cooldown_hours {
                    return true;
                }
            }
        }
    }
    false
}

/// Record that the user dismissed an update for a specific version
#[cfg(not(target_arch = "wasm32"))]
pub fn dismiss_update(version: &str) {
    if let Some(path) = get_update_cooldown_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = std::fs::write(&path, format!("{}|{}", version, now));
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn check_for_updates(
    tx: tokio::sync::mpsc::Sender<crate::app::AsyncAction>,
    manual: bool,
) {
    let current_version = env!("CARGO_PKG_VERSION");
    let client = reqwest::Client::builder()
        .user_agent("matrix-iptv-cli-updater")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    // Use a GET request to the latest release to check the tag
    // GitHub redirects /latest to the specific tag URL
    if let Ok(resp) = client
        .get("https://github.com/officebeats/matrix-iptv/releases/latest")
        .send()
        .await
    {
        let final_url = resp.url().to_string();
        // URL is likely https://github.com/officebeats/matrix-iptv/releases/tag/v3.0.9
        if let Some(tag) = final_url.split("/tag/").last() {
            let tag = tag.trim_start_matches('v');
            if is_newer_version(current_version, tag) {
                // For automatic (non-manual) checks: skip if user dismissed this version recently
                if !manual && is_update_dismissed(tag, 24) {
                    return; // User dismissed this version within the last 24 hours
                }
                let _ = tx
                    .send(crate::app::AsyncAction::UpdateAvailable(tag.to_string()))
                    .await;
            } else if manual {
                let _ = tx.send(crate::app::AsyncAction::NoUpdateFound).await;
            }
        }
    } else if manual {
        let _ = tx
            .send(crate::app::AsyncAction::Error(
                "Failed to check for updates. Please check your connection.".to_string(),
            ))
            .await;
    }
}

/// On Windows, perform the self-update directly from the Rust binary.
/// This bypasses cli.js entirely (which may be an older version with the EBUSY bug).
/// Writes a PowerShell script that downloads the new binary, replaces the old one, and relaunches.
#[cfg(not(target_arch = "wasm32"))]
pub fn perform_windows_self_update() -> Result<(), anyhow::Error> {
    let current_exe = std::env::current_exe()?;
    let exe_path = current_exe.to_string_lossy().replace('\\', "\\\\");
    let download_url = "https://github.com/officebeats/matrix-iptv/releases/latest/download/matrix-iptv-windows.exe";

    let ps_script = format!(
        r#"
# Matrix IPTV Self-Update Script
$ErrorActionPreference = 'Stop'
Start-Sleep -Seconds 2

$exePath = "{exe_path}"
$tempPath = "$exePath.update.tmp"
$backupPath = "$exePath.old.$([DateTimeOffset]::Now.ToUnixTimeMilliseconds())"

try {{
    # Download new binary
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $webClient = New-Object System.Net.WebClient
    $webClient.Headers.Add('User-Agent', 'matrix-iptv-cli-updater')
    $webClient.DownloadFile("{download_url}", $tempPath)
    
    # Replace binary with retries
    $maxAttempts = 15
    for ($i = 0; $i -lt $maxAttempts; $i++) {{
        try {{
            if (Test-Path $exePath) {{
                Move-Item -Path $exePath -Destination $backupPath -Force
                try {{ Remove-Item $backupPath -Force -ErrorAction SilentlyContinue }} catch {{}}
            }}
            Move-Item -Path $tempPath -Destination $exePath -Force
            break
        }} catch {{
            if ($i -eq ($maxAttempts - 1)) {{ throw }}
            Start-Sleep -Seconds (1 + $i * 0.5)
        }}
    }}
    
    # Relaunch with retries
    Start-Sleep -Seconds 1
    $maxSpawn = 5
    for ($j = 0; $j -lt $maxSpawn; $j++) {{
        try {{
            Start-Process -FilePath $exePath -WindowStyle Normal
            break
        }} catch {{
            if ($j -eq ($maxSpawn - 1)) {{ throw }}
            Start-Sleep -Seconds (1 + $j * 0.5)
        }}
    }}
}} catch {{
    Write-Host "`n[!] Update failed: $_" -ForegroundColor Red
    if (Test-Path $tempPath) {{ Remove-Item $tempPath -Force -ErrorAction SilentlyContinue }}
    Write-Host "Press any key to exit..."
    $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')
}}

# Clean up this script
Remove-Item -Path $MyInvocation.MyCommand.Source -Force -ErrorAction SilentlyContinue
"#
    );

    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("matrix-iptv-update.ps1");
    std::fs::write(&script_path, ps_script)?;

    Command::new("powershell.exe")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-File",
            &script_path.to_string_lossy(),
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    Ok(())
}
