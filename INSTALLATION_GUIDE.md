# 🚀 Kael-OS Installation Guide

Welcome! This guide will help you install Kael-OS and Ollama (the local AI engine) on your system.

## 📋 Quick Start (Recommended)

### One-Command Installation

Run this single command in your terminal:

```bash
curl -L https://raw.githubusercontent.com/LeeTheOrc/kael-os/master/scripts/install-all.sh | bash
```

This will:
1. ✅ Install system dependencies (GTK, OpenSSL, etc.)
2. ✅ Install Ollama (local AI runtime)
3. ✅ Download llama:latest and phi3 AI models (7+ GB)
4. ✅ Build and install Kael-OS
5. ✅ Create desktop menu shortcut
6. ✅ Launch the app

**Estimated time**: 20-30 minutes (depends on internet speed)  
**Disk space needed**: 30+ GB

---

## 🐧 Linux Installation (Step by Step)

### Prerequisites

You'll need:
- Ubuntu 20.04+ / Debian 11+ / Fedora 36+ / Arch (any recent distro)
- 30+ GB free disk space
- Internet connection
- Terminal access (any shell: bash, fish, zsh)

### Method 1: Automated Script (Recommended)

```bash
# 1. Clone the repository
git clone https://github.com/LeeTheOrc/kael-os.git
cd kael-os/Kael-OS-AI

# 2. Run the installer
./scripts/install-all.sh

# 3. Follow the prompts
# The script will ask for sudo password for system dependencies
```

### Method 2: Manual Installation

If you prefer to install step-by-step:

#### Step 1: Install System Dependencies

**Ubuntu/Debian:**
```bash
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
```

**Fedora:**
```bash
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
```

**Arch Linux:**
```bash
sudo pacman -Syu
sudo pacman -S \
    openssl \
    gtk3 \
    libappindicator-gtk3 \
    librsvg \
    webkit2gtk \
    base-devel \
    curl \
    git
```

#### Step 2: Install Rust (if not already installed)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Step 3: Install Ollama

```bash
curl https://ollama.ai/install.sh | sh
```

Start the Ollama service:
```bash
# User mode (recommended)
systemctl --user start ollama.service
systemctl --user enable ollama.service

# OR system-wide
sudo systemctl start ollama.service
sudo systemctl enable ollama.service
```

#### Step 4: Download AI Models

```bash
# Download llama:latest (4.7 GB)
ollama pull llama:latest

# Download phi3 as backup (2.7 GB)
ollama pull phi3
```

This will take 10-15 minutes depending on your internet speed.

#### Step 5: Build and Install Kael-OS

```bash
# Navigate to the project
cd /path/to/kael-os/Kael-OS-AI/src-tauri

# Build release version
cargo build --release

# Install to system
sudo install -Dm 755 \
    target/release/kael-os \
    /usr/local/bin/kael-os

# Create desktop entry
sudo tee /usr/share/applications/kael-os.desktop > /dev/null << 'EOF'
[Desktop Entry]
Type=Application
Name=Kael-OS
Comment=AI-Powered Terminal Assistant
Exec=kael-os
Icon=preferences-system-windows
Categories=Development;
Terminal=false
EOF
```

#### Step 6: Launch

```bash
# From terminal
kael-os

# OR from application menu (search for "Kael-OS")
# OR create a systemd service (optional)
```

---

## 🍎 macOS Installation

### Prerequisites

- macOS 10.15+ (Catalina or newer)
- Homebrew installed
- 30+ GB free disk space

### Installation Steps

```bash
# 1. Install Homebrew dependencies
brew install openssl curl git

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 3. Install Ollama
# Download from: https://ollama.ai/download/mac
# OR use: brew install --cask ollama

# 4. Download AI models
ollama pull llama:latest
ollama pull phi3

# 5. Build Kael-OS
cd kael-os/Kael-OS-AI/src-tauri
cargo build --release

# 6. Install and run
cp target/release/kael-os /usr/local/bin/
kael-os
```

---

## 🪟 Windows Installation

Windows support coming soon! For now, use:
- **WSL 2** (Windows Subsystem for Linux) + Ubuntu
- **Virtual Machine** running Linux

### Using WSL 2

```bash
# 1. Enable WSL 2 (Windows PowerShell as Admin)
wsl --install

# 2. Install Ubuntu
wsl --install -d Ubuntu-22.04

# 3. Open Ubuntu terminal and follow Linux installation steps above
```

---

## ✅ Verify Installation

After installation, check that everything is working:

```bash
# Check Kael-OS
kael-os --version

# Check Ollama
ollama --version

# Check Ollama service
systemctl --user status ollama.service

# Check models
ollama list
```

You should see output like:
```
NAME              ID              SIZE     MODIFIED
llama:latest      2c05a317e9fd    4.7 GB   2 hours ago
phi3:latest       7d5a0d0c9ab     2.7 GB   1 hour ago
```

---

## 🚀 First Launch

When you launch Kael-OS for the first time:

1. **Chat panel opens** - Ready for your first question
2. **Local AI warmup** - Models load into memory (~1-2 seconds)
3. **Status indicator** - Shows "✅ Local AI: Ready"
4. **Start chatting!** - Type any question

### Example First Queries

**Local AI** (system tasks):
```
How do I install a package with pacman?
What's my current storage usage?
How do I check running processes?
```

**Cloud AI** (complex tasks):
```
Write a Rust function that implements quicksort
Explain async/await in Rust
Design a database schema for a blog
```

---

## ⚙️ Configuration

### Customize AI Models

Edit or create `~/.config/kael-os/.env`:

```bash
# Primary local model
OLLAMA_MODEL=llama:latest

# Fallback model (if primary unavailable)
OLLAMA_FALLBACK=phi3

# API keys (optional)
GEMINI_API_KEY=your_key_here
MISTRAL_API_KEY=your_key_here
```

### Enable Cloud Providers

In the app: **Settings → Providers**

1. Add your API keys
2. Select which providers to use
3. Kael-OS will auto-fall back if local AI isn't available

---

## 🆘 Troubleshooting

### "Local AI service is not responding"

**Problem**: Ollama isn't running

**Solutions**:
```bash
# Start Ollama manually
ollama serve

# OR enable systemd service
systemctl --user start ollama.service
systemctl --user enable ollama.service

# Check if it's running
curl http://localhost:11434/api/tags
```

### "Local AI models not found"

**Problem**: Models weren't downloaded

**Solution**:
```bash
ollama pull llama:latest
ollama pull phi3
```

### App crashes on startup

**Problem**: Missing system libraries

**Solution**: Reinstall system dependencies
```bash
# Ubuntu/Debian
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev

# Fedora
sudo dnf install gtk3-devel webkit2gtk3-devel

# Arch
sudo pacman -S gtk3 webkit2gtk
```

### Ollama won't start on system boot

**Solution**: Enable systemd service
```bash
# User mode (recommended)
systemctl --user enable ollama.service

# System-wide (requires sudo)
sudo systemctl enable ollama.service
```

### "Cannot build - Rust not found"

**Solution**: Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Models are too slow

**Problem**: Not enough RAM or CPU

**Options**:
1. Use smaller model: `ollama pull tinyllama`
2. Enable GPU acceleration (NVIDIA/AMD)
3. Use cloud providers (faster but requires API key)

---

## 📦 Uninstall

### Remove Kael-OS

```bash
# Remove binary
sudo rm /usr/local/bin/kael-os

# Remove desktop entry
sudo rm /usr/share/applications/kael-os.desktop

# Remove config (optional)
rm -rf ~/.config/kael-os
```

### Remove Ollama (optional)

```bash
# Stop service
systemctl --user stop ollama.service

# Remove Ollama
sudo rm -rf /usr/local/bin/ollama
sudo rm -rf ~/.ollama

# Remove models (if needed to free space)
rm -rf ~/.ollama/models
```

---

## 📊 Disk Space Requirements

| Component | Size | Notes |
|-----------|------|-------|
| Kael-OS binary | 19 MB | Fully self-contained |
| Ollama service | 200 MB | System daemon |
| llama:latest | 4.7 GB | Primary AI model |
| phi3 | 2.7 GB | Failover AI model |
| Cache/Temp | 1-2 GB | Runtime cache |
| **Total** | **~30 GB** | **Minimum recommended** |

---

## 🔧 Advanced Configuration

### Custom Model Merging

You can combine models for better performance:

```bash
# Use a faster model as primary
ollama pull mistral

# Customize in .env
OLLAMA_MODEL=mistral
OLLAMA_FALLBACK=phi3
```

### GPU Acceleration

For NVIDIA GPUs:
```bash
# Install NVIDIA Container Toolkit
sudo apt install nvidia-container-toolkit

# Ollama will auto-detect and use GPU
ollama serve
```

For AMD GPUs:
```bash
# Install ROCm
# Download from: https://rocmdocs.amd.com/en/docs-5.2.1/

# Ollama will auto-detect
ollama serve
```

### Running Multiple Models

You can run different models in different Ollama instances:

```bash
# Terminal 1 (llama)
OLLAMA_NUM_GPU=1 ollama serve

# Terminal 2 (phi3 on CPU)
OLLAMA_MODEL=phi3 OLLAMA_NUM_GPU=0 ollama serve -p 11435
```

---

## 📖 Additional Resources

- **Quick Reference**: [QUICK_REFERENCE.md](QUICK_REFERENCE.md)
- **Setup Guide**: [SETUP.md](SETUP.md)
- **Dependencies**: [DEPENDENCIES_AND_BUNDLING.md](DEPENDENCIES_AND_BUNDLING.md)
- **Hardware Detection**: [HARDWARE_DETECTION_GUIDE.md](HARDWARE_DETECTION_GUIDE.md)

---

## 💬 Getting Help

If you encounter issues:

1. Check **Troubleshooting** section above
2. Review logs: `~/.config/kael-os/logs/`
3. Open an issue: [GitHub Issues](https://github.com/LeeTheOrc/kael-os/issues)
4. Join Discord: [Kael-OS Community](https://discord.gg/kael-os)

---

## 🎉 You're All Set!

Congratulations! Kael-OS is now installed and ready to use.

**First steps**:
1. Launch Kael-OS: `kael-os`
2. Ask a system question (local AI will respond)
3. Add cloud API keys in Settings for advanced questions
4. Explore the Ideas panel for AI-generated suggestions

Happy coding! 🚀
