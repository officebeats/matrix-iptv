# Quick Series Test
# This verifies that Series categories and streams load correctly

Write-Host "=== Testing Series Functionality ===" -ForegroundColor Cyan
Write-Host ""

# Run the QA bot which tests Series
Write-Host "Running comprehensive Series test..." -ForegroundColor Yellow
cargo run --release --bin qa_bot 2>&1 | Select-String "Series|Account|categories|streams|PASS|FAIL" | ForEach-Object {
    if ($_ -match "PASS") {
        Write-Host $_ -ForegroundColor Green
    } elseif ($_ -match "FAIL") {
        Write-Host $_ -ForegroundColor Red
    } else {
        Write-Host $_
    }
}

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next: Launch the app and manually verify:" -ForegroundColor White
Write-Host "  1. Select a playlist (Trex or Strong8K)" -ForegroundColor Gray
Write-Host "  2. Choose 'SERIES (VOD) [White Rabbit]'" -ForegroundColor Gray
Write-Host "  3. Verify categories appear in left pane" -ForegroundColor Gray
Write-Host "  4. Select a category and verify streams load" -ForegroundColor Gray
