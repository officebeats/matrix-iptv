#!/bin/bash

# Matrix IPTV - Instant Binary Installer (Mac, Linux)
set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}[*] Initializing Matrix IPTV Instant Installation (v1.0.3)...${NC}"

# Helper: Detect and export Homebrew paths (macOS)
# This ensures mpv and other Homebrew binaries are discoverable
detect_and_export_homebrew_paths() {
    local BREW_PREFIXES=()
    
    # Apple Silicon Homebrew prefix
    if [ -d "/opt/homebrew" ]; then
        BREW_PREFIXES+=("/opt/homebrew")
    fi
    
    # Intel Mac / traditional Homebrew prefix
    if [ -d "/usr/local/Homebrew" ] || [ -d "/usr/local/bin/brew" ] || [ -x "/usr/local/bin/brew" ]; then
        BREW_PREFIXES+=("/usr/local")
    fi
    
    # Add each prefix's bin and sbin to PATH if not already present
    for prefix in "${BREW_PREFIXES[@]}"; do
        for dir in "$prefix/bin" "$prefix/sbin"; do
            if [ -d "$dir" ] && [[ ":$PATH:" != *":$dir:"* ]]; then
                export PATH="$dir:$PATH"
                echo -e "${CYAN}[*] Added $dir to PATH${NC}"
            fi
        done
    done
}

# 1. Detect OS
OS="$(uname)"
BINARY_NAME="matrix-iptv-linux"
if [[ "$OS" == "Darwin" ]]; then
    BINARY_NAME="matrix-iptv-macos"
    
    # Detect and add Homebrew paths BEFORE checking for mpv
    echo -e "${CYAN}[*] Detecting Homebrew paths...${NC}"
    detect_and_export_homebrew_paths
    
    # Ensure Homebrew and MPV
    if ! command -v mpv &> /dev/null; then
        echo -e "${YELLOW}[!] MPV Player not found in PATH. Attempting install via Homebrew...${NC}"
        if ! command -v brew &> /dev/null; then
            echo -e "${YELLOW}[!] Homebrew not found. Installing Homebrew first...${NC}"
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)" || true
            # Re-detect paths after Homebrew install
            detect_and_export_homebrew_paths
        fi
        brew install mpv || true
        # Re-detect paths after mpv install
        detect_and_export_homebrew_paths
    else
        echo -e "${GREEN}[✓] mpv found at: $(command -v mpv)${NC}"
    fi
fi

# 2. Setup Folder
INSTALL_DIR="$HOME/.matrix-iptv"
mkdir -p "$INSTALL_DIR"
BINARY_URL="https://github.com/officebeats/matrix-iptv/releases/latest/download/$BINARY_NAME"

# 3. Download Binary
echo -e "${CYAN}[*] Downloading pre-built binary for $OS (Instant)...${NC}"
# Use -f to catch 404s
if ! curl -L -f -o "$INSTALL_DIR/matrix-iptv" "$BINARY_URL"; then
    echo -e "${RED}--------------------------------------------------${NC}"
    echo -e "${RED}❌ DOWNLOAD ERROR${NC}"
    echo -e "The pre-built binary wasn't found at: $BINARY_URL"
    echo -e "This usually means the GitHub Release (v1.0.0) is still building."
    echo -e "Please wait 1-2 minutes and try again."
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
    
    # Don't add if already there
    if ! grep -q "$INSTALL_DIR" "$SHELL_CONFIG" 2>/dev/null; then
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_CONFIG"
        echo -e "${GREEN}[+] Added to $SHELL_CONFIG.${NC}"
    fi
fi

echo -e "\n${GREEN}[*] SUCCESS: Matrix IPTV is ready!${NC}"
echo "--------------------------------------------------"
if [[ "$OS" == "Darwin" ]]; then
    echo -e "${CYAN}Tip: If Matrix IPTV cannot find mpv later, ensure your shell PATH includes Homebrew:${NC}"
    echo -e "  Apple Silicon: ${YELLOW}export PATH=\"/opt/homebrew/bin:\$PATH\"${NC}"
    echo -e "  Intel Mac:     ${YELLOW}export PATH=\"/usr/local/bin:\$PATH\"${NC}"
    echo -e "(Add the above line to your ~/.zshrc or ~/.bash_profile)"
    echo "--------------------------------------------------"
fi
echo -e "Launching Matrix IPTV (Press Ctrl+C to exit)..."
echo "--------------------------------------------------"

# Run the app
"$INSTALL_DIR/matrix-iptv"
