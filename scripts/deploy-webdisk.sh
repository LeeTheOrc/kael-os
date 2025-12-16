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
  curl -sS --insecure -X MKCOL -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
    "${base_url}${WEBDAV_BASE_PATH}/$d" >/dev/null 2>&1 || true
done

# Upload files preserving subpaths (one level)
pushd "$LOCAL_DIR" >/dev/null

# Upload root HTML, CSS, JS files
for f in *.html *.css *.js *.sh; do
  if [[ -f "$f" ]]; then
    echo "Uploading $f..."
    curl -sS --insecure -T "$f" -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
      "${base_url}${WEBDAV_BASE_PATH}/$f"
  fi
done

# Upload subdirectory files
for sub in downloads docs arch repo pkgbuild; do
  if [[ -d "$sub" ]]; then
    # Create subdirectory if needed
    curl -sS --insecure -X MKCOL -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
      "${base_url}${WEBDAV_BASE_PATH}/$sub" >/dev/null 2>&1 || true
    
    # Upload all files in subdirectory
    for f in "$sub"/*; do
      if [[ -f "$f" ]]; then
        echo "Uploading $f..."
        curl -sS --insecure -T "$f" -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
          "${base_url}${WEBDAV_BASE_PATH}/$f"
      fi
    done
  fi
done

# Upload nested downloads subdirectories
for nested in downloads/desktop downloads/apk; do
  if [[ -d "$nested" ]]; then
    # Create nested directory
    curl -sS --insecure -X MKCOL -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
      "${base_url}${WEBDAV_BASE_PATH}/$nested" >/dev/null 2>&1 || true
    
    for f in "$nested"/*; do
      if [[ -f "$f" ]]; then
        echo "Uploading $f..."
        curl -sS --insecure -T "$f" -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
          "${base_url}${WEBDAV_BASE_PATH}/$f"
      fi
    done
  fi
done

popd >/dev/null

echo "Deploy complete."
