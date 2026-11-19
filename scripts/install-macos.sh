#!/usr/bin/env bash
set -euo pipefail

REPO="williamjkc69/gity-x"
ARCH="$(uname -m)"
if [ "$ARCH" = "arm64" ]; then
  ASSET_SUFFIX="macos-arm64.dmg"
else
  ASSET_SUFFIX="macos-x64.dmg"
fi

echo "==> Fetching latest gity x release for macOS ($ARCH)..."
DMG_URL=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
  | grep -oE "https://github.com/${REPO}/releases/download/[^\"]*${ASSET_SUFFIX}" \
  | head -n 1)

if [ -z "$DMG_URL" ]; then
  echo "Error: Could not find ${ASSET_SUFFIX} in latest GitHub Release." >&2
  exit 1
fi

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

DMG_FILE="$TMP_DIR/gity-x.dmg"
MOUNT_DIR="$TMP_DIR/mount"
mkdir -p "$MOUNT_DIR"

echo "==> Downloading $DMG_URL..."
curl -fL "$DMG_URL" -o "$DMG_FILE"

echo "==> Installing gity x.app to /Applications..."
hdiutil attach "$DMG_FILE" -mountpoint "$MOUNT_DIR" -nobrowse -quiet
rm -rf "/Applications/gity x.app"
cp -R "$MOUNT_DIR/gity x.app" "/Applications/gity x.app"
hdiutil detach "$MOUNT_DIR" -quiet

# Remove macOS quarantine attribute for unsigned open-source builds
xattr -cr "/Applications/gity x.app" 2>/dev/null || true

echo "==> Installed! Launch 'gity x' from Applications or Spotlight."
