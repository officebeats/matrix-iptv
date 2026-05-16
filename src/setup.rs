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
    check_and_install_dependencies_with_output(false)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn check_and_install_dependencies_verbose() -> Result<(), anyhow::Error> {
    check_and_install_dependencies_with_output(true)
}

#[cfg(not(target_arch = "wasm32"))]
fn check_and_install_dependencies_with_output(verbose: bool) -> Result<(), anyhow::Error> {
    let mut announced = false;
    let mut announce = || -> Result<(), anyhow::Error> {
        if !announced {
            println!("Checking dependencies...");
            announced = true;
        }
        Ok(())
    };

    if verbose {
        announce()?;
    }

    if let Some(mpv_path) = get_mpv_path() {
        if verbose {
            if mpv_path == "mpv" {
                println!("  ✓ mpv found.");
            } else {
                println!("  ✓ mpv found at: {}", mpv_path);
            }
        }
    } else {
        announce()?;
        println!("  ✗ mpv NOT found.");
        if cfg!(target_os = "windows") {
            println!("  Attempting to install mpv using winget...");
            install_mpv_windows()?;
        } else if cfg!(target_os = "macos") {
            println!("  Attempting to install mpv using homebrew...");
            install_mpv_macos()?;
        }
    }

    if let Some(vlc_path) = get_vlc_path() {
        if verbose {
            if vlc_path == "vlc" {
                println!("  ✓ vlc found.");
            } else {
                println!("  ✓ vlc found at: {}", vlc_path);
            }
        }
    } else {
        announce()?;
        println!("  ✗ vlc NOT found.");
        if cfg!(target_os = "windows") {
            println!("  Attempting to install vlc using winget...");
            install_vlc_windows()?;
        } else if cfg!(target_os = "macos") {
            println!("  Attempting to install vlc using homebrew...");
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

#[cfg(not(target_arch = "wasm32"))]
fn platform_update_asset_name() -> Option<&'static str> {
    match std::env::consts::OS {
        "windows" => Some("matrix-iptv-windows.exe"),
        "linux" => Some("matrix-iptv-linux"),
        "macos" => Some("matrix-iptv-macos"),
        _ => None,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn release_tag_name(release: &serde_json::Value) -> Option<String> {
    release
        .get("tag_name")
        .and_then(|tag| tag.as_str())
        .map(|tag| tag.trim_start_matches(['v', 'V']).to_string())
        .filter(|tag| !tag.is_empty())
}

#[cfg(not(target_arch = "wasm32"))]
fn release_has_update_asset(release: &serde_json::Value, asset_name: &str) -> bool {
    release
        .get("assets")
        .and_then(|assets| assets.as_array())
        .map(|assets| {
            assets
                .iter()
                .any(|asset| asset.get("name").and_then(|name| name.as_str()) == Some(asset_name))
        })
        .unwrap_or(false)
}

#[cfg(not(target_arch = "wasm32"))]
fn is_legacy_scoped_npm_binary_path(path: &Path) -> bool {
    let parts: Vec<String> = path
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => {
                Some(value.to_string_lossy().to_ascii_lowercase())
            }
            _ => None,
        })
        .collect();

    parts.ends_with(&[
        "node_modules".to_string(),
        "@officebeats".to_string(),
        "matrix-iptv-cli".to_string(),
        "bin".to_string(),
        "matrix-iptv.exe".to_string(),
    ])
}

/// Returns the path to the update cooldown file (stored next to config)
#[cfg(not(target_arch = "wasm32"))]
fn get_update_cooldown_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("", "", "matrix-iptv")
        .map(|dirs| dirs.config_dir().join(".update_cooldown"))
}

/// Returns the path to the skipped update file (stored next to config).
#[cfg(not(target_arch = "wasm32"))]
fn get_update_skip_path() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("", "", "matrix-iptv")
        .map(|dirs| dirs.config_dir().join(".update_skip"))
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

/// Check if a specific update version was explicitly skipped.
#[cfg(not(target_arch = "wasm32"))]
fn is_update_skipped(version: &str) -> bool {
    let path = match get_update_skip_path() {
        Some(p) => p,
        None => return false,
    };

    std::fs::read_to_string(&path)
        .map(|content| content.trim() == version)
        .unwrap_or(false)
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

/// Record that the user wants to skip a specific update version.
#[cfg(not(target_arch = "wasm32"))]
pub fn skip_update(version: &str) {
    if let Some(path) = get_update_skip_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, version);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn check_for_updates(
    tx: tokio::sync::mpsc::Sender<crate::app::AsyncAction>,
    manual: bool,
) {
    if std::env::var_os("MATRIX_IPTV_SKIP_UPDATE").is_some() {
        return;
    }

    let current_version = env!("CARGO_PKG_VERSION");
    let client = reqwest::Client::builder()
        .user_agent("matrix-iptv-cli-updater")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    if let Ok(resp) = client
        .get("https://api.github.com/repos/officebeats/matrix-iptv/releases/latest")
        .send()
        .await
    {
        let release = resp.json::<serde_json::Value>().await.ok();
        let tag = release.as_ref().and_then(release_tag_name);

        if let Some(tag) = tag {
            if is_newer_version(current_version, &tag) {
                if !manual && (is_update_dismissed(&tag, 24) || is_update_skipped(&tag)) {
                    return;
                }

                let Some(asset_name) = platform_update_asset_name() else {
                    if manual {
                        let _ = tx
                            .send(crate::app::AsyncAction::Error(
                                "Auto-update is not supported on this platform.".to_string(),
                            ))
                            .await;
                    }
                    return;
                };

                if !release
                    .as_ref()
                    .map(|release| release_has_update_asset(release, asset_name))
                    .unwrap_or(false)
                {
                    if manual {
                        let _ = tx
                            .send(crate::app::AsyncAction::Error(format!(
                                "Latest release v{} does not include {} yet. Try again later.",
                                tag, asset_name
                            )))
                            .await;
                    }
                    return;
                }

                let _ = tx.send(crate::app::AsyncAction::UpdateAvailable(tag)).await;
            } else if manual {
                let _ = tx.send(crate::app::AsyncAction::NoUpdateFound).await;
            }
        } else if manual {
            let _ = tx
                .send(crate::app::AsyncAction::Error(
                    "Failed to read the latest GitHub release version.".to_string(),
                ))
                .await;
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
/// This is the fallback for standalone binaries and older wrappers. The npm
/// wrapper remains the preferred path because it can replace the child binary
/// after the app exits, but this helper keeps standalone installs reliable too.
#[cfg(not(target_arch = "wasm32"))]
pub fn perform_windows_self_update() -> Result<(), anyhow::Error> {
    let current_exe = std::env::current_exe()?;
    let exe_path = current_exe.to_string_lossy().replace('\\', "\\\\");
    let legacy_scoped_npm_install = if is_legacy_scoped_npm_binary_path(&current_exe) {
        "$true"
    } else {
        "$false"
    };
    let ps_script_template = r#"
# Matrix IPTV Self-Update Script
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
Start-Sleep -Seconds 2

$exePath = "__EXE_PATH__"
$tempPath = "$exePath.update.tmp"
$backupPath = "$exePath.old.$([DateTimeOffset]::Now.ToUnixTimeMilliseconds())"
$minBinarySize = 102400
$legacyScopedNpmInstall = __LEGACY_SCOPED_NPM_INSTALL__
$legacyScopedNpmRepairSucceeded = $false
$step = 0
$totalSteps = if ($legacyScopedNpmInstall) { 7 } else { 6 }

function Write-Step($message) {
    $script:step += 1
    Write-Host "[$script:step/$script:totalSteps] $message" -ForegroundColor Cyan
}

function Write-Detail($message) {
    Write-Host "    $message" -ForegroundColor DarkGray
}

function Format-Bytes([int64]$bytes) {
    if ($bytes -lt 1024) { return "$bytes B" }
    $units = @('KB', 'MB', 'GB')
    $size = [double]$bytes / 1024
    $unit = $units[0]
    for ($i = 1; $i -lt $units.Length -and $size -ge 1024; $i++) {
        $size = $size / 1024
        $unit = $units[$i]
    }
    if ($size -ge 10) {
        return ("{0:N1} {1}" -f $size, $unit)
    }
    return ("{0:N2} {1}" -f $size, $unit)
}

function Get-MatrixVersion($path) {
    $oldWrapper = $env:MATRIX_IPTV_WRAPPER
    $oldSkipUpdate = $env:MATRIX_IPTV_SKIP_UPDATE
    try {
        $env:MATRIX_IPTV_WRAPPER = '1'
        $env:MATRIX_IPTV_SKIP_UPDATE = '1'
        $output = & $path --version 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Version check failed for ${path}: $output"
        }
    } finally {
        if ($null -eq $oldWrapper) {
            Remove-Item Env:\MATRIX_IPTV_WRAPPER -ErrorAction SilentlyContinue
        } else {
            $env:MATRIX_IPTV_WRAPPER = $oldWrapper
        }

        if ($null -eq $oldSkipUpdate) {
            Remove-Item Env:\MATRIX_IPTV_SKIP_UPDATE -ErrorAction SilentlyContinue
        } else {
            $env:MATRIX_IPTV_SKIP_UPDATE = $oldSkipUpdate
        }
    }

    $match = [regex]::Match(($output -join "`n"), '(\d+\.\d+\.\d+)')
    if (-not $match.Success) {
        throw "Could not read Matrix IPTV version from ${path}"
    }
    return $match.Groups[1].Value
}

function Get-LegacyScopedNpmPrefix($path) {
    try {
        $file = [System.IO.FileInfo]::new($path)
        $bin = $file.Directory
        if ($null -eq $bin) { return $null }

        $pkg = $bin.Parent
        $scope = if ($null -ne $pkg) { $pkg.Parent } else { $null }
        $nodeModules = if ($null -ne $scope) { $scope.Parent } else { $null }
        $prefix = if ($null -ne $nodeModules) { $nodeModules.Parent } else { $null }

        if (
            $bin.Name -ieq 'bin' -and
            $pkg.Name -ieq 'matrix-iptv-cli' -and
            $scope.Name -ieq '@officebeats' -and
            $nodeModules.Name -ieq 'node_modules' -and
            $null -ne $prefix
        ) {
            return $prefix.FullName
        }
    } catch {
        return $null
    }

    return $null
}

function Repair-LegacyScopedNpmInstall($targetVersion) {
    if (-not $legacyScopedNpmInstall) {
        return $false
    }

    Write-Step "Repairing legacy npm wrapper"
    $npm = Get-Command npm.cmd -ErrorAction SilentlyContinue
    if ($null -eq $npm) {
        $npm = Get-Command npm -ErrorAction SilentlyContinue
    }

    if ($null -eq $npm) {
        Write-Detail "npm was not found; binary update succeeded but wrapper repair was skipped."
        return $false
    }

    $packageSpec = "@officebeats/matrix-iptv-cli@$targetVersion"
    Write-Detail "Detected old scoped npm layout. Updating $packageSpec..."
    $output = & $npm.Source install -g $packageSpec --no-audit --no-fund 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Detail "Wrapper repair skipped because npm exited with code $LASTEXITCODE."
        $tail = @($output | Select-Object -Last 4) -join " "
        if ($tail) {
            Write-Detail $tail
        }
        return $false
    }

    Write-Detail "Legacy npm wrapper repaired."
    return $true
}

function Start-MatrixIptv($fallbackPath) {
    $candidates = @()

    if ($legacyScopedNpmRepairSucceeded) {
        $command = Get-Command matrix-iptv.cmd -ErrorAction SilentlyContinue
        if ($null -ne $command) {
            $candidates += $command.Source
        }

        $prefix = Get-LegacyScopedNpmPrefix $fallbackPath
        if ($prefix) {
            $candidates += (Join-Path $prefix 'matrix-iptv.cmd')
        }
    }

    $candidates += $fallbackPath

    foreach ($candidate in ($candidates | Select-Object -Unique)) {
        if ($candidate -and (Test-Path $candidate)) {
            Start-Process -FilePath $candidate -WindowStyle Normal
            return
        }
    }

    throw "Unable to relaunch Matrix IPTV after update."
}

function Download-MatrixAsset($url, $destination, [int64]$expectedBytes) {
    Add-Type -AssemblyName System.Net.Http

    if (Test-Path $destination) {
        Remove-Item $destination -Force
    }

    $client = [System.Net.Http.HttpClient]::new()
    try {
        $client.Timeout = [TimeSpan]::FromSeconds(120)
        $client.DefaultRequestHeaders.UserAgent.ParseAdd('matrix-iptv-cli-updater')
        $client.DefaultRequestHeaders.Add('Cache-Control', 'no-cache')

        $request = [System.Net.Http.HttpRequestMessage]::new([System.Net.Http.HttpMethod]::Get, $url)
        $response = $client.SendAsync($request, [System.Net.Http.HttpCompletionOption]::ResponseHeadersRead).GetAwaiter().GetResult()
        $response.EnsureSuccessStatusCode() | Out-Null

        [int64]$totalBytes = 0
        if ($response.Content.Headers.ContentLength.HasValue) {
            $totalBytes = $response.Content.Headers.ContentLength.Value
        } elseif ($expectedBytes -gt 0) {
            $totalBytes = $expectedBytes
        }

        $inputStream = $response.Content.ReadAsStreamAsync().GetAwaiter().GetResult()
        $outputStream = [System.IO.File]::Open($destination, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write, [System.IO.FileShare]::None)
        try {
            $buffer = New-Object byte[] (1024 * 1024)
            [int64]$downloadedBytes = 0
            $lastPercent = -10
            [int64]$lastLoggedBytes = 0

            while (($read = $inputStream.Read($buffer, 0, $buffer.Length)) -gt 0) {
                $outputStream.Write($buffer, 0, $read)
                $downloadedBytes += $read

                if ($totalBytes -gt 0) {
                    $percent = [Math]::Min(100, [int][Math]::Floor(($downloadedBytes * 100.0) / $totalBytes))
                    if ($percent -ne $lastPercent -and ($percent -ge ($lastPercent + 10) -or $percent -eq 100)) {
                        Write-Detail ("Download {0}% ({1} / {2})" -f $percent, (Format-Bytes $downloadedBytes), (Format-Bytes $totalBytes))
                        $lastPercent = $percent
                    }
                } elseif (($downloadedBytes - $lastLoggedBytes) -ge (5 * 1024 * 1024)) {
                    Write-Detail ("Downloaded {0}..." -f (Format-Bytes $downloadedBytes))
                    $lastLoggedBytes = $downloadedBytes
                }
            }
        } finally {
            $outputStream.Dispose()
            $inputStream.Dispose()
        }
    } finally {
        $client.Dispose()
    }
}

try {
    Write-Host ""
    Write-Host "Matrix IPTV update" -ForegroundColor Green
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $headers = @{
        'Accept' = 'application/vnd.github+json'
        'Cache-Control' = 'no-cache'
        'User-Agent' = 'matrix-iptv-cli-updater'
    }

    Write-Step "Resolving latest GitHub release"
    $release = Invoke-RestMethod -Uri 'https://api.github.com/repos/officebeats/matrix-iptv/releases/latest' -Headers $headers
    $targetVersion = [string]$release.tag_name
    $targetVersion = $targetVersion -replace '^[vV]', ''
    $currentVersion = Get-MatrixVersion $exePath
    Write-Detail "Current: $currentVersion"
    Write-Detail "Target : $targetVersion"

    if ($currentVersion -eq $targetVersion) {
        Write-Host "[+] Matrix IPTV is already up to date." -ForegroundColor Green
        $legacyScopedNpmRepairSucceeded = Repair-LegacyScopedNpmInstall $targetVersion
        Write-Step "Relaunching Matrix IPTV"
        Start-MatrixIptv $exePath
        Write-Host "[+] Matrix IPTV $targetVersion is ready." -ForegroundColor Green
        Remove-Item -Path $MyInvocation.MyCommand.Source -Force -ErrorAction SilentlyContinue
        exit 0
    }

    $asset = $release.assets | Where-Object { $_.name -eq 'matrix-iptv-windows.exe' } | Select-Object -First 1
    if (-not $asset) {
        throw "Latest release does not include matrix-iptv-windows.exe"
    }

    Write-Step "Downloading $($asset.name) from $($release.tag_name)"
    Download-MatrixAsset $asset.browser_download_url $tempPath $asset.size

    Write-Step "Verifying downloaded binary"
    $downloaded = Get-Item $tempPath
    if ($downloaded.Length -lt $minBinarySize) {
        throw "Downloaded file is too small: $($downloaded.Length) bytes"
    }
    if ($asset.size -and $downloaded.Length -ne $asset.size) {
        throw "Downloaded file size mismatch: expected $($asset.size), got $($downloaded.Length)"
    }
    Write-Detail ("File size verified ({0})." -f (Format-Bytes $downloaded.Length))

    if ($asset.digest -and $asset.digest.StartsWith('sha256:')) {
        $expectedHash = $asset.digest.Substring(7).ToLowerInvariant()
        $actualHash = (Get-FileHash -Algorithm SHA256 -Path $tempPath).Hash.ToLowerInvariant()
        if ($actualHash -ne $expectedHash) {
            throw "Downloaded binary checksum did not match the GitHub release asset digest"
        }
        Write-Detail "Checksum verified."
    } else {
        Write-Detail "GitHub did not provide a release asset digest; using size and version checks."
    }

    $downloadedVersion = Get-MatrixVersion $tempPath
    if ($downloadedVersion -ne $targetVersion) {
        throw "Downloaded binary reports version $downloadedVersion, expected $targetVersion"
    }
    Write-Detail "Downloaded binary reports version $downloadedVersion."
    
    Write-Step "Installing update"
    $maxAttempts = 15
    for ($i = 0; $i -lt $maxAttempts; $i++) {
        try {
            if ((Test-Path $exePath) -and -not (Test-Path $backupPath)) {
                Move-Item -Path $exePath -Destination $backupPath -Force
            }
            Move-Item -Path $tempPath -Destination $exePath -Force
            break
        } catch {
            if ($i -eq ($maxAttempts - 1)) { throw }
            $delay = 1 + $i * 0.5
            Write-Detail "Executable is still locked. Retrying in $delay seconds..."
            Start-Sleep -Seconds $delay
        }
    }

    Write-Step "Verifying installed binary"
    $installedVersion = Get-MatrixVersion $exePath
    if ($installedVersion -ne $targetVersion) {
        throw "Installed binary reports version $installedVersion, expected $targetVersion"
    }
    Write-Detail "Installed binary reports version $installedVersion."

    if (Test-Path $backupPath) {
        Remove-Item $backupPath -Force -ErrorAction SilentlyContinue
    }

    $legacyScopedNpmRepairSucceeded = Repair-LegacyScopedNpmInstall $targetVersion
    
    Start-Sleep -Seconds 1
    Write-Step "Relaunching Matrix IPTV"
    $maxSpawn = 5
    for ($j = 0; $j -lt $maxSpawn; $j++) {
        try {
            Start-MatrixIptv $exePath
            break
        } catch {
            if ($j -eq ($maxSpawn - 1)) { throw }
            $delay = 1 + $j * 0.5
            Write-Detail "Launch was blocked. Retrying in $delay seconds..."
            Start-Sleep -Seconds $delay
        }
    }
    Write-Host "[+] Matrix IPTV $targetVersion is ready." -ForegroundColor Green
} catch {
    Write-Host "`n[!] Update failed: $_" -ForegroundColor Red
    if (Test-Path $tempPath) { Remove-Item $tempPath -Force -ErrorAction SilentlyContinue }
    if (Test-Path $backupPath) {
        if (Test-Path $exePath) { Remove-Item $exePath -Force -ErrorAction SilentlyContinue }
        Move-Item -Path $backupPath -Destination $exePath -Force
    }
    Write-Host "Press any key to exit..."
    $null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')
}

# Clean up this script
Remove-Item -Path $MyInvocation.MyCommand.Source -Force -ErrorAction SilentlyContinue
"#;
    let ps_script = ps_script_template
        .replace("__EXE_PATH__", &exe_path)
        .replace("__LEGACY_SCOPED_NPM_INSTALL__", legacy_scoped_npm_install);

    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join(format!("matrix-iptv-update-{}.ps1", std::process::id()));
    std::fs::write(&script_path, ps_script)?;

    Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script_path.to_string_lossy(),
        ])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn release_tag_name_strips_v_prefix() {
        let release = json!({ "tag_name": "v4.3.12" });

        assert_eq!(release_tag_name(&release).as_deref(), Some("4.3.12"));
    }

    #[test]
    fn release_has_update_asset_matches_platform_asset_name() {
        let release = json!({
            "assets": [
                { "name": "matrix-iptv-linux" },
                { "name": "matrix-iptv-windows.exe" }
            ]
        });

        assert!(release_has_update_asset(
            &release,
            "matrix-iptv-windows.exe"
        ));
        assert!(!release_has_update_asset(&release, "matrix-iptv-macos"));
    }

    #[test]
    fn release_has_update_asset_handles_missing_assets() {
        let release = json!({ "tag_name": "v4.3.12" });

        assert!(!release_has_update_asset(
            &release,
            "matrix-iptv-windows.exe"
        ));
    }

    #[test]
    fn legacy_scoped_npm_binary_path_matches_old_direct_package_layout() {
        let path = Path::new(
            r"C:\Users\beats\AppData\Roaming\npm\node_modules\@officebeats\matrix-iptv-cli\bin\matrix-iptv.exe",
        );

        assert!(is_legacy_scoped_npm_binary_path(path));
    }

    #[test]
    fn legacy_scoped_npm_binary_path_ignores_new_nested_dependency_layout() {
        let path = Path::new(
            r"C:\Users\beats\AppData\Roaming\npm\node_modules\@officebeats\matrix-iptv-cli\node_modules\matrix-iptv\bin\matrix-iptv.exe",
        );

        assert!(!is_legacy_scoped_npm_binary_path(path));
    }
}
