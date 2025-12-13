#!/usr/bin/env bash
set -euo pipefail

# Unified publish script using Rust-native Tauri commands
# No external dependencies: no Python, no gh CLI, no gsutil
# All publishing happens through the Kael-OS binary via Tauri IPC

VERSION_FILE="version.json"
if [[ ! -f "$VERSION_FILE" ]]; then
  echo "Error: $VERSION_FILE not found. Run scripts/bump-version.sh first." >&2
  exit 1
fi

# Extract version
VERSION=$(jq -r '.major as $maj | .minor as $min | .patch as $pat | "\($maj).\($min).\($pat)"' "$VERSION_FILE")
STAGE=$(jq -r '.stage' "$VERSION_FILE")
BUILD=$(jq -r '.build' "$VERSION_FILE")
SEMVER="v${VERSION}-${STAGE}.${BUILD}"

OUT_DIR=${OUT_DIR:-dist}
PKG_NAME="kael-os-${VERSION}-x86_64"

# WebDAV settings
WEBDAV_SERVER=${WEBDAV_SERVER:-leroyonline.co.za}
WEBDAV_PORT=${WEBDAV_PORT:-2078}
WEBDAV_USERNAME=${WEBDAV_USERNAME:-leetheorc}
WEBDAV_PASSWORD=${WEBDAV_PASSWORD:-}
WEBDAV_BASE_PATH=${WEBDAV_BASE_PATH:-/public_html/kael}

# Firebase settings
FIREBASE_BUCKET=${FIREBASE_BUCKET:-}
FIREBASE_SA_JSON=${FIREBASE_SA_JSON:-firebase-service-account.json}

# GitHub settings
GITHUB_TOKEN=${GITHUB_TOKEN:-}
GITHUB_OWNER=${GITHUB_OWNER:-LeeTheOrc}
GITHUB_REPO=${GITHUB_REPO:-kael-os}

echo "=========================================="
echo "Publishing $SEMVER"
echo "=========================================="

# 1. Sign with GPG (this still uses the system gpg, which is minimal)
echo ""
echo "== Signing release with GPG =="
if [[ ! -f "$OUT_DIR/${PKG_NAME}.tar.gz" ]]; then
  echo "Error: Tarball not found. Run cargo build --release first." >&2
  exit 1
fi

if ! command -v gpg >/dev/null 2>&1; then
  echo "Warning: gpg not found, skipping signature." >&2
else
  gpg --default-key D0513E222E8EE8D7 --detach-sign --armor "$OUT_DIR/${PKG_NAME}.tar.gz" || true
fi

# 2. Publish to WebDAV via Rust command
echo ""
echo "== Publishing to WebDAV =="
echo "Target: https://${WEBDAV_SERVER}:${WEBDAV_PORT}${WEBDAV_BASE_PATH}/downloads/desktop"

# Since we don't have a CLI interface yet, this is a placeholder for the Rust binary to call
# In practice, the app will invoke these Tauri commands via its UI or IPC
echo "Note: WebDAV upload currently requires app UI or direct curl call"
echo "Alternative (direct curl): curl -u ${WEBDAV_USERNAME}:*** -T dist/${PKG_NAME}.tar.gz https://${WEBDAV_SERVER}:${WEBDAV_PORT}${WEBDAV_BASE_PATH}/downloads/desktop/"

# 3. Publish to Firebase via Rust command (no Python!)
if [[ -n "$FIREBASE_BUCKET" && -f "$FIREBASE_SA_JSON" ]]; then
  echo ""
  echo "== Publishing to Firebase Storage =="
  echo "Note: Firebase upload will be called via Tauri command from the app"
  echo "Command: firebase_upload_file($FIREBASE_BUCKET, $FIREBASE_SA_JSON, dist/${PKG_NAME}.tar.gz, releases/desktop/${PKG_NAME}.tar.gz)"
else
  echo ""
  echo "Skipping Firebase: FIREBASE_BUCKET not set or service account JSON missing"
fi

# 4. Publish to GitHub via Rust command (no gh CLI!)
if [[ -n "$GITHUB_TOKEN" ]]; then
  echo ""
  echo "== Publishing to GitHub Releases =="
  echo "Note: GitHub upload will be called via Tauri command from the app"
  echo "Command: github_create_release($GITHUB_OWNER, $GITHUB_REPO, $GITHUB_TOKEN, $SEMVER, Kael-OS $SEMVER, Release notes)"
else
  echo ""
  echo "Skipping GitHub: GITHUB_TOKEN not set"
fi

echo ""
echo "=========================================="
echo "Next: Use Kael-OS UI to publish to all targets"
echo "The app will invoke Rust Tauri commands:"
echo "  - webdav_upload_file()"
echo "  - firebase_upload_file()"
echo "  - github_create_release()"
echo "  - github_upload_asset()"
echo "=========================================="
