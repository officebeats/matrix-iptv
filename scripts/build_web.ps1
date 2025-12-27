$ErrorActionPreference = "Stop"

Write-Host "🚧 Building Matrix IPTV PWA (WEB WASM)..." -ForegroundColor Cyan

# 1. Check for wasm-pack
if (-not (Get-Command "wasm-pack" -ErrorAction SilentlyContinue)) {
    Write-Host "❌ wasm-pack is not installed. Please install it first (cargo install wasm-pack)." -ForegroundColor Red
    exit 1
}

# 2. Build for web target
Write-Host "⚙️ Compiling Rust to WebAssembly..." -ForegroundColor Yellow
wasm-pack build --target web --release --out-dir pkg

# 3. Setup Dist folder
$dist = "dist"
if (Test-Path $dist) { Remove-Item $dist -Recurse -Force }
New-Item -ItemType Directory -Path $dist | Out-Null

# 4. Copy Assets
Write-Host "-> Copying assets to $dist..." -ForegroundColor Green
Copy-Item "assets/index.html" -Destination "$dist/index.html" -Force
Copy-Item "assets" -Destination $dist -Recurse -ErrorAction SilentlyContinue 
# Remove the copied assets/index.html inside dist/assets to avoid duplication if desired, 
# but main one is in dist root.
Copy-Item "pkg" -Destination $dist -Recurse

# 5. Success
Write-Host "✅ Build Complete! To test, run: python -m http.server -d dist" -ForegroundColor Green
