#!/usr/bin/env bash
# Direct installation script for Kael-OS (no package manager required)
# This script builds and installs Kael-OS directly to the system

set -e

VERSION=$(cat version.json | jq -r '.version')
INSTALL_PREFIX="${1:--/usr/local}"  # Allow custom prefix, default to /usr/local

echo "🔨 Building Kael-OS v${VERSION}..."
cargo build --release 2>&1 | tail -5

BINARY_PATH="target/release/kael-os"
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Build failed: Binary not found at $BINARY_PATH"
    exit 1
fi

echo "📦 Installing to $INSTALL_PREFIX..."

# Install binary
sudo install -Dm755 "$BINARY_PATH" "$INSTALL_PREFIX/bin/kael-os"
echo "✅ Binary installed to $INSTALL_PREFIX/bin/kael-os"

# Install desktop entry for menu integration
sudo install -Dm644 /dev/stdin "$INSTALL_PREFIX/share/applications/kael-os.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Kael-OS
Comment=Self-contained forge for building and publishing Arch apps
Exec=/usr/local/bin/kael-os
Icon=kael-os
Terminal=false
Categories=Development;Utility;
StartupNotify=true
MimeType=text/plain;
EOF
echo "✅ Desktop entry installed (menu shortcut created)"

# Install icons
sudo install -Dm644 "src-tauri/icons/icon.png" "$INSTALL_PREFIX/share/pixmaps/kael-os.png"
sudo install -Dm644 "src-tauri/icons/128x128.png" "$INSTALL_PREFIX/share/icons/hicolor/128x128/apps/kael-os.png"
echo "✅ Icons installed"

# Create systemd user service for tray icon (optional, for persistent menubar presence)
sudo install -Dm644 /dev/stdin "$INSTALL_PREFIX/lib/systemd/user/kael-os.service" << EOF
[Unit]
Description=Kael-OS Tray Application
After=graphical-session-reached.target

[Service]
Type=notify
ExecStart=$INSTALL_PREFIX/bin/kael-os
Restart=always
RestartSec=5

[Install]
WantedBy=graphical-session.target
EOF
echo "✅ Systemd user service installed (optional)"

echo ""
echo "🎉 Installation complete!"
echo ""
echo "📝 To launch:"
echo "   $ kael-os"
echo ""
echo "🔧 To enable systemd auto-start (for tray presence):"
echo "   $ systemctl --user enable --now kael-os.service"
echo ""
echo "✨ Kael-OS is now available in your applications menu!"
