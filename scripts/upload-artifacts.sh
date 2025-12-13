#!/usr/bin/env bash
set -euo pipefail

# Uploads arbitrary artifacts to Web Disk under /artifacts/<git-sha>/

WEBDAV_SERVER=${WEBDAV_SERVER:-leroyonline.co.za}
WEBDAV_PORT=${WEBDAV_PORT:-2078}
WEBDAV_USERNAME=${WEBDAV_USERNAME:-leetheorc}
WEBDAV_PASSWORD=${WEBDAV_PASSWORD:-}
WEBDAV_BASE_PATH=${WEBDAV_BASE_PATH:-/public_html/kael}

ARTIFACTS_DIR=${1:-dist}
GIT_SHA=$(git rev-parse --short HEAD)
REMOTE_DIR="${WEBDAV_BASE_PATH}/artifacts/${GIT_SHA}"
BASE_URL="https://${WEBDAV_SERVER}:${WEBDAV_PORT}"

if [[ -z "$WEBDAV_PASSWORD" ]]; then
  echo "WEBDAV_PASSWORD is required" >&2
  exit 1
fi

# Ensure remote dir
curl -sS -X MKCOL -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
  "${BASE_URL}${REMOTE_DIR}" >/dev/null || true

echo "Uploading artifacts from ${ARTIFACTS_DIR} to ${BASE_URL}${REMOTE_DIR}"
shopt -s nullglob
for f in "${ARTIFACTS_DIR}"/*; do
  name=$(basename "$f")
  curl -sS -T "$f" -u "${WEBDAV_USERNAME}:${WEBDAV_PASSWORD}" \
    "${BASE_URL}${REMOTE_DIR}/$name"
done
echo "Artifacts uploaded to ${BASE_URL}${REMOTE_DIR}"
