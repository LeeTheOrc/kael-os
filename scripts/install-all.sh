#!/bin/bash

# Kael-OS + Ollama Complete Installation Script
# This script installs both Kael-OS and Ollama with all dependencies

set -e

VERSION="0.3.0"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_step() {
    echo -e "${YELLOW}➜${NC} $1"
}

# Check if running on supported OS
check_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        print_error "Unsupported OS: $OSTYPE"
        exit 1
    fi
}

# Detect Linux distro
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif [ -f /etc/debian_version ]; then
        echo "debian"
    elif [ -f /etc/fedora-release ]; then
        echo "fedora"
    else
        echo "unknown"
    fi
}

# Install system dependencies
install_system_deps() {
    print_header "📦 Installing System Dependencies"
    
    OS=$(check_os)
    
    if [ "$OS" = "linux" ]; then
        DISTRO=$(detect_distro)
        
        case $DISTRO in
            ubuntu|debian)
                print_step "Debian/Ubuntu detected"
                sudo apt-get update
                sudo apt-get install -y \
                    libssl-dev \
                    libgtk-3-dev \
                    libayatana-appindicator3-dev \
                    librsvg2-dev \
                    libwebkit2gtk-4.1-dev \
                    curl \
                    build-essential \
                    git
                print_success "System dependencies installed"
                ;;
            fedora)
                print_step "Fedora detected"
                sudo dnf install -y \
                    openssl-devel \
                    gtk3-devel \
                    appindicator-gtk3-devel \
                    librsvg2-devel \
                    webkit2gtk3-devel \
                    curl \
                    gcc \
                    make \
                    git
                print_success "System dependencies installed"
                ;;
            arch|manjaro)
                print_step "Arch Linux detected"
                sudo pacman -Syu --noconfirm
                sudo pacman -S --noconfirm \
                    openssl \
                    gtk3 \
                    libappindicator-gtk3 \
                    librsvg \
                    webkit2gtk \
                    base-devel \
                    curl \
                    git
                print_success "System dependencies installed"
                ;;
            *)
                print_warning "Unknown distro: $DISTRO"
                print_step "Please install system dependencies manually:"
                echo "  - libssl-dev"
                echo "  - libgtk-3-dev"
                echo "  - librsvg2-dev"
                echo "  - curl"
                ;;
        esac
    elif [ "$OS" = "macos" ]; then
        print_step "macOS detected"
        if ! command -v brew &> /dev/null; then
            print_error "Homebrew not installed"
            print_step "Install from: https://brew.sh"
            exit 1
        fi
        brew install openssl curl git
        print_success "System dependencies installed"
    fi
}

# Install/verify Ollama
install_ollama() {
    print_header "🤖 Installing Ollama"
    
    if command -v ollama &> /dev/null; then
        print_success "Ollama already installed"
        if systemctl --user is-active ollama.service &> /dev/null 2>&1 || \
           systemctl is-active ollama.service &> /dev/null 2>&1; then
            print_success "Ollama service is running"
            return 0
        fi
    fi
    
    print_step "Downloading Ollama..."
    curl -fsSL https://ollama.ai/install.sh | sh
    print_success "Ollama installed"
    
    # Start Ollama service
    print_step "Starting Ollama service..."
    if command -v systemctl &> /dev/null; then
        # Try user service first (recommended)
        if systemctl --user start ollama.service &> /dev/null 2>&1; then
            systemctl --user enable ollama.service
            print_success "Ollama service started (user mode)"
        elif sudo systemctl start ollama.service &> /dev/null 2>&1; then
            sudo systemctl enable ollama.service
            print_success "Ollama service started (root mode)"
        else
            print_warning "Could not start Ollama service automatically"
            print_step "Start manually with: ollama serve"
        fi
    else
        print_warning "systemctl not found, please start Ollama manually"
        print_step "Run: nohup ollama serve &"
    fi
    
    sleep 3
    
    # Wait for Ollama to be ready
    print_step "Waiting for Ollama to be ready..."
    for i in {1..30}; do
        if curl -s http://localhost:11434/api/tags &> /dev/null; then
            print_success "Ollama is ready"
            return 0
        fi
        sleep 1
    done
    
    print_warning "Ollama may still be starting up, continuing anyway..."
}

# Pull AI models
pull_models() {
    print_header "📥 Downloading AI Models"
    
    # Check if Ollama is responsive
    if ! curl -s http://localhost:11434/api/tags &> /dev/null; then
        print_warning "Ollama not responding, skipping model download"
        print_step "Download models later with:"
        echo "  ollama pull llama:latest"
        echo "  ollama pull phi3"
        return 1
    fi
    
    # Pull models (in background if possible)
    print_step "Pulling llama:latest (4.7 GB, may take 10-15 min)..."
    ollama pull llama:latest &
    LLAMA_PID=$!
    
    print_step "Pulling phi3 (2.7 GB, parallel download)..."
    ollama pull phi3 &
    PHI_PID=$!
    
    # Wait for both
    wait $LLAMA_PID $PHI_PID
    
    print_success "Models downloaded"
}

