#!/usr/bin/env bash
# scripts/link-dev.sh - Link release binaries to /usr/local/bin for local development
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd -P)"

TARGET_DIR="${TARGET_DIR:-/usr/local/bin}"
RELEASE_BIN="${REPO_ROOT}/target/release/octo-work"

FORCE_BUILD=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    -b|--build)
      FORCE_BUILD=1
      shift
      ;;
    --target-dir)
      TARGET_DIR="$2"
      shift 2
      ;;
    -h|--help)
      echo "Usage: $0 [options]"
      echo ""
      echo "Options:"
      echo "  -b, --build       Force re-building release binary with 'cargo build --release'"
      echo "  --target-dir DIR  Custom destination directory (default: /usr/local/bin)"
      echo "  -h, --help        Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

# Build release binary if requested or if missing
if [[ "$FORCE_BUILD" -eq 1 ]] || [[ ! -f "$RELEASE_BIN" ]]; then
  echo "==> Building release binary in ${REPO_ROOT}..."
  (cd "$REPO_ROOT" && cargo build --release)
fi

if [[ ! -f "$RELEASE_BIN" ]]; then
  echo "Error: Release binary not found at ${RELEASE_BIN}" >&2
  exit 1
fi

# Ensure target directory exists
if [[ ! -d "$TARGET_DIR" ]]; then
  echo "==> Target directory ${TARGET_DIR} does not exist. Creating..."
  if [[ -w "$(dirname "$TARGET_DIR")" ]]; then
    mkdir -p "$TARGET_DIR"
  else
    sudo mkdir -p "$TARGET_DIR"
  fi
fi

# Determine if sudo is needed for writing to TARGET_DIR
SUDO_CMD=""
if [[ ! -w "$TARGET_DIR" ]]; then
  echo "==> Notice: Writing to ${TARGET_DIR} requires administrative privileges (sudo)."
  SUDO_CMD="sudo"
fi

# Create symlinks for 'work' and 'octo-work'
echo "==> Linking ${RELEASE_BIN} -> ${TARGET_DIR}/work"
$SUDO_CMD ln -sf "$RELEASE_BIN" "${TARGET_DIR}/work"

echo "==> Linking ${RELEASE_BIN} -> ${TARGET_DIR}/octo-work"
$SUDO_CMD ln -sf "$RELEASE_BIN" "${TARGET_DIR}/octo-work"

echo "==> Success! Symlinks created:"
ls -l "${TARGET_DIR}/work" "${TARGET_DIR}/octo-work"

echo "==> Verifying linked binary:"
"${TARGET_DIR}/work" --version
