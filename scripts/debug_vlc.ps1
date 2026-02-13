# VLC Debug Script for Matrix IPTV
# Use this to troubleshoot why VLC is not starting or playing

$vlcPath = "C:\Program Files\VideoLAN\VLC\vlc.exe"
if (!(Test-Path $vlcPath)) {
    $vlcPath = "C:\Program Files (x86)\VideoLAN\VLC\vlc.exe"
}

if (!(Test-Path $vlcPath)) {
    Write-Host "✗ VLC NOT FOUND in standard locations." -ForegroundColor Red
    Write-Host "Please install VLC from https://www.videolan.org/"
    exit
}

Write-Host "✓ VLC Found at: $vlcPath" -ForegroundColor Green
$url = Read-Host "Paste the Stream URL to test (or press Enter for a test pattern)"
if ($url -eq "") {
    $url = "screen://"
}

Write-Host "`nAttempting to launch VLC in HEADLESS mode with IPT-CLI optimizations..." -ForegroundColor Cyan
Write-Host "Command: & `"$vlcPath`" `"$url`" --intf=dummy --dummy-quiet --no-video-title-show --network-caching=3000 --verbose=2 --one-instance --ffmpeg-skiploopfilter=all --hwdec=auto --deinterlace=1 --deinterlace-mode=bob"

# Run VLC and keep console open to see errors
& $vlcPath $url `
   --intf=dummy `
   --dummy-quiet `
   --no-video-title-show `
   --network-caching=3000 `
   --verbose=2 `
   --one-instance `
   --ffmpeg-skiploopfilter=all `
   --hwdec=auto `
   --deinterlace=1 `
   --deinterlace-mode=bob

Write-Host "`nVLC has exited. If you saw no window and no errors, try removing '--intf=dummy' to see the GUI." -ForegroundColor Yellow
