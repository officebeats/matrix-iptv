# Matrix IPTV - Easy Installer for Windows

Write-Host "🟢 Initializing Matrix IPTV Installation..." -ForegroundColor Green

# 1. Check for Prerequisites
$hasMpv = Get-Command mpv -ErrorAction SilentlyContinue
if (-not $hasMpv) {
    Write-Host "🟡 MPV not found. Installing via winget..." -ForegroundColor Yellow
    winget install info.mpv.mpv
}
else {
    Write-Host "✅ MPV is already installed." -ForegroundColor Green
}

$hasCargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $hasCargo) {
    Write-Host "❌ Rust/Cargo not found. Please install it from https://rustup.rs/ first." -ForegroundColor Red
    exit
}

# 2. Build the App
Write-Host "🚀 Building Matrix IPTV (Core Engine)..." -ForegroundColor Cyan
cargo build --release --bin matrix-iptv

if (-not $?) {
    Write-Host "❌ Build failed. Please check the errors above." -ForegroundColor Red
    exit
}

# 3. Create Install Directory
$installDir = "$HOME\.matrix-iptv"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir | Out-Null
}

# 4. Copy Binary
Write-Host "📦 Installing to $installDir..." -ForegroundColor Cyan
Copy-Item "target\release\matrix-iptv.exe" "$installDir\matrix-iptv.exe" -Force

# 5. Add to PATH (Persistent)
Write-Host "🌐 Adding to System Path (so you can run 'matrix-iptv' anywhere)..." -ForegroundColor Cyan
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$installDir", "User")
    $env:Path = [Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [Environment]::GetEnvironmentVariable("Path", "User")
}

Write-Host ""
Write-Host "🎉 INSTALLATION COMPLETE!" -ForegroundColor Green
Write-Host "--------------------------------------------------"
Write-Host "You can now open a NEW terminal and simply type:" -ForegroundColor White
Write-Host "matrix-iptv" -ForegroundColor Green -NoNewline
Write-Host " to launch the system." -ForegroundColor White
Write-Host "--------------------------------------------------"
Write-Host "Press any key to finish..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
