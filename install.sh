#!/bin/bash

# Kael-OS Auto-Installer for Arch Linux
# https://kael-os.dev

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# ASCII Art Banner
print_banner() {
    echo -e "${PURPLE}"
    cat << "EOF"
    ___  __           __     ____  _____
   / _ |/ /_____ ___ / /    / __ \/ ___/
  / __ / / __/ // // /__  / /_/ /\__ \ 
 /_/ |_/_/\__/\_, /____/  \____/___/_/ 
             /___/                      
EOF
    echo -e "${NC}"
    echo -e "${BLUE}Kael-OS Auto-Installer v1.0${NC}"
    echo -e "${BLUE}Local-first AI for Arch Linux${NC}"
    echo ""
}

# Check if running on Arch Linux
check_arch() {
    if [ ! -f /etc/arch-release ]; then
        echo -e "${YELLOW}⚠ Warning: This installer is optimized for Arch Linux${NC}"
        echo -e "${YELLOW}You may need to install dependencies manually on other distributions${NC}"
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# Detect AUR helper
detect_aur_helper() {
    if command -v paru &> /dev/null; then
        echo "paru"
    elif command -v yay &> /dev/null; then
        echo "yay"
    else
        echo "none"
    fi
}

# Install from AUR
install_from_aur() {
    local aur_helper=$1
    
    echo -e "${GREEN}✓ Found AUR helper: $aur_helper${NC}"
    echo -e "${BLUE}Installing Kael-OS from AUR...${NC}"
    
    $aur_helper -S kael-os-ai --noconfirm
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Kael-OS installed successfully!${NC}"
        return 0
    else
        echo -e "${RED}✗ Failed to install from AUR${NC}"
        return 1
    fi
}

# Install from GitHub releases
install_from_github() {
    echo -e "${BLUE}Installing from GitHub releases...${NC}"
    
    # Create temporary directory
    TMP_DIR=$(mktemp -d)
    cd "$TMP_DIR"
    
    # Get latest release URL
    LATEST_URL="https://github.com/leetheorc/Kael-OS-AI/releases/latest/download/kael-os-arch.pkg.tar.zst"
    
    echo -e "${BLUE}Downloading latest release...${NC}"
    curl -L -o kael-os.pkg.tar.zst "$LATEST_URL"
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}✗ Failed to download release${NC}"
        rm -rf "$TMP_DIR"
        return 1
    fi
    
    echo -e "${BLUE}Installing package...${NC}"
    sudo pacman -U kael-os.pkg.tar.zst --noconfirm
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Kael-OS installed successfully!${NC}"
        rm -rf "$TMP_DIR"
        return 0
    else
        echo -e "${RED}✗ Failed to install package${NC}"
        rm -rf "$TMP_DIR"
        return 1
    fi
}

# Check and install Ollama
install_ollama() {
    if command -v ollama &> /dev/null; then
        echo -e "${GREEN}✓ Ollama is already installed${NC}"
        return 0
    fi
    
    echo -e "${YELLOW}Ollama is not installed. Kael-OS requires Ollama for local AI.${NC}"
    read -p "Install Ollama now? (Y/n) " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        echo -e "${BLUE}Installing Ollama...${NC}"
        curl -fsSL https://ollama.com/install.sh | sh
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ Ollama installed successfully${NC}"
            
            # Start Ollama service
            echo -e "${BLUE}Starting Ollama service...${NC}"
            sudo systemctl enable ollama
            sudo systemctl start ollama
            
            # Suggest downloading a model
            echo -e "${YELLOW}Would you like to download a recommended AI model?${NC}"
            echo "1) llama2 (4GB, good for most systems)"
            echo "2) mistral (4GB, fast and efficient)"
            echo "3) Skip for now"
            read -p "Choice (1-3): " -n 1 -r
            echo
            
            case $REPLY in
                1)
                    echo -e "${BLUE}Downloading llama2...${NC}"
                    ollama pull llama2
                    ;;
                2)
                    echo -e "${BLUE}Downloading mistral...${NC}"
                    ollama pull mistral
                    ;;
                *)
                    echo -e "${YELLOW}Skipping model download. You can download models later with 'ollama pull <model>'${NC}"
                    ;;
            esac
        else
            echo -e "${RED}✗ Failed to install Ollama${NC}"
            echo -e "${YELLOW}You can install it manually from https://ollama.com${NC}"
        fi
    fi
}

# Main installation function
main() {
    print_banner
    check_arch
    
    # Detect AUR helper
    AUR_HELPER=$(detect_aur_helper)
    
    if [ "$AUR_HELPER" != "none" ]; then
        echo -e "${GREEN}Installing Kael-OS via AUR...${NC}"
        if install_from_aur "$AUR_HELPER"; then
            install_ollama
            
            echo ""
            echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo -e "${GREEN}✓ Installation complete!${NC}"
            echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo ""
            echo -e "${BLUE}Run Kael-OS with:${NC} kael-os"
            echo -e "${BLUE}Documentation:${NC} https://kael-os.dev/docs.html"
            echo -e "${BLUE}Get help:${NC} https://discord.gg/kael-os"
            echo ""
            exit 0
        fi
    fi
    
    # Fallback to GitHub releases
    echo -e "${YELLOW}No AUR helper found. Installing from GitHub releases...${NC}"
    if install_from_github; then
        install_ollama
        
        echo ""
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}✓ Installation complete!${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
        echo -e "${BLUE}Run Kael-OS with:${NC} kael-os"
        echo -e "${BLUE}Documentation:${NC} https://kael-os.dev/docs.html"
        echo -e "${BLUE}Get help:${NC} https://discord.gg/kael-os"
        echo ""
        exit 0
    else
        echo -e "${RED}✗ Installation failed${NC}"
        echo ""
        echo -e "${YELLOW}Manual installation options:${NC}"
        echo "1. Install an AUR helper (paru or yay)"
        echo "2. Download from https://github.com/leetheorc/Kael-OS-AI/releases"
        echo "3. Build from source: https://kael-os.dev/docs.html#build-from-source"
        exit 1
    fi
}

# Run main function
main
