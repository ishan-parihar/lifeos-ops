#!/usr/bin/env bash
# LifeOS installer — fetches the latest release binary for the current platform
# and installs it to /usr/local/bin/lifeos (or a user-chosen path).
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/ishan-parihar/lifeos-ops/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/ishan-parihar/lifeos-ops/main/install.sh | bash -s -- --version v0.6.1
#   curl -fsSL https://raw.githubusercontent.com/ishan-parihar/lifeos-ops/main/install.sh | bash -s -- --bin-dir ~/.local/bin
set -euo pipefail

REPO="ishan-parihar/lifeos-ops"
VERSION="latest"
BIN_DIR="/usr/local/bin"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --bin-dir) BIN_DIR="$2"; shift 2 ;;
    --help|-h)
      echo "Usage: install.sh [--version v0.6.1] [--bin-dir /usr/local/bin]"
      exit 0 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"
case "$OS" in
  Linux)  PLATFORM_OS="unknown-linux" ;;
  Darwin) PLATFORM_OS="apple-darwin" ;;
  *) echo "Unsupported OS: $OS" >&2; exit 1 ;;
esac
case "$ARCH" in
  x86_64|amd64)  PLATFORM_ARCH="x86_64" ;;
  arm64|aarch64) PLATFORM_ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

# Prefer musl on Linux for a fully static binary (works on glibc + Alpine/musl systems alike).
if [[ "$PLATFORM_OS" == "unknown-linux" ]]; then
  TARGET="${PLATFORM_ARCH}-unknown-linux-musl"
else
  TARGET="${PLATFORM_ARCH}-${PLATFORM_OS}"
fi

# Resolve "latest" → concrete tag via GitHub API
if [[ "$VERSION" == "latest" ]]; then
  echo "Resolving latest release..."
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep -m1 '"tag_name"' | sed -E 's/.*"tag_name":\s*"([^"]+)".*/\1/')"
  if [[ -z "$VERSION" ]]; then
    echo "Could not resolve latest release tag." >&2
    exit 1
  fi
fi

URL="https://github.com/${REPO}/releases/download/${VERSION}/lifeos-${TARGET}.tar.gz"
echo "Downloading lifeos ${VERSION} for ${TARGET}..."
echo "  → ${URL}"

if ! curl -fSL -o "${TMP_DIR}/lifeos.tar.gz" "$URL"; then
  echo "Download failed. The release asset for target ${TARGET} may not exist." >&2
  echo "Check available assets at: https://github.com/${REPO}/releases/tag/${VERSION}" >&2
  exit 1
fi

echo "Extracting..."
tar -xzf "${TMP_DIR}/lifeos.tar.gz" -C "${TMP_DIR}"

# Write to BIN_DIR (sudo if not writable)
if [[ -w "$BIN_DIR" ]]; then
  install -m 0755 "${TMP_DIR}/lifeos" "${BIN_DIR}/lifeos"
else
  echo "bin-dir ${BIN_DIR} not writable — retrying with sudo"
  sudo install -m 0755 "${TMP_DIR}/lifeos" "${BIN_DIR}/lifeos"
fi

echo "Installed: $(command -v lifeos 2>/dev/null || echo "${BIN_DIR}/lifeos")"
lifeos --version || true
echo
echo "Next steps:"
echo "  export NOTION_API_TOKEN=your_token"
echo "  lifeos discover"
echo "  lifeos init && lifeos pull"
