# Matrix IPTV - Ultra-Safe Installer for Windows
$ErrorActionPreference = "Stop"

try {
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan
    Write-Host "🟢  MATRIX IPTV SYSTEM INSTALLATION" -ForegroundColor Green
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan

    # 1. Check for MPV
    $hasMpv = Get-Command mpv -ErrorAction SilentlyContinue
    if (-not $hasMpv) {
        Write-Host "[*] MPV Player not found. Attempting install..." -ForegroundColor Yellow
        # Try winget but don't crash if it fails
        try { winget install info.mpv.mpv --accept-source-agreements --accept-package-agreements } catch { 
            Write-Host "[!] Auto-install failed. Please install MPV manually from mpv.io" -ForegroundColor Red
        }
    }
    else {
        Write-Host "[+] MPV Player is already installed." -ForegroundColor Green
    }

    # 2. Check for Rust
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "--------------------------------------------------" -ForegroundColor Red
        Write-Host "❌ ERROR: Rust/Cargo not found!" -ForegroundColor Red
        Write-Host "To fix this, please visit: https://rustup.rs/" -ForegroundColor White
        Write-Host "Install Rust, then run this command again." -ForegroundColor White
        Write-Host "--------------------------------------------------" -ForegroundColor Red
        Read-Host "Press Enter to exit"
        return
    }

    # 3. Build Process
    Write-Host "[*] Compiling high-performance engine..." -ForegroundColor Cyan
    cargo build --release --bin matrix-iptv

    # 4. Installation Folders
    $destFolder = "$HOME\.matrix-iptv"
    if (-not (Test-Path $destFolder)) {
        New-Item -ItemType Directory -Path $destFolder | Out-Null
    }

    # 5. Move Binary
    Write-Host "[*] Finalizing system files..." -ForegroundColor Cyan
    Copy-Item "target\release\matrix-iptv.exe" "$destFolder\matrix-iptv.exe" -Force

    # 6. Global Path Support
    $oldPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($oldPath -notlike "*$destFolder*") {
        Write-Host "[*] Enabling 'run-anywhere' capability..." -ForegroundColor Cyan
        $newPath = "$oldPath;$destFolder"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        $env:Path += ";$destFolder"
    }

    Write-Host ""
    Write-Host "✅  SUCCESS: Matrix IPTV is installed!" -ForegroundColor Green
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan
    Write-Host "You can now open a NEW terminal and type: " -NoNewline
    Write-Host "matrix-iptv" -ForegroundColor Green
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan
    Read-Host "Press Enter to finish"

}
catch {
    Write-Host ""
    Write-Host "--------------------------------------------------" -ForegroundColor Red
    Write-Host "❌ CRITICAL ERROR DURING INSTALLATION" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor White
    Write-Host "--------------------------------------------------" -ForegroundColor Red
    Read-Host "Press Enter to exit"
}
