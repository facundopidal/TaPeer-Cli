#!/usr/bin/env bash
set -e

REPO="facundopidal/tapeer-cli"

# Detect OS
OS="$(uname -s)"
case "$OS" in
  Linux)  OS_TYPE="unknown-linux-gnu" ;;
  Darwin) OS_TYPE="apple-darwin" ;;
  *)
    echo "Error: Unsupported operating system: $OS" >&2
    exit 1
    ;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64)
    TARGET="x86_64-${OS_TYPE}"
    ;;
  i386|i686)
    TARGET="i686-${OS_TYPE}"
    ;;
  aarch64|arm64)
    TARGET="aarch64-${OS_TYPE}"
    ;;
  *)
    echo "Error: Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
esac

DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/tapeer-${TARGET}.tar.gz"

echo "==> Downloading TaPeer CLI for ${TARGET}..."
TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/tapeer.tar.gz"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TMP_DIR/tapeer.tar.gz" "$DOWNLOAD_URL"
else
  echo "Error: curl or wget is required to download TaPeer CLI." >&2
  exit 1
fi

tar -xzf "$TMP_DIR/tapeer.tar.gz" -C "$TMP_DIR"

# Decide installation directory
if [ -w "/usr/local/bin" ]; then
  INSTALL_DIR="/usr/local/bin"
  mv "$TMP_DIR/tapeer" "$INSTALL_DIR/tapeer"
  chmod +x "$INSTALL_DIR/tapeer"
else
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
  mv "$TMP_DIR/tapeer" "$INSTALL_DIR/tapeer"
  chmod +x "$INSTALL_DIR/tapeer"

  # Ensure INSTALL_DIR is in PATH
  case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
      SHELL_CONFIG=""
      if [ -f "$HOME/.bashrc" ]; then
        SHELL_CONFIG="$HOME/.bashrc"
      elif [ -f "$HOME/.zshrc" ]; then
        SHELL_CONFIG="$HOME/.zshrc"
      elif [ -f "$HOME/.profile" ]; then
        SHELL_CONFIG="$HOME/.profile"
      fi

      if [ -n "$SHELL_CONFIG" ]; then
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$SHELL_CONFIG"
        echo "==> Added $INSTALL_DIR to PATH in $SHELL_CONFIG"
      fi
      ;;
  esac
fi

echo "==> ✓ TaPeer CLI installed successfully to ${INSTALL_DIR}/tapeer"
echo "==> Run 'tapeer --help' to get started!"
