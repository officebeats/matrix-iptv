# Check for wasm-pack
if (Get-Command "wasm-pack" -ErrorAction SilentlyContinue) {
    Write-Host "Building for Web via wasm-pack..."
    wasm-pack build --target web
    if ($?) {
        Write-Host "Build complete!"
        Write-Host "Now start a server in this directory, for example:"
        Write-Host "  python -m http.server 8000"
        Write-Host "Then open http://localhost:8000 in Chrome."
    } else {
        Write-Host "Build failed."
    }
} else {
    Write-Host "Error: 'wasm-pack' is not installed."
    Write-Host "Please install it by running:"
    Write-Host "  cargo install wasm-pack"
    Write-Host "Or visit https://rustwasm.github.io/wasm-pack/installer/"
}
