#!/usr/bin/env bash

set -euo pipefail

REPO="${LIFE_TRACKER_GITHUB_REPOSITORY:-Xsiff/life-tracker}"
INSTALL_DIR="${LIFE_TRACKER_INSTALL_DIR:-$HOME/.local/bin}"
TAG=""
FORCE=0

usage() {
  cat <<'EOF'
Install a private life-tracker release from GitHub Releases.

Usage:
  scripts/install_release.sh [--tag vX.Y.Z] [--repo owner/repo] [--dir PATH] [--force]

Environment:
  GH_TOKEN / GITHUB_TOKEN
      GitHub token used for private release downloads when gh is not available.
  LIFE_TRACKER_GITHUB_REPOSITORY
      Override the default repository.
  LIFE_TRACKER_INSTALL_DIR
      Override the default install directory (~/.local/bin).
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      TAG="$2"
      shift 2
      ;;
    --repo)
      REPO="$2"
      shift 2
      ;;
    --dir)
      INSTALL_DIR="$2"
      shift 2
      ;;
    --force)
      FORCE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

ensure_macos() {
  if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "This installer currently supports macOS only." >&2
    exit 1
  fi
}

target_triple() {
  case "$(uname -m)" in
    arm64)
      echo "aarch64-apple-darwin"
      ;;
    x86_64)
      echo "x86_64-apple-darwin"
      ;;
    *)
      echo "Unsupported macOS architecture: $(uname -m)" >&2
      exit 1
      ;;
  esac
}

github_token() {
  if [[ -n "${GH_TOKEN:-}" ]]; then
    printf '%s' "$GH_TOKEN"
    return
  fi

  if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    printf '%s' "$GITHUB_TOKEN"
    return
  fi

  printf ''
}

have_gh_auth() {
  command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1
}

fetch_release_json() {
  local endpoint="$1"
  local token
  token="$(github_token)"

  if have_gh_auth; then
    gh api "$endpoint"
    return
  fi

  if [[ -z "$token" ]]; then
    echo "Private release download requires gh authentication or GH_TOKEN/GITHUB_TOKEN." >&2
    exit 1
  fi

  require_command curl
  curl -fsSL \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: Bearer $token" \
    "https://api.github.com/$endpoint"
}

python_json_field() {
  local program="$1"
  python3 -c "$program"
}

ensure_python_for_json() {
  if ! command -v python3 >/dev/null 2>&1; then
    echo "python3 is required for installer metadata parsing." >&2
    exit 1
  fi
}

download_archive() {
  local tag="$1"
  local asset_name="$2"
  local asset_id="$3"
  local archive_path="$4"
  local token
  token="$(github_token)"

  if have_gh_auth; then
    gh release download "$tag" --repo "$REPO" --pattern "$asset_name" --dir "$(dirname "$archive_path")"
    if [[ "$(dirname "$archive_path")/$asset_name" != "$archive_path" ]]; then
      mv "$(dirname "$archive_path")/$asset_name" "$archive_path"
    fi
    return
  fi

  if [[ -z "$token" ]]; then
    echo "Private release download requires gh authentication or GH_TOKEN/GITHUB_TOKEN." >&2
    exit 1
  fi

  curl -fsSL \
    -H "Accept: application/octet-stream" \
    -H "Authorization: Bearer $token" \
    -o "$archive_path" \
    "https://api.github.com/repos/$REPO/releases/assets/$asset_id"
}

ensure_macos
require_command tar
ensure_python_for_json

TARGET="$(target_triple)"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

if [[ -n "$TAG" ]]; then
  RELEASE_ENDPOINT="repos/$REPO/releases/tags/$TAG"
else
  RELEASE_ENDPOINT="repos/$REPO/releases/latest"
fi

RELEASE_JSON="$(fetch_release_json "$RELEASE_ENDPOINT")"
RESOLVED_TAG="$(
  printf '%s' "$RELEASE_JSON" | python_json_field '
import json, sys
data = json.load(sys.stdin)
print(data["tag_name"])
')"
ASSET_NAME="life-tracker-${RESOLVED_TAG}-${TARGET}.tar.gz"
ASSET_ID="$(
  ASSET_NAME="$ASSET_NAME" printf '%s' "$RELEASE_JSON" | python_json_field '
import json, sys
data = json.load(sys.stdin)
target = __import__("os").environ["ASSET_NAME"]
for asset in data.get('assets', []):
    if asset.get("name") == target:
        print(asset["id"])
        break
else:
    raise SystemExit(1)
' 2>/dev/null || true
)"

if [[ -z "$ASSET_ID" ]]; then
  echo "Could not find release asset $ASSET_NAME in $REPO@$RESOLVED_TAG." >&2
  exit 1
fi

ARCHIVE_PATH="$TMPDIR/$ASSET_NAME"
download_archive "$RESOLVED_TAG" "$ASSET_NAME" "$ASSET_ID" "$ARCHIVE_PATH"

EXTRACT_DIR="$TMPDIR/extract"
mkdir -p "$EXTRACT_DIR"
tar -xzf "$ARCHIVE_PATH" -C "$EXTRACT_DIR"

SOURCE_BINARY="$(find "$EXTRACT_DIR" -type f -name life-tracker -perm -111 | head -n 1)"
if [[ -z "$SOURCE_BINARY" ]]; then
  SOURCE_BINARY="$(find "$EXTRACT_DIR" -type f -name life-tracker | head -n 1)"
fi

if [[ -z "$SOURCE_BINARY" ]]; then
  echo "Could not find life-tracker binary inside $ASSET_NAME." >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"
DEST="$INSTALL_DIR/life-tracker"

if [[ -e "$DEST" && "$FORCE" -ne 1 ]]; then
  echo "$DEST already exists. Re-run with --force to replace it." >&2
  exit 1
fi

cp "$SOURCE_BINARY" "$DEST"
chmod 755 "$DEST"

echo "Installed life-tracker $RESOLVED_TAG to $DEST"
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo "Warning: $INSTALL_DIR is not on PATH for this shell." >&2
fi
