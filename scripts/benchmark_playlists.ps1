# Benchmark IPTV Playlists for Matrix IPTV
$accounts = @(
    @{ name = "Strong 8K"; url = "http://pledge78502.cdn-akm.me:80"; user = "7c34d33c9e21"; pass = "037dacb169" },
    @{ name = "Trex"; url = "http://line.offcial-trex.pro"; user = "3a6aae52fb"; pass = "39c165888139" },
    @{ name = "Strong8k2-PC"; url = "http://zfruvync.duperab.xyz"; user = "PE1S9S8U"; pass = "11EZZUMW" },
    @{ name = "Mega OTT 1"; url = "http://line.4smart.in"; user = "45Z88W6"; pass = "Z7PHTX3" }
)

Write-Host "=== Matrix IPTV Performance Benchmark ===" -ForegroundColor Cyan

foreach ($acc in $accounts) {
    Write-Host "`nProcessing: $($acc.name)" -ForegroundColor Yellow
    
    # 1. Authentication & Category Fetch
    $startTime = Get-Date
    try {
        $baseUrl = $acc.url
        $user = $acc.user
        $pass = $acc.pass
        
        $catUrl = $baseUrl + "/player_api.php?username=" + $user + "&password=" + $pass + "&action=get_live_categories"
        $cats = Invoke-RestMethod -Uri $catUrl -TimeoutSec 30
        $duration = (Get-Date) - $startTime
        Write-Host "  ✅ Categories: $($cats.Count) items in $($duration.TotalSeconds)s" -ForegroundColor Green
        
        # 2. Search for MSNBC
        Write-Host "  🔍 Searching for MSNBC..."
        $searchStart = Get-Date
        $streamsUrl = $baseUrl + "/player_api.php?username=" + $user + "&password=" + $pass + "&action=get_live_streams"
        $streams = Invoke-RestMethod -Uri $streamsUrl -TimeoutSec 120
        $searchDuration = (Get-Date) - $searchStart
        
        $msnbc = $streams | Where-Object { $_.name -like "*MSNBC*" }
        if ($msnbc) {
            Write-Host "  📍 Found $($msnbc.Count) MSNBC streams in $($searchDuration.TotalSeconds)s" -ForegroundColor Cyan
            $msnbc | Select-Object -First 3 | ForEach-Object {
                Write-Host "    - [$($_.stream_id)] $($_.name)"
                $playUrl = $baseUrl + "/live/" + $user + "/" + $pass + "/" + $_.stream_id + ".ts"
                Write-Host "      Link: $playUrl"
            }
        } else {
            Write-Host "  ❌ MSNBC NOT FOUND in this playlist." -ForegroundColor Red
        }
    } catch {
        Write-Host "  ❌ Error: $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host "`n=== Benchmark Complete ===" -ForegroundColor Cyan
