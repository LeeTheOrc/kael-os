# Self-Contained Build Configuration

## Overview
This document describes how to build Kael-OS as a fully self-contained application with all dependencies bundled, so it works everywhere without requiring system dependencies.

## Build Types

### 1. Development Build
```bash
cargo build
```
- Fast compilation, unoptimized
- Produces `target/debug/kael-os`
- Use for testing during development

### 2. Release Build (Optimized)
```bash
cargo build --release
```
- Slower compilation, highly optimized
- Produces `target/release/kael-os`
- ~80% smaller and much faster execution
- Use for distribution

## Platform-Specific Builds

### Windows (.msi installer)
```bash
# Install WiX toolset first
# https://wixtoolset.org/releases/

# Build release binary
cargo build --release

# Create installer
wix build --output kael-os.msi --package target/release/kael-os.exe
```

**Result**: `kael-os.msi` - Single-file installer, no dependencies needed on user machine

### Linux (.AppImage)
```bash
# Install appimagetool first
# https://github.com/AppImage/AppImageKit/releases

# Build release binary
cargo build --release

# Create AppImage
mkdir -p AppDir/usr/bin
mkdir -p AppDir/usr/lib
mkdir -p AppDir/usr/share/applications

# Copy binary
cp target/release/kael-os AppDir/usr/bin/

# Create desktop file
cat > AppDir/usr/share/applications/kael-os.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Kael-OS
Exec=kael-os
Icon=kael-os
Categories=Utility;
EOF

# Create AppImage
appimagetool AppDir kael-os-x86_64.AppImage
chmod +x kael-os-x86_64.AppImage
```

**Result**: `kael-os-x86_64.AppImage` - Portable, no installation needed, works on any Linux

### macOS (.dmg)
```bash
# Requires macOS/Xcode

# Build release binary
cargo build --release

# Create .app bundle
mkdir -p kael-os.app/Contents/{MacOS,Resources}
cp target/release/kael-os kael-os.app/Contents/MacOS/
# Copy icon and info plist...

# Create DMG
hdiutil create -volname "Kael-OS" -srcfolder kael-os.app -ov -format UDZO kael-os.dmg
```

**Result**: `kael-os.dmg` - Standard macOS installer

## Self-Contained Features

### 1. Bundled Resources
The app includes:
- ✅ All Rust dependencies (statically linked)
- ✅ Crypto libraries (AES-256-GCM)
- ✅ GPG support (system call to `gpg`)
- ✅ SSL/TLS certificates (self-signed generation)
- ✅ WebDAV file transfer
- ✅ Multi-provider AI support (Ollama, cloud APIs)
- ✅ Terminal emulation (via PTY)

### 2. Runtime Requirements
**Minimal system dependencies:**
- None for basic functionality (fully self-contained binary)
- `gpg` command (optional, only if using GPG signing)
- `curl` or similar (optional, for WebDAV transfers)
- Internet connection (optional, for cloud AI providers)

### 3. Configuration Files
Location: `~/.config/kael-os/` (Linux/macOS) or `%APPDATA%\kael-os\` (Windows)
- `config.toml` - App settings
- `credentials.json` - Encrypted API keys
- `database.db` - Local SQLite database

### 4. Environment Variables
Optional (app works without these):
- `OLLAMA_API_URL` - Local Ollama instance
- `FIREBASE_PROJECT_ID` - Firebase configuration
- `GOOGLE_API_KEY` - Google Gemini API
- `ANTHROPIC_API_KEY` - Claude API

## Build Optimization Flags

### Cargo.toml Release Profile
```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization (slower build)
strip = true            # Remove symbols
```

### File Size Targets
- **Debug build**: ~150MB
- **Release build (unstripped)**: ~45MB
- **Release build (stripped)**: ~25MB
- **Minimal (with UPX)**: ~8MB

### Strip Binary
```bash
# Linux
strip target/release/kael-os

# macOS
strip target/release/kael-os

# Windows (MSVC)
rust-strip target/release/kael-os.exe

# Compression (optional)
upx --best --lzma target/release/kael-os -o kael-os.upx
```

## Docker Self-Contained Build

Build Kael-OS in Docker for maximum portability:

```dockerfile
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y gpg ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/kael-os /usr/local/bin/
ENTRYPOINT ["kael-os"]
```

Build: `docker build -t kael-os:latest .`

## GitHub Actions CI/CD

Auto-build for all platforms:

```yaml
name: Build Release

on:
  push:
    tags: ['v*']

jobs:
  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - uses: softprops/action-gh-release@v1
        with:
          files: target/release/kael-os

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - uses: softprops/action-gh-release@v1
        with:
          files: target/release/kael-os.exe

  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - uses: softprops/action-gh-release@v1
        with:
          files: target/release/kael-os
```

## Verification Checklist

- [ ] Binary runs on fresh OS install (no extra steps)
- [ ] All features work offline (except cloud AI)
- [ ] Config/data saved to correct location
- [ ] File size < 30MB (release build)
- [ ] No warnings/errors on build
- [ ] Smoke test on Windows 10+, Ubuntu 20.04+, macOS 10.15+
- [ ] Hash matches published release
- [ ] Signature verifies with GPG key

## Deployment Scripts

### Linux/macOS Install Script
```bash
#!/bin/bash
set -e
VERSION="0.2.0"
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

wget https://github.com/LeeTheOrc/kael-os/releases/download/v${VERSION}/kael-os-${OS}-${ARCH}
chmod +x kael-os-${OS}-${ARCH}
sudo mv kael-os-${OS}-${ARCH} /usr/local/bin/kael-os
kael-os --version
```

### Windows Install Script (PowerShell)
```powershell
$Version = "0.2.0"
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }
$Url = "https://github.com/LeeTheOrc/kael-os/releases/download/v$Version/kael-os-windows-$Arch.msi"

Invoke-WebRequest $Url -OutFile "kael-os.msi"
msiexec /i kael-os.msi /quiet /norestart
kael-os --version
```

## Testing Self-Contained Build

```bash
# Test in clean environment
docker run --rm -v $(pwd)/target/release/kael-os:/app/kael-os ubuntu:latest /app/kael-os --help

# Check dependencies
ldd target/release/kael-os  # Linux
otool -L target/release/kael-os  # macOS
```

## Success Criteria

✅ Your build is fully self-contained when:
1. **Single binary** - No separate runtime or config needed
2. **No install** - Can copy binary to any location and run
3. **Works offline** - Basic features work without internet
4. **Fast startup** - < 1 second from launch to ready
5. **Portable** - Works on fresh OS without additional packages
6. **Hashable** - Same build produces same binary (reproducible)

