#!/bin/bash
# Matrix IPTV Test Runner
# Usage: matrix-test [command]
# Commands: run, build, clean, diagnose

set -e

PROJECT_DIR="/tmp/matrix-iptv"
BINARY_NAME="matrix-test"

# Auto-detect home directory
HOME_DIR="${HOME:-$(eval echo ~$(whoami))}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

command="$1"
shift || true

cd "$PROJECT_DIR" 2>/dev/null || { echo -e "${RED}Error: Project not found at $PROJECT_DIR${NC}"; exit 1; }

case "$command" in
    run)
        echo -e "${GREEN}Building and running Matrix IPTV Test...${NC}"
        cargo run --bin "$BINARY_NAME" "$@"
        ;;
    build)
        echo -e "${GREEN}Building Matrix IPTV Test...${NC}"
        cargo build --bin "$BINARY_NAME" "$@"
        echo -e "${GREEN}Build complete!${NC}"
        echo "Binary location: $PROJECT_DIR/target/debug/$BINARY_NAME"
        ;;
    clean)
        echo -e "${YELLOW}Cleaning build artifacts...${NC}"
        cargo clean
        rm -f "$PROJECT_DIR/mpv_playback.log" 2>/dev/null || true
        rm -f "$PROJECT_DIR/vlc_playback.log" 2>/dev/null || true
        echo -e "${GREEN}Clean complete!${NC}"
        ;;
    diagnose)
        echo -e "${GREEN}Running full diagnostic...${NC}"
        echo ""
        echo "=== System Info ==="
        echo "OS: $(uname -s)"
        echo "MPV: $(which mpv 2>/dev/null || echo 'NOT FOUND')"
        echo "VLC: $(which vlc 2>/dev/null || echo 'NOT FOUND')"
        echo ""
        
        if [ -f "$PROJECT_DIR/mpv_playback.log" ]; then
            echo "=== Recent MPV Log ==="
            tail -20 "$PROJECT_DIR/mpv_playback.log"
        fi
        
        echo ""
        echo "=== Running Test ==="
        cargo run --bin "$BINARY_NAME" "$@"
        ;;
    *)
        echo -e "${GREEN}Matrix IPTV Test Runner${NC}"
        echo ""
        echo "Usage: matrix-test <command>"
        echo ""
        echo "Commands:"
        echo "  run       - Build and run the test (default)"
        echo "  build     - Build the test binary"
        echo "  clean     - Clean build artifacts"
        echo "  diagnose  - Show system info + run test"
        echo ""
        echo "Examples:"
        echo "  matrix-test run"
        echo "  matrix-test diagnose"
        echo "  matrix-test build --release"
        echo "  matrix-test clean"
        ;;
esac