#!/bin/sh
set -e

REPO="radther/bludio"
RELEASE_URL="https://github.com/${REPO}/releases/latest"

# ── Detect architecture ──
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  ARCH_TAG="x86_64" ;;
    aarch64) ARCH_TAG="aarch64" ;;
    *)       echo "Error: Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

# ── Fetch latest release tag via redirect ──
# GitHub's /releases/latest redirects to /releases/tag/<tag>. This avoids
# the GitHub API and its strict rate limits (60 req/hr for unauthenticated).
echo "Fetching latest release..."
FINAL_URL=$(curl -fsSL -o /dev/null -w "%{url_effective}" "$RELEASE_URL")
TAG=$(echo "$FINAL_URL" | sed -E 's|.*/tag/||')
if [ -z "$TAG" ] || [ "$TAG" = "$FINAL_URL" ]; then
    echo "Error: Could not determine latest release tag" >&2
    exit 1
fi
echo "Latest release: $TAG"

# ── Download tarball ──
TARBALL="bludio-${TAG}-linux-${ARCH_TAG}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${TAG}/${TARBALL}"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

echo "Downloading $TARBALL..."
curl -fsSL -o "$TMPDIR/$TARBALL" "$URL" || {
    echo "Error: Failed to download $URL" >&2
    exit 1
}

# ── Extract ──
echo "Extracting..."
tar xzf "$TMPDIR/$TARBALL" -C "$TMPDIR"
EXTRACTED_DIR="$TMPDIR/bludio-${TAG}-linux-${ARCH_TAG}"

# ── Detect install mode ──
if [ "$(id -u)" -eq 0 ]; then
    SYSTEM_INSTALL=1
    BIN_DIR="/usr/local/bin"
    DESKTOP_DIR="/usr/share/applications"
    POLICY_DIR="/usr/share/polkit-1/actions"
else
    SYSTEM_INSTALL=0
    BIN_DIR="$HOME/.local/bin"
    DESKTOP_DIR="$HOME/.local/share/applications"
    POLICY_DIR=""
fi

# ── Install binary ──
echo "Installing binary to $BIN_DIR..."
mkdir -p "$BIN_DIR"
cp "$EXTRACTED_DIR/bludio" "$BIN_DIR/bludio"
chmod +x "$BIN_DIR/bludio"

# ── Install .desktop file ──
echo "Installing desktop entry..."
mkdir -p "$DESKTOP_DIR"
cp "$EXTRACTED_DIR/bludio.desktop" "$DESKTOP_DIR/"

# ── Install PolicyKit policy (system only) ──
if [ "$SYSTEM_INSTALL" -eq 1 ]; then
    echo "Installing PolicyKit policy..."
    mkdir -p "$POLICY_DIR"
    cp "$EXTRACTED_DIR/dev.toomosin.bludio.policy" "$POLICY_DIR/"
else
    echo ""
    echo "Warning: Running without root — PolicyKit policy not installed."
    echo "         Privileged Bluetooth operations (restart) will be unavailable."
    echo "         Run with sudo to enable full functionality."
fi

# ── Check PATH for user installs ──
if [ "$SYSTEM_INSTALL" -eq 0 ]; then
    case ":$PATH:" in
        *:"$BIN_DIR":*) ;;
        *)
            echo ""
            echo "Warning: $BIN_DIR is not in your PATH."
            echo "Add this to your shell profile (.bashrc, .zshrc, etc.):"
            echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
            ;;
    esac
fi

# ── Summary ──
echo ""
echo "Bludio $TAG installed successfully!"
echo "  Binary:   $BIN_DIR/bludio"
echo "  Desktop:  $DESKTOP_DIR/bludio.desktop"
if [ "$SYSTEM_INSTALL" -eq 1 ]; then
    echo "  Policy:   $POLICY_DIR/dev.toomosin.bludio.policy"
else
    echo "  Policy:   (skipped — run with sudo for full functionality)"
fi
