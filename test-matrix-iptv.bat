@echo off
setlocal
echo [*] Matrix IPTV // Building Latest Source...
taskkill /F /IM matrix-iptv.exe 2>nul

:: Navigate to the directory where this script is located
pushd "%~dp0"

:: 1. Rebuild the binary - using the bin flag to be explicit
cargo build --release --bin matrix-iptv

if %ERRORLEVEL% NEQ 0 (
    echo [!] Build Failed!
    popd
    exit /b %ERRORLEVEL%
)

:: 2. Ensure the bin directory exists
if not exist "bin" mkdir "bin"

:: 3. Copy the fresh binary to the bin folder for the wrapper
copy /y target\release\matrix-iptv.exe bin\matrix-iptv.exe >nul

:: 4. Launch via the Intelligent Wrapper (Node.js)
:: This enables the Auto-Update logic to function correctly
node bin\cli.js

popd
endlocal
