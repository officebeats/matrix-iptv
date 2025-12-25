# Matrix IPTV Test Command
function test {
    param([string]$app)
    if ($app -eq "matrix-iptv") {
        Stop-Process -Name "matrix-iptv" -Force -ErrorAction SilentlyContinue
        & "c:\Users\admin-beats\Documents\00 Vibe Coding\IPTV\target\release\matrix-iptv.exe"
    }
}
