#!/usr/bin/env bash
# LifeOS installer — fetches the latest release binary for the current platform
# and installs it to /usr/local/bin/lifeos (or a user-chosen path).
#
# Usage:
#   # Public repo:
#   curl -fsSL https://raw.githubusercontent.com/ishan-parihar/lifeos-ops/main/install.sh | bash
#
#   # Private repo (token required for both the script AND the asset download):
#   GITHUB_TOKEN=github_pat_xxx curl -fsSL -H "Authorization: token $GITHUB_TOKEN" \
#     https://raw.githubusercontent.com/ishan-parihar/lifeos-ops/main/install.sh | bash
#
#   # Or pin a version / custom bin dir:
#   ... | bash -s -- --version v0.6.1 --bin-dir ~/.local/bin
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
      echo "Env:   GITHUB_TOKEN — required for private repos"
      exit 0 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

# Token-aware curl: adds Authorization header if GITHUB_TOKEN is set.
curl_auth() {
  if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    curl -fsSL -H "Authorization: token ${GITHUB_TOKEN}" "$@"
  else
    curl -fsSL "$@"
  fi
}

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
# Falls back to gnu if the musl asset is not present in the release.
if [[ "$PLATFORM_OS" == "unknown-linux" ]]; then
  TARGET_MUSL="${PLATFORM_ARCH}-unknown-linux-musl"
  TARGET_GNU="${PLATFORM_ARCH}-unknown-linux-gnu"
else
  TARGET_MUSL=""
  TARGET_GNU="${PLATFORM_ARCH}-${PLATFORM_OS}"
fi

# Resolve "latest" → concrete tag via GitHub API
if [[ "$VERSION" == "latest" ]]; then
  echo "Resolving latest release..."
  VERSION="$(curl_auth "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep -m1 '"tag_name"' | sed -E 's/.*"tag_name":\s*"([^"]+)".*/\1/')"
  if [[ -z "$VERSION" ]]; then
    echo "Could not resolve latest release tag." >&2
    echo "If this is a private repo, set GITHUB_TOKEN before running this script." >&2
    exit 1
  fi
fi

# Try musl first (Linux), then gnu, then macOS target
try_targets=()
[[ -n "$TARGET_MUSL" ]] && try_targets+=("$TARGET_MUSL")
try_targets+=("$TARGET_GNU")

downloaded=0
for TARGET in "${try_targets[@]}"; do
  URL="https://github.com/${REPO}/releases/download/${VERSION}/lifeos-${TARGET}.tar.gz"
  echo "Trying ${TARGET}..."
  echo "  → ${URL}"
  if curl_auth -o "${TMP_DIR}/lifeos.tar.gz" "$URL" 2>/dev/null; then
    echo "  ✓ downloaded"
    downloaded=1
    break
  else
    echo "  ✗ not available"
  fi
done

if [[ "$downloaded" -eq 0 ]]; then
  echo "No suitable release asset found for ${PLATFORM_ARCH}-${PLATFORM_OS}." >&2
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

INSTALLED_PATH="$(command -v lifeos 2>/dev/null || echo "${BIN_DIR}/lifeos")"
echo "Installed: ${INSTALLED_PATH}"
"${INSTALLED_PATH}" --version || true
echo
echo "Next steps:"
echo "  export NOTION_API_TOKEN=your_token"
echo "  lifeos discover"
echo "  lifeos init && lifeos pull"
