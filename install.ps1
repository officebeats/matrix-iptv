# Matrix IPTV - Zero-Click Universal Installer for Windows

$ErrorActionPreference = "Stop"
$installDir = "$HOME\.matrix-iptv"
$repoUrl = "https://github.com/officebeats/matrix-iptv.git"

try {
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan
    Write-Host "🟢  MATRIX IPTV SYSTEM INSTALLER" -ForegroundColor Green
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan

    # 1. Ensure Install Directory Exists
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir | Out-Null
    }

    # 2. Check for MPV (Video Engine)
    if (-not (Get-Command mpv -ErrorAction SilentlyContinue)) {
        Write-Host "[*] MPV Player not found. Installing system dependency..." -ForegroundColor Yellow
        try { 
            winget install info.mpv.mpv --accept-source-agreements --accept-package-agreements 
            Write-Host "[+] MPV Installed successfully." -ForegroundColor Green
        }
        catch { 
            Write-Host "[!] Auto-install failed. Please install MPV from mpv.io manually later." -ForegroundColor Red
        }
    }

    # 3. Check for Git (Source Control)
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Host "[*] Git not found. Installing dependency..." -ForegroundColor Yellow
        winget install Git.Git --accept-source-agreements
        Write-Host "[!] Git installed. You may need to RESTART your terminal after this finishes." -ForegroundColor Yellow
    }

    # 4. Check for Rust/Cargo (Compiler)
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "[*] Rust Compiler not found. Installing language engine..." -ForegroundColor Yellow
        # Use winget for Rustup (Official)
        winget install Rust.Rustup --accept-source-agreements
        Write-Host "--------------------------------------------------" -ForegroundColor Yellow
        Write-Host "⚠️  ACTION REQUIRED: Rust has been installed." -ForegroundColor White
        Write-Host "Please CLOSE this window and run the install command again" -ForegroundColor White
        Write-Host "to finish the setup (Path needs to refresh)." -ForegroundColor White
        Write-Host "--------------------------------------------------" -ForegroundColor Yellow
        Read-Host "Press Enter to exit"
        exit
    }

    # 5. Fetch Latest Source
    Set-Location $installDir
    if (Test-Path "src-dev") {
        Write-Host "[*] Updating local source code..." -ForegroundColor Cyan
        Set-Location "src-dev"
        git pull
    }
    else {
        Write-Host "[*] Downloading Matrix IPTV system source..." -ForegroundColor Cyan
        git clone $repoUrl "src-dev"
        Set-Location "src-dev"
    }

    # 6. Build High-Performance Engine
    Write-Host "[*] Compiling core engine (this may take a minute)..." -ForegroundColor Cyan
    cargo build --release --bin matrix-iptv

    # 7. Finalize Paths
    $binaryPath = "$installDir\matrix-iptv.exe"
    Copy-Item "target\release\matrix-iptv.exe" $binaryPath -Force

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        Write-Host "[*] Enabling 'run-anywhere' capability..." -ForegroundColor Cyan
        [Environment]::SetEnvironmentVariable("Path", $userPath + ";$installDir", "User")
        $env:Path += ";$installDir"
    }

    Write-Host ""
    Write-Host "✅  SUCCESS: Matrix IPTV is ready!" -ForegroundColor Green
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan
    Write-Host "Launching Matrix IPTV..." -ForegroundColor Yellow
    Start-Process $binaryPath
    
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan
    Write-Host "Usage: Just type 'matrix-iptv' in any future terminal." -ForegroundColor Gray
    Read-Host "Press Enter to close installer"

}
catch {
    Write-Host ""
    Write-Host "--------------------------------------------------" -ForegroundColor Red
    Write-Host "❌ INSTALLER ERROR" -ForegroundColor Red
    Write-Host "Details: $($_.Exception.Message)" -ForegroundColor White
    Write-Host "--------------------------------------------------" -ForegroundColor Red
    Read-Host "Press Enter to close"
}
