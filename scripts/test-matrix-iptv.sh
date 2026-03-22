#!/bin/bash
# test-matrix-iptv - Build and run Matrix IPTV interactively from anywhere

# Kill any existing runs
pkill -f "matrix-iptv" 2>/dev/null || true
pkill -f "matrix-test" 2>/dev/null || true
sleep 1

PROJECT_DIR="/tmp/matrix-iptv"

# Build the app
cd "$PROJECT_DIR"
cargo build 2>&1 | tail -3

# Copy binary to home for easy access
cp "$PROJECT_DIR/target/debug/matrix-iptv" ~/matrix-iptv-bin
chmod +x ~/matrix-iptv-bin

# Run interactively
exec ~/matrix-iptv-bin