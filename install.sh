#!/usr/bin/env bash
# url-md installer (macOS / Linux)
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Bwkyd/url-md/main/install.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/Bwkyd/url-md/main/install.sh | bash -s v0.1.1
#
# Env:
#   URL_MD_INSTALL  override install dir (default: $HOME/.url-md)

set -euo pipefail

REPO="Bwkyd/url-md"
VERSION="${1:-latest}"
INSTALL_DIR="${URL_MD_INSTALL:-$HOME/.url-md}"
BIN_DIR="$INSTALL_DIR/bin"

# ----- platform detection -----
case "$(uname -ms)" in
    'Darwin arm64')    TARGET=aarch64-apple-darwin ;;
    'Darwin x86_64')
        echo "error: Intel Mac prebuilt binary not available (GitHub macos-13 runner is too slow)." >&2
        echo "       use 'cargo install --git https://github.com/$REPO url-md --locked' instead." >&2
        exit 1 ;;
    'Linux x86_64')    TARGET=x86_64-unknown-linux-gnu ;;
    'Linux aarch64' | 'Linux arm64')
        echo "error: Linux arm64 prebuilt binary not available yet." >&2
        echo "       use 'cargo install --git https://github.com/$REPO url-md --locked' instead." >&2
        exit 1 ;;
    *)
        echo "error: unsupported platform: $(uname -ms)" >&2
        exit 1 ;;
esac

# ----- resolve download URL -----
if [ "$VERSION" = "latest" ]; then
    URL="https://github.com/$REPO/releases/latest/download/url-md-$TARGET.tar.gz"
else
    URL="https://github.com/$REPO/releases/download/$VERSION/url-md-$TARGET.tar.gz"
fi

# ----- download + extract -----
mkdir -p "$BIN_DIR"
TMP=$(mktemp -d)
trap "rm -rf '$TMP'" EXIT

echo "↓ $URL"
curl --fail --location --progress-bar -o "$TMP/url-md.tar.gz" "$URL"
tar -xzf "$TMP/url-md.tar.gz" -C "$TMP"

# the archive layout is "<archive>/url-md"; find and copy
BINARY=$(find "$TMP" -name url-md -type f -perm -u+x | head -1)
[ -z "$BINARY" ] && BINARY=$(find "$TMP" -name url-md -type f | head -1)
[ -z "$BINARY" ] && { echo "error: url-md binary not found in archive" >&2; exit 1; }

install -m 755 "$BINARY" "$BIN_DIR/url-md"

# ----- output -----
INSTALLED_VERSION="$("$BIN_DIR/url-md" --version 2>/dev/null || echo 'unknown')"
echo ""
echo "✓ Installed: $BIN_DIR/url-md ($INSTALLED_VERSION)"
echo ""

# ----- PATH hint -----
if echo ":$PATH:" | grep -q ":$BIN_DIR:"; then
    echo "  $BIN_DIR is already on PATH — run 'url-md --help' to get started."
else
    case "${SHELL##*/}" in
        zsh)  RC="$HOME/.zshrc" ;;
        bash) RC="$HOME/.bashrc" ;;
        fish) RC="$HOME/.config/fish/config.fish" ;;
        *)    RC="your shell rc file" ;;
    esac
    echo "  Add to PATH by appending this line to $RC:"
    echo ""
    echo "    export PATH=\"$BIN_DIR:\$PATH\""
    echo ""
    echo "  Then 'source $RC' (or reopen shell) and run 'url-md --help'."
fi