# Install Kael-OS
install_kaelos() {
    print_header "💾 Installing Kael-OS"
    
    SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
    REPO_ROOT="$(dirname "$SCRIPT_DIR")"
    
    # Check if we have the source
    if [ ! -f "$REPO_ROOT/src-tauri/Cargo.toml" ]; then
        print_error "Cargo.toml not found at $REPO_ROOT/src-tauri/"
        print_step "Make sure you're running this from the Kael-OS repository"
        exit 1
    fi
    
    # Check for Rust
    if ! command -v rustc &> /dev/null; then
        print_error "Rust not installed"
        print_step "Install from: https://rustup.rs/"
        exit 1
    fi
    
    print_step "Building Kael-OS (this may take 2-5 minutes)..."
    cd "$REPO_ROOT/src-tauri"
    cargo build --release
    
    BINARY_PATH="$REPO_ROOT/src-tauri/target/release/kael-os"
    
    if [ ! -f "$BINARY_PATH" ]; then
        print_error "Build failed: binary not found at $BINARY_PATH"
        exit 1
    fi
    
    print_success "Kael-OS built successfully"
    
    # Install to system
    print_step "Installing to /usr/local/bin/..."
    sudo install -Dm 755 "$BINARY_PATH" /usr/local/bin/kael-os
    print_success "Installed to /usr/local/bin/kael-os"
    
    # Desktop entry
    print_step "Creating desktop entry..."
    sudo tee /usr/share/applications/kael-os.desktop > /dev/null <<EOF
[Desktop Entry]
Type=Application
Name=Kael-OS
Comment=AI-Powered Terminal Assistant
Exec=kael-os
Icon=preferences-system-windows
Categories=Development;
Terminal=false
EOF
    print_success "Desktop entry created"
    
    # Create systemd user service (optional)
    print_step "Setting up systemd service..."
    mkdir -p ~/.config/systemd/user/
    tee ~/.config/systemd/user/kael-os.service > /dev/null <<EOF
[Unit]
Description=Kael-OS AI Terminal
After=network.target ollama.service

[Service]
Type=simple
ExecStart=/usr/local/bin/kael-os
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
EOF
    systemctl --user daemon-reload
    print_success "Systemd service configured"
}

# Final status check
final_status() {
    print_header "🔍 Verifying Installation"
    
    echo -e "${BLUE}Component Status:${NC}"
    
    # Check Kael-OS
    if command -v kael-os &> /dev/null; then
        print_success "Kael-OS: Installed"
        kael-os --version 2>/dev/null || echo "  Version: $VERSION"
    else
        print_error "Kael-OS: Not found in PATH"
    fi
    
    # Check Ollama
    if command -v ollama &> /dev/null; then
        print_success "Ollama: Installed"
        OLLAMA_VERSION=$(ollama --version 2>/dev/null || echo "unknown")
        echo "  $OLLAMA_VERSION"
    else
        print_error "Ollama: Not installed"
    fi
    
    # Check Ollama service
    if systemctl --user is-active ollama.service &> /dev/null 2>&1 || \
       systemctl is-active ollama.service &> /dev/null 2>&1; then
        print_success "Ollama Service: Running"
    else
        print_warning "Ollama Service: Not running"
        echo "  Start with: systemctl --user start ollama.service"
    fi
    
    # Check models
    if curl -s http://localhost:11434/api/tags &> /dev/null; then
        MODELS=$(curl -s http://localhost:11434/api/tags | grep -o '"name":"[^"]*' | cut -d'"' -f4)
        if echo "$MODELS" | grep -q "llama"; then
            print_success "AI Models: llama:latest available"
        fi
        if echo "$MODELS" | grep -q "phi"; then
            print_success "AI Models: phi3 available"
        fi
    else
        print_warning "AI Models: Could not check (Ollama not responding)"
    fi
}

# Main installation flow
main() {
    clear
    echo -e "${BLUE}"
    echo "╔════════════════════════════════════════════════════════╗"
    echo "║                                                        ║"
    echo "║          🚀 Kael-OS + Ollama Installation 🤖          ║"
    echo "║                    Version $VERSION                       ║"
    echo "║                                                        ║"
    echo "╚════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    
    print_header "Installation Plan"
    echo "This script will:"
    echo "  1. Install system dependencies (GTK, OpenSSL, etc.)"
    echo "  2. Install Ollama (local AI runtime)"
    echo "  3. Download llama:latest and phi3 models (7+ GB)"
    echo "  4. Build and install Kael-OS"
    echo ""
    echo -e "${BLUE}For Arch Linux users:${NC}"
    echo "  You can also install via AUR package (auto-installs Ollama):"
    echo "  paru -S kael-os"
    echo ""
    echo -e "${YELLOW}Estimated time: 20-30 minutes${NC}"
    echo -e "${YELLOW}Disk space needed: 30+ GB${NC}"
    echo ""
    read -p "Continue? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_step "Installation cancelled"
        exit 0
    fi
    
    # Run installation steps
    install_system_deps
    install_ollama
    pull_models
    install_kaelos
    final_status
    
    # Success message
    print_header "🎉 Installation Complete!"
    echo -e "${GREEN}Kael-OS is ready to use!${NC}\n"
    echo "Launch the app with any of these commands:"
    echo -e "  ${BLUE}kael-os${NC}                    # Command line"
    echo -e "  ${BLUE}Kael-OS${NC}                    # From application menu"
    echo -e "  ${BLUE}systemctl --user start kael-os${NC}  # As service"
    echo ""
    echo "For more info, visit the documentation:"
    echo "  📖 README.md"
    echo "  ⚡ QUICK_REFERENCE.md"
    echo "  🔧 SETUP.md"
    echo ""
    
    # Ask to launch
    read -p "Launch Kael-OS now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        kael-os &
    fi
}

# Run main function
main "$@"
