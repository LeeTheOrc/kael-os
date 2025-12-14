# 📦 Kael-OS Dependencies & Bundling Strategy

## Overview

Kael-OS has **build-time dependencies** and **runtime dependencies**. We can bundle some but not others.

---

## 🔧 Build-Time Dependencies (Embedded in Binary)

These are Rust crates compiled into the final executable—**no external installation needed**:

### Core Framework
- **Dioxus** - Reactive UI framework
- **Tauri** - Desktop bridge (system tray, window management)
- **Tokio** - Async runtime

### Network & Cloud
- **reqwest** - HTTP client (Firebase, Cloud Functions)
- **Firebase SDK** - Cloud integration
- **Ollama API client** - HTTP calls to local Ollama

### Security & Storage
- **aes-gcm** - AES-256-GCM encryption
- **pbkdf2** - Key derivation
- **SQLite** - Local database
- **Ring** - Cryptographic operations

### System Integration
- **regex** - Pattern matching for command rewriting
- **sysinfo** - Hardware detection (CPU cores, storage)
- **whoami** - User information

**Binary Size**: ~19 MB (fully self-contained)

---

## 🤖 Runtime Dependencies (External Services)

These **cannot be bundled** because they're standalone applications/services:

### Required
- **Ollama** - Local AI runtime (llama:latest + phi3 models)
- **Rust toolchain** - For building (dev only, not needed for end users)

### Optional
- **Firebase Emulator Suite** - For local Firebase testing (dev only)
- **Node.js** - NOT needed (Cloud Functions run on Google's servers)

---

## ⚡ Installation Scenarios

### Scenario 1: End User (No Development)

**What they need:**
1. Kael-OS binary (we provide)
2. Ollama service (they install)

**Installation:**
```bash
# Option A: Direct install
./scripts/install-direct.sh  # Handles everything

# Option B: Package manager (Arch)
paru -S kael-os-bin

# Option C: Manual
1. Install Ollama: https://ollama.ai
2. Run Kael-OS: kael-os
```

### Scenario 2: Developer

**What they need:**
1. Rust toolchain
2. System libraries (GTK, WebKit, OpenSSL)
3. Ollama service
4. Source code

**Installation:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system deps
./setup-deps.sh

# Install Ollama
curl https://ollama.ai/install.sh | sh

# Build and run
cd src-tauri
cargo run
```

---

## 📥 Bundling Strategy

### ✅ What We CAN Bundle

1. **Ollama installer script** - We can provide a one-click installer
2. **Pre-configured models** - Provide download script for llama + phi3
3. **Install wrapper script** - Detect OS and handle both app + Ollama installation

### ❌ What We CANNOT Bundle

1. **Ollama binary itself** - It's a 200+ MB service with native C code
2. **Pre-downloaded models** - llama is 4.7 GB, phi3 is 2.7 GB (too large to distribute)
3. **Rust compiler** - Only needed for development, not production

---

## 🚀 Unified Installer (Recommended)

Create a single installer that handles both the app and Ollama:

### `scripts/install-all.sh`

```bash
#!/bin/bash
set -e

echo "🚀 Kael-OS + Ollama Complete Installation"

# 1. Install Ollama
if ! command -v ollama &> /dev/null; then
    echo "📥 Installing Ollama..."
    curl https://ollama.ai/install.sh | sh
    sleep 2
fi

# 2. Start Ollama service
echo "🤖 Starting Ollama service..."
systemctl --user enable --now ollama.service || \
    systemctl enable --now ollama.service || \
    nohup ollama serve &

sleep 3

# 3. Pull models if not present
echo "📦 Downloading AI models (first run only)..."
ollama pull llama:latest &
ollama pull phi3 &
wait

# 4. Install Kael-OS
echo "💾 Installing Kael-OS..."
./scripts/install-direct.sh

echo "✅ Installation complete!"
echo "🚀 Launch with: kael-os"
```

---

## 🔍 Auto-Detection & Graceful Degradation

### Startup Check in App

The app should automatically:

1. **Check if Ollama is running** (ping localhost:11434)
2. **Check if models are present** (ask Ollama for available models)
3. **Handle missing Ollama gracefully**:
   - Show setup wizard if Ollama not found
   - Suggest installation command for their OS
   - Still allow Cloud Functions to work

### Implementation Location

**File**: `src-tauri/src/services/ollama_manager.rs` (create new)

```rust
pub async fn check_ollama_setup() -> OllamaStatus {
    match ping_ollama().await {
        Ok(_) => {
            // Ollama is running
            if has_models().await {
                OllamaStatus::Ready
            } else {
                OllamaStatus::MissingModels
            }
        }
        Err(_) => {
            // Ollama not running
            if is_ollama_installed().await {
                OllamaStatus::NotRunning
            } else {
                OllamaStatus::NotInstalled
            }
        }
    }
}

pub enum OllamaStatus {
    Ready,                    // ✅ All good
    NotInstalled,            // ⚠️ Needs installation
    NotRunning,              // ⚠️ Installed but not started
    MissingModels,           // ⚠️ Running but no AI models
}
```

### UI Feedback

When app starts, show status in chat UI:

```
✅ Local AI: Ready (llama:latest, phi3)
☁️  Cloud AI: Connected (Gemini 1.5 Flash)
```

Or if problem:

```
⚠️ Local AI: Not running
   Install: curl https://ollama.ai/install.sh | sh
   Then:    ollama serve

☁️ Cloud AI: Still available
```

---

## 📋 Dependency Matrix

| Component | Type | Size | Bundled | Auto-Install | Notes |
|-----------|------|------|---------|--------------|-------|
| Dioxus | Build | Embedded | ✅ | - | UI framework |
| Tauri | Build | Embedded | ✅ | - | Desktop bridge |
| Tokio | Build | Embedded | ✅ | - | Async runtime |
| reqwest | Build | Embedded | ✅ | - | HTTP client |
| SQLite | Build | Embedded | ✅ | - | Database |
| aes-gcm | Build | Embedded | ✅ | - | Encryption |
| **Ollama** | **Runtime** | **200+ MB** | ❌ | ✅ | **Local AI** |
| llama:latest | Runtime | 4.7 GB | ❌ | ✅ | Primary model |
| phi3 | Runtime | 2.7 GB | ❌ | ✅ | Failover model |
| Rust (dev) | Build | 1.2 GB | ❌ | ❌ | Dev only |

---

## 🎯 Installation Path for v0.3.0

### For End Users

**One-liner:**
```bash
curl -L https://kael-os.dev/install.sh | bash
```

This script:
1. Detects OS
2. Installs system dependencies
3. Installs Ollama (if not present)
4. Downloads/installs Kael-OS binary
5. Pulls AI models
6. Creates desktop shortcut
7. Launches the app

### For Developers

```bash
git clone https://github.com/LeeTheOrc/kael-os.git
cd kael-os/Kael-OS-AI
./setup-deps.sh          # System libs
curl -L https://ollama.ai/install.sh | sh  # Ollama
cd src-tauri && cargo run
```

### For AUR Users (Arch Linux)

```bash
paru -S kael-os-bin  # Fully automated
```

The PKGBUILD:
- Lists `ollama` as dependency
- Auto-downloads Rust
- Creates systemd service

---

## ⚡ Performance Impact

### First Launch Timing

| Stage | Time | Notes |
|-------|------|-------|
| App startup | <500ms | Pure Rust, fast |
| Ollama connectivity check | ~50ms | Simple HTTP ping |
| Model warm-up (first query) | ~1-2s | llama loads into memory |
| Cloud API call (if escalated) | ~200-500ms | Network dependent |
| **Total first user query** | **~2s** | Still acceptable |

### Subsequent Queries

- **Local AI**: ~100ms (model already loaded)
- **Cloud AI**: ~200-500ms (network + Gemini processing)

---

## 🔐 Security Notes

### What's NOT Bundled

❌ **Ollama source code** - Open source, auditable separately  
❌ **Pre-trained models** - Users download from official sources  
❌ **API keys** - Only stored locally in encrypted SQLite  

### What IS Included

✅ **AES-256-GCM encryption** - Protects stored API keys  
✅ **PBKDF2 key derivation** - 100,000 iterations  
✅ **Random nonces/salts** - Cryptographically secure  
✅ **No hardcoded credentials** - All external to binary  

---

## 📝 Next Steps

1. **Create `scripts/install-all.sh`** - Unified installer for app + Ollama
2. **Add `OllamaManager`** - Startup status checks and graceful degradation
3. **Update INSTALLATION.md** - Point to unified installer
4. **Create AUR package** - With Ollama as dependency
5. **Setup CDN** - For binary distribution (GitHub Releases)

---

## 💡 Summary

| Aspect | Answer |
|--------|--------|
| **Can we build AIs into the app?** | No, Ollama is 200+ MB external service |
| **Can we auto-install Ollama?** | Yes, via installer script |
| **Can we ship models with app?** | No, too large (7+ GB combined) |
| **Can we provide download script?** | Yes, `ollama pull llama:latest` |
| **Is app standalone without Ollama?** | Partially—Cloud AI still works |
| **Total installed size** | App: 19 MB + Ollama: 200 MB |
| **First install time** | ~10-15 min (Ollama + models download) |

**Result**: Streamlined installation experience that feels "bundled" even though components are modular and auto-installable.
