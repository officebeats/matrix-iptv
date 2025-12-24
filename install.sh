#!/bin/bash

# Matrix IPTV - Zero-Click Universal Installer (Mac, Linux)
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}[*] Initializing Matrix IPTV Installation...${NC}"

# 1. Dependency: Homebrew (Mac only)
OS="$(uname)"
if [[ "$OS" == "Darwin" ]]; then
    if ! command -v brew &> /dev/null; then
        echo -e "${YELLOW}[!] Homebrew not found. It's required for Mac dependencies. Installing...${NC}"
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)" || true
    fi
fi

# 2. Dependency: Git
if ! command -v git &> /dev/null; then
    echo -e "${YELLOW}[!] Git not found. Installing...${NC}"
    if [[ "$OS" == "Darwin" ]]; then brew install git;
    elif command -v apt-get &> /dev/null; then sudo apt-get update && sudo apt-get install -y git;
    fi
fi

# 3. Dependency: MPV
if ! command -v mpv &> /dev/null; then
    echo -e "${YELLOW}[!] MPV Player not found. Installing...${NC}"
    if [[ "$OS" == "Darwin" ]]; then brew install mpv;
    elif command -v apt-get &> /dev/null; then sudo apt-get install -y mpv;
    fi
fi

# 4. Dependency: Rust
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}[!] Rust Compiler not found. Installing...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# 5. Setup Workspace
INSTALL_DIR="$HOME/.matrix-iptv"
mkdir -p "$INSTALL_DIR"
REPO_URL="https://github.com/officebeats/matrix-iptv.git"

if [ ! -d "$INSTALL_DIR/src-dev/.git" ]; then
    echo -e "${CYAN}[*] Downloading Matrix IPTV system source...${NC}"
    git clone "$REPO_URL" "$INSTALL_DIR/src-dev"
else
    echo -e "${CYAN}[*] Updating system source...${NC}"
    cd "$INSTALL_DIR/src-dev" && git pull
fi

# 6. Build
cd "$INSTALL_DIR/src-dev"
echo -e "${CYAN}[*] Compiling high-performance engine...${NC}"
cargo build --release --bin matrix-iptv

# 7. Finalize
BINARY_DEST="$INSTALL_DIR/matrix-iptv"
cp target/release/matrix-iptv "$BINARY_DEST"
chmod +x "$BINARY_DEST"

# Update PATH
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
"$BINARY_DEST"
echo "--------------------------------------------------"
