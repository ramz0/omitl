#!/usr/bin/env sh
# Omitl installer for Linux and macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/yourusername/omitl/main/install.sh | sh

set -e

REPO="yourusername/omitl"
BINARY="omitl"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# ── Detect OS and architecture ────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  TARGET="omitl-linux-x86_64" ;;
      aarch64) TARGET="omitl-linux-arm64"  ;;
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
    esac
    EXT="tar.gz"
    ;;
  Darwin)
    case "$ARCH" in
      x86_64)  TARGET="omitl-macos-x86_64" ;;
      arm64)   TARGET="omitl-macos-arm64"  ;;
      *) echo "Unsupported architecture: $ARCH" && exit 1 ;;
    esac
    EXT="tar.gz"
    ;;
  *)
    echo "Unsupported OS: $OS"
    echo "For Windows, use: irm https://raw.githubusercontent.com/${REPO}/main/install.ps1 | iex"
    exit 1
    ;;
esac

# ── Fetch latest release tag ──────────────────────────────────────────────
echo "Fetching latest release..."
TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" |
  grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

if [ -z "$TAG" ]; then
  echo "Could not determine latest version. Check your internet connection."
  exit 1
fi

echo "Installing omitl ${TAG} for ${OS}/${ARCH}..."

# ── Download and install ──────────────────────────────────────────────────
URL="https://github.com/${REPO}/releases/download/${TAG}/${TARGET}.${EXT}"
TMP="$(mktemp -d)"

curl -fsSL "$URL" | tar xz -C "$TMP"

if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP/$BINARY" "$INSTALL_DIR/$BINARY"
else
  sudo mv "$TMP/$BINARY" "$INSTALL_DIR/$BINARY"
fi

chmod +x "$INSTALL_DIR/$BINARY"
rm -rf "$TMP"

echo ""
echo "omitl ${TAG} installed to ${INSTALL_DIR}/${BINARY}"
echo "Run 'omitl --help' to get started."
