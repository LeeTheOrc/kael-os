#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/deploy-webdisk.sh [path]
# Uploads the provided local folder (defaults to website/) to WebDAV base path.

WEBDAV_SERVER=${WEBDAV_SERVER:-leroyonline.co.za}
WEBDAV_PORT=${WEBDAV_PORT:-2078}
WEBDAV_USERNAME=${WEBDAV_USERNAME:-leetheorc}
WEBDAV_PASSWORD=${WEBDAV_PASSWORD:-}
WEBDAV_BASE_PATH=${WEBDAV_BASE_PATH:-/public_html/kael}

LOCAL_DIR=${1:-website}

if [[ -z "$WEBDAV_PASSWORD" ]]; then
  echo "WEBDAV_PASSWORD is required" >&2
  exit 1
fi

base_url="https://${WEBDAV_SERVER}:${WEBDAV_PORT}"

echo "Deploying $LOCAL_DIR to ${base_url}${WEBDAV_BASE_PATH}"

# Ensure base subfolders exist
for d in downloads docs arch repo pkgbuild; do
  curl -sS -X MKCOL -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
    "${base_url}${WEBDAV_BASE_PATH}/$d" >/dev/null || true
done

# Upload files preserving subpaths (one level)
pushd "$LOCAL_DIR" >/dev/null
for f in index.html; do
  curl -sS -T "$f" -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
    "${base_url}${WEBDAV_BASE_PATH}/$f"
done

for sub in downloads docs arch repo pkgbuild; do
  if [[ -f "$sub/index.html" ]]; then
    curl -sS -T "$sub/index.html" -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
      "${base_url}${WEBDAV_BASE_PATH}/$sub/index.html"
  fi
done

# Upload PKGBUILD
if [[ -f pkgbuild/PKGBUILD ]]; then
  curl -sS -T pkgbuild/PKGBUILD -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
    "${base_url}${WEBDAV_BASE_PATH}/pkgbuild/PKGBUILD"
fi
popd >/dev/null

echo "Deploy complete."
