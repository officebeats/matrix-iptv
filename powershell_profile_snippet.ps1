# Add this to your PowerShell profile to create a 'iptv' command
# Run this once: notepad $PROFILE
# Then paste this function and save

function Start-MatrixIPTV {
    Stop-Process -Name "matrix-iptv" -Force -ErrorAction SilentlyContinue
    & "c:\Users\admin-beats\Documents\00 Vibe Coding\IPTV\target\release\matrix-iptv.exe"
}

Set-Alias iptv Start-MatrixIPTV
