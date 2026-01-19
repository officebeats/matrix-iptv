@echo off
setlocal
echo [*] Matrix IPTV // Build Environment: 3.3.2
echo [*] DEBUG: Script: %~f0
echo [*] DEBUG: Workspace: %~dp0

:: Force kill everything to unlock files
taskkill /F /IM matrix-iptv.exe /IM node.exe /IM mpv.exe 2>nul
timeout /t 1 /nobreak >nul

:: Set local target dir to avoid OneDrive sync locks
set CARGO_TARGET_DIR=C:\Users\admin-beats\cargo-target

pushd "%~dp0"

echo [*] Rebuilding binary...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo [!] Build Failed!
    popd
    exit /b 1
)

echo [*] Syncing binary to bin/ folder...
:: Copy from the new target location
copy /Y "C:\Users\admin-beats\cargo-target\release\matrix-iptv.exe" "bin\matrix-iptv.exe" >nul

echo [*] Verifying version before launch:
bin\matrix-iptv.exe --version

echo [*] Launching Node wrapper (with --skip-update)...
node bin\cli.js --skip-update

popd
endlocal
