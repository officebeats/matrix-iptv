#!/bin/bash

# Matrix IPTV - Universal Installer (Mac, Linux, Windows-Bash)
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}[*] Initializing Matrix IPTV Installation...${NC}"

# 1. Determine OS
OS="$(uname)"
IS_WINDOWS=false
case "$OS" in
    "Darwin")
        echo -e "${CYAN}[*] System identified as macOS.${NC}"
        if ! command -v brew &> /dev/null; then
            echo -e "${YELLOW}[!] Homebrew not found. It's recommended for macOS.${NC}"
        fi
        if ! command -v mpv &> /dev/null; then
            echo -e "${YELLOW}[!] MPV not found. Installing via brew...${NC}"
            if command -v brew &> /dev/null; then brew install mpv; fi
        fi
        ;;
    "Linux")
        echo -e "${CYAN}[*] System identified as Linux.${NC}"
        if ! command -v mpv &> /dev/null; then
            echo -e "${YELLOW}[!] MPV not found. Please install it (e.g., sudo apt install mpv).${NC}"
        fi
        ;;
    *"MINGW"*|*"MSYS"*|*"CYGWIN"*)
        echo -e "${CYAN}[*] System identified as Windows (Bash environment).${NC}"
        IS_WINDOWS=true
        ;;
    *)
        echo -e "${RED}[!] Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

# 2. Check for Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}[!] Rust/Cargo not found. Please install it from https://rustup.rs/ first.${NC}"
    exit 1
fi

# 3. Build the App
echo -e "${CYAN}[*] Building Matrix IPTV (Core Engine)...${NC}"
cargo build --release --bin matrix-iptv

# 4. Install
INSTALL_DIR="$HOME/.matrix-iptv"
mkdir -p "$INSTALL_DIR"

BINARY_PATH=""
if [ "$IS_WINDOWS" = true ]; then
    echo -e "${CYAN}[*] Installing to $INSTALL_DIR (Windows)...${NC}"
    BINARY_PATH="$INSTALL_DIR/matrix-iptv.exe"
    cp target/release/matrix-iptv.exe "$BINARY_PATH"
    powershell.exe -Command "[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';$HOME\.matrix-iptv', 'User')"
    echo -e "${GREEN}[+] Added to Windows User Path.${NC}"
else
    echo -e "${CYAN}[*] Installing to $INSTALL_DIR...${NC}"
    BINARY_PATH="$INSTALL_DIR/matrix-iptv"
    cp target/release/matrix-iptv "$BINARY_PATH"
    chmod +x "$BINARY_PATH"

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        SHELL_CONFIG=""
        if [[ "$SHELL" == */zsh ]]; then SHELL_CONFIG="$HOME/.zshrc";
        elif [[ "$SHELL" == */bash ]]; then SHELL_CONFIG="$HOME/.bashrc"; fi
        
        if [ -n "$SHELL_CONFIG" ]; then
            echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_CONFIG"
            echo -e "${GREEN}[+] Added to $SHELL_CONFIG.${NC}"
        fi
    fi
fi

echo -e "\n${GREEN}[*] SUCCESS: Installation Complete!${NC}"
echo "--------------------------------------------------"
echo -e "Launching Matrix IPTV for the first time..."
echo "--------------------------------------------------"

# Launch the app
if [ "$IS_WINDOWS" = true ]; then
    start "$BINARY_PATH"
else
    "$BINARY_PATH"
fi
