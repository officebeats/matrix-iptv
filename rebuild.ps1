# Force kill and wait for process to fully terminate
Get-Process -Name "matrix-iptv" -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 1000

# Build
Write-Host "Building..." -ForegroundColor Cyan
cargo build --release --bin matrix-iptv

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful! Starting app..." -ForegroundColor Green
    cargo run --release --bin matrix-iptv
}
else {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
