#!/usr/bin/env bash
# Build script for creating Arch Linux installer package

set -e

VERSION=$(cat version.json | jq -r '.version')
echo "🔨 Building Kael-OS v${VERSION} Arch installer..."

# Create temp directory for build
BUILD_DIR=$(mktemp -d)
trap "rm -rf $BUILD_DIR" EXIT

echo "📦 Preparing package structure..."
mkdir -p "$BUILD_DIR/kael-os-${VERSION}"

# Copy source to build directory
cp -r . "$BUILD_DIR/kael-os-${VERSION}" --exclude=target --exclude=.git --exclude=node_modules

# Copy PKGBUILD
cp PKGBUILD "$BUILD_DIR/"

# Create tarball
cd "$BUILD_DIR"
tar czf "kael-os-${VERSION}.tar.gz" "kael-os-${VERSION}"

echo "✅ Building package with makepkg..."
cd "$BUILD_DIR"
makepkg --syncdeps --noconfirm 2>&1

# Copy built package to current directory
cd /home/leetheorc/Kael-os/kael-os
cp "$BUILD_DIR"/*.pkg.tar.zst . 2>/dev/null || echo "⚠️  No package built (might need paru/makepkg installed)"

echo ""
echo "📝 Installer build complete!"
echo "   To install: paru -U kael-os-${VERSION}-*.pkg.tar.zst"
echo "   Or: sudo pacman -U kael-os-${VERSION}-*.pkg.tar.zst"
