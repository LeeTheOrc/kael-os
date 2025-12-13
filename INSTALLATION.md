# 🚀 Kael-OS Installation & Packaging Guide

## Overview

This document covers how to build and install Kael-OS as a proper system package with menu integration and tray icon support.

## Quick Install (Direct)

For immediate installation without package management:

```bash
./scripts/install-direct.sh
```

This will:
- Build the release binary
- Install to `/usr/local/bin/kael-os`
- Create desktop entry (menu shortcut)
- Install application icons
- Set up optional systemd user service

**Launch:**
```bash
kael-os
```

## Arch Linux Package (PKGBUILD)

### Build Arch Package

```bash
./scripts/build-arch-installer.sh
```

This creates an installable `.pkg.tar.zst` package suitable for:
- Local installation via `sudo pacman -U kael-os-*.pkg.tar.zst`
- Uploading to AUR
- Distribution on other Arch-based systems

### Install Arch Package

```bash
paru -U kael-os-0.1.0-1-x86_64.pkg.tar.zst
# or
sudo pacman -U kael-os-0.1.0-1-x86_64.pkg.tar.zst
```

## Installation Features

### Menu Integration

After installation, Kael-OS appears in your application menu under **Development** category. Click to launch directly from your desktop environment.

**Verification:**
```bash
ls -la /usr/share/applications/kael-os.desktop
```

### Tray/Menubar Icon

The app icon displays in your system tray when running. To ensure it persists:

```bash
systemctl --user enable --now kael-os.service
```

This auto-starts Kael-OS on login and keeps the tray icon visible.

### Uninstall

**Direct install:**
```bash
sudo rm /usr/local/bin/kael-os
sudo rm /usr/local/share/applications/kael-os.desktop
sudo rm /usr/local/share/pixmaps/kael-os.png
```

**Arch package:**
```bash
sudo pacman -R kael-os
```

## Build Profiles

### Development Build (Fast Compile)

```bash
cargo build
./target/debug/kael-os
```

### Release Build (Optimized)

```bash
cargo build --release
./target/release/kael-os
```

Binary size: ~19 MB (fully self-contained)

### Ultra-Minimal Build (Smallest Size)

```bash
cargo build --profile release-minimal
./target/release/kael-os
```

## Distribution

### WebDAV Upload

```bash
./scripts/publish-desktop.sh
```

Uploads installer to `leroyonline.co.za:2078/public_html/kael`

### Firebase Distribution

```bash
./scripts/publish-firebase.sh
```

Uploads to Google Cloud Storage bucket.

### GitHub Releases

```bash
./scripts/publish-rust-native.sh
```

Creates GitHub release with signed assets.

## Troubleshooting

### App won't launch
- Verify binary exists: `which kael-os`
- Check permissions: `ls -la /usr/local/bin/kael-os` (should be 755)
- Try direct path: `/usr/local/bin/kael-os`

### Icon not showing in tray
- Ensure display server supports system tray
- Check Tauri configuration in `src-tauri/tauri.conf.json`
- Verify icon files exist: `ls src-tauri/icons/`

### Menu shortcut missing
- Desktop environment cache may need refresh
- Manually refresh: `update-desktop-database ~/.local/share/applications/`
- Force via environment: `GTK_DEBUG=notifications kael-os`

### Installation to custom location

```bash
./scripts/install-direct.sh /opt/kael-os
# Then launch: /opt/kael-os/bin/kael-os
```

## Building for Other Distros

### Debian/Ubuntu (.deb)

```bash
# Use `cargo-deb` or Tauri's built-in bundler
cargo install cargo-deb
cargo deb --release
```

### RPM (Fedora/openSUSE)

```bash
# Use `cargo-rpm`
cargo install cargo-rpm
cargo rpm build --release
```

### macOS (.dmg)

```bash
# Tauri builds this automatically with Xcode
cargo tauri build --bundles dmg
```

### Windows (.msi)

```bash
# Tauri builds this automatically
cargo tauri build --bundles msi
```

## Release Workflow

1. **Version Bump**
   ```bash
   ./scripts/bump-version.sh
   ```
   Updates `version.json` and propagates to Tauri config.

2. **Test Build**
   ```bash
   cargo build --release
   ```

3. **Package**
   ```bash
   ./scripts/build-arch-installer.sh
   ```

4. **Publish**
   ```bash
   ./scripts/publish-all.sh  # WebDAV + Firebase + GitHub
   ```

## System Requirements

- **OS:** Linux (Arch, Debian, Ubuntu, Fedora, etc.)
- **CPU:** x86_64
- **RAM:** 512 MB minimum
- **Dependencies:** Installed automatically via PKGBUILD

### Runtime Dependencies

- `glibc` - C standard library
- `fontconfig` - Font configuration
- `xorg-libs` - X11 display server libraries
- `libxkbcommon` - Keyboard input handling
- `libxcb` - X11 protocol library
- `dbus` - System message bus

## Security

- Binary is fully self-contained (no external dependencies)
- GPG signing available for release assets
- Firebase auth via OAuth2 JWT tokens
- All credentials stored in `.env.local` (never committed)

---

**Last Updated:** 2024-12-14  
**Version:** 0.1.0-beta.1  
**Maintainer:** Kael-OS Team
