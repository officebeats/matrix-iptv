#!/bin/bash

# Matrix IPTV - Instant Binary Installer (Mac, Linux)
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}[*] Initializing Matrix IPTV Instant Installation...${NC}"

# 1. Detect OS
OS="$(uname)"
BINARY_NAME="matrix-iptv-linux"
if [[ "$OS" == "Darwin" ]]; then
    BINARY_NAME="matrix-iptv-macos"
    # Ensure Homebrew and MPV
    if ! command -v mpv &> /dev/null; then
        echo -e "${YELLOW}[!] MPV Player not found. Attempting install via Homebrew...${NC}"
        if ! command -v brew &> /dev/null; then
             /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)" || true
        fi
        brew install mpv || true
    fi
fi

# 2. Setup Folder
INSTALL_DIR="$HOME/.matrix-iptv"
mkdir -p "$INSTALL_DIR"
BINARY_URL="https://github.com/officebeats/matrix-iptv/releases/latest/download/$BINARY_NAME"

# 3. Download Binary
echo -e "${CYAN}[*] Downloading pre-built binary for $OS (Instant)...${NC}"
if ! curl -L -o "$INSTALL_DIR/matrix-iptv" "$BINARY_URL"; then
    echo -e "${RED}--------------------------------------------------${NC}"
    echo -e "${RED}❌ DOWNLOAD ERROR${NC}"
    echo -e "The pre-built binary wasn't found. This usually means:"
    echo -e "1. The GitHub repository is still PRIVATE."
    echo -e "2. No 'Release' has been created yet on GitHub."
    echo -e "${RED}--------------------------------------------------${NC}"
    exit 1
fi

chmod +x "$INSTALL_DIR/matrix-iptv"

# 4. Update PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    SHELL_CONFIG=""
    if [[ "$SHELL" == */zsh ]]; then SHELL_CONFIG="$HOME/.zshrc"
    elif [[ "$SHELL" == */bash ]]; then SHELL_CONFIG="$HOME/.bashrc"
    else SHELL_CONFIG="$HOME/.profile"; fi
    
    echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_CONFIG"
    echo -e "${GREEN}[+] Added to $SHELL_CONFIG.${NC}"
fi

echo -e "\n${GREEN}[*] SUCCESS: Matrix IPTV is ready!${NC}"
echo "--------------------------------------------------"
echo -e "Launching Matrix IPTV..."
"$INSTALL_DIR/matrix-iptv"
echo "--------------------------------------------------"
