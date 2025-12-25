# Matrix IPTV - Instant Binary Installer for Windows

$ErrorActionPreference = "Stop"
$installDir = "$HOME\.matrix-iptv"
$binaryUrl = "https://github.com/officebeats/matrix-iptv/releases/latest/download/matrix-iptv-windows.exe"

try {
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan
    Write-Host "🟢  MATRIX IPTV INSTANT INSTALLER" -ForegroundColor Green
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan

    # 1. Setup Folder
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir | Out-Null
    }

    # 2. Runtime Dependency: MPV
    if (-not (Get-Command mpv -ErrorAction SilentlyContinue)) {
        Write-Host "[*] MPV Player not found (needed for video). Installing..." -ForegroundColor Yellow
        try { 
            winget install info.mpv.mpv --accept-source-agreements --accept-package-agreements 
        }
        catch { 
            Write-Host "[!] Auto-install failed. Please install MPV from mpv.io manually later." -ForegroundColor Red
        }
    }

    # 3. Download Binary (Instant)
    Write-Host "[*] Downloading high-performance binary (Instant)..." -ForegroundColor Cyan
    $destPath = "$installDir\matrix-iptv.exe"
    
    # Try to download. Note: Requires the repo to be PUBLIC or have a release.
    try {
        Invoke-WebRequest -Uri $binaryUrl -OutFile $destPath -Headers @{"Cache-Control" = "no-cache" }
    }
    catch {
        Write-Host "--------------------------------------------------" -ForegroundColor Red
        Write-Host "❌ DOWNLOAD ERROR" -ForegroundColor Red
        Write-Host "The pre-built binary wasn't found. This usually means:" -ForegroundColor White
        Write-Host "1. The GitHub repository is still PRIVATE." -ForegroundColor White
        Write-Host "2. No 'Release' has been created yet on GitHub." -ForegroundColor White
        Write-Host "--------------------------------------------------" -ForegroundColor Red
        Read-Host "Press Enter to exit"
        return
    }

    # 4. Global Path Support
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
    Start-Process $destPath
    
    Write-Host "--------------------------------------------------" -ForegroundColor Cyan
    Write-Host "Usage: Just type 'matrix-iptv' in any future terminal." -ForegroundColor Gray
    Read-Host "Press Enter to finish"

}
catch {
    Write-Host "❌ CRITICAL ERROR: $($_.Exception.Message)" -ForegroundColor Red
    Read-Host "Press Enter to exit"
}
