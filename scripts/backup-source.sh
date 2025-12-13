#!/usr/bin/env bash
set -euo pipefail

# Archives repo source and uploads to private WebDAV path.

WEBDAV_SERVER=${WEBDAV_SERVER:-leroyonline.co.za}
WEBDAV_PORT=${WEBDAV_PORT:-2078}
WEBDAV_USERNAME=${WEBDAV_USERNAME:-leetheorc}
WEBDAV_PASSWORD=${WEBDAV_PASSWORD:-}
PRIVATE_PATH=${PRIVATE_PATH:-/public_html/kael-private}

ARCHIVE_DIR=${ARCHIVE_DIR:-dist}
ARCHIVE_NAME=${ARCHIVE_NAME:-kael-os-source-$(date +%Y%m%d-%H%M%S).tar.gz}

mkdir -p "$ARCHIVE_DIR"

echo "Creating source archive: $ARCHIVE_DIR/$ARCHIVE_NAME"
tar --exclude=target --exclude=node_modules -czf "$ARCHIVE_DIR/$ARCHIVE_NAME" .

if [[ -z "$WEBDAV_PASSWORD" ]]; then
  echo "WEBDAV_PASSWORD is required" >&2
  exit 1
fi

BASE_URL="https://${WEBDAV_SERVER}:${WEBDAV_PORT}"
REMOTE="${PRIVATE_PATH}/${ARCHIVE_NAME}"

echo "Uploading to private WebDAV: ${BASE_URL}${REMOTE}"
curl -sS -T "$ARCHIVE_DIR/$ARCHIVE_NAME" -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
  "${BASE_URL}${REMOTE}"

echo "Backup complete: ${BASE_URL}${REMOTE}"
