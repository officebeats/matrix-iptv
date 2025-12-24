# Matrix IPTV - Easy Installer for Windows (Fixed)

Write-Host "[*] Initializing Matrix IPTV Installation..." -ForegroundColor Green

# 1. Check for MPV
$hasMpv = Get-Command mpv -ErrorAction SilentlyContinue
if (-not $hasMpv) {
    Write-Host "[!] MPV not found. Attempting to install via winget..." -ForegroundColor Yellow
    winget install info.mpv.mpv
}
else {
    Write-Host "[+] MPV is already installed." -ForegroundColor Green
}

# 2. Check for Rust
$hasCargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $hasCargo) {
    Write-Host "[!] Rust/Cargo not found. Please install it from https://rustup.rs/ first." -ForegroundColor Red
    Write-Host "Press any key to exit..."
    [void]$Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    return
}

# 3. Build the App
Write-Host "[*] Building Matrix IPTV (Core Engine)..." -ForegroundColor Cyan
cargo build --release --bin matrix-iptv

if (-not $?) {
    Write-Host "[!] Build failed. Please check the errors above." -ForegroundColor Red
    Write-Host "Press any key to exit..."
    [void]$Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    return
}

# 4. Create Install Directory
$installDir = "$HOME\.matrix-iptv"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir | Out-Null
}

# 5. Copy Binary
Write-Host "[*] Installing to $installDir..." -ForegroundColor Cyan
Copy-Item "target\release\matrix-iptv.exe" "$installDir\matrix-iptv.exe" -Force

# 6. Add to PATH (Persistent)
Write-Host "[*] Adding to System Path..." -ForegroundColor Cyan
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    $newPath = $userPath + ";" + $installDir
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    # Update current session path
    $env:Path = $env:Path + ";" + $installDir
}

Write-Host ""
Write-Host "[*] SUCCESS: Installation Complete!" -ForegroundColor Green
Write-Host "--------------------------------------------------"
Write-Host "You can now open a NEW terminal and type: matrix-iptv" -ForegroundColor White
Write-Host "--------------------------------------------------"
Write-Host "Press any key to finish..."
[void]$Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
