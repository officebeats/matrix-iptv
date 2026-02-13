# Matrix IPTV - MPV Debug Launch Script
# This script launches MPV with the exact arguments used by the app, but keeps the console visible.

$url = Read-Host "Paste the stream URL to test"
if (-not $url) { 
    Write-Host "Error: No URL provided." -ForegroundColor Red
    exit 
}

$pipe_name = "\\.\pipe\mpv_ipc_debug"
$mpv_path = "mpv" # Assumes it's in PATH, which we verified earlier

Write-Host "Launching MPV with Optimized Settings..." -ForegroundColor Cyan
Write-Host "Check the console window that opens for error messages."

& $mpv_path $url `
   --geometry=1280x720 `
   --force-window `
   --no-fs `
   --osc=yes `
   --video-sync=display-resample `
   --interpolation=yes `
   --tscale=linear `
   --tscale-clamp=0.0 `
   --cache=yes `
   --demuxer-max-bytes=512MiB `
   --demuxer-max-back-bytes=128MiB `
   --demuxer-readahead-secs=60 `
   --stream-buffer-size=2MiB `
   --framedrop=vo `
   --vd-lavc-fast `
   --vd-lavc-skiploopfilter=all `
   --vd-lavc-threads=0 `
   --scale=catmull_rom `
   --cscale=catmull_rom `
   --dscale=catmull_rom `
   --scale-antiring=0.7 `
   --cscale-antiring=0.7 `
   --hwdec=auto-copy `
   --stream-lavf-o=reconnect_at_eof=1,reconnect_streamed=1,reconnect_delay_max=5 `
   --http-reconnect=yes `
   --d3d11-flip=yes `
   --gpu-api=d3d11 `
   --user-agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36" `
   --input-ipc-server=$pipe_name

Write-Host "`nMPV has exited." -ForegroundColor Yellow
pause
