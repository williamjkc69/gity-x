#!/usr/bin/env bash
set -euo pipefail

APP_NAME="gity x"
BIN_NAME="gity-x"
VERSION="${1:-0.2.0}"
ARCH="${2:-$(uname -m)}"

if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
  TARGET="aarch64-apple-darwin"
  ARCH_LABEL="arm64"
else
  TARGET="x86_64-apple-darwin"
  ARCH_LABEL="x64"
fi

echo "==> Building $BIN_NAME v$VERSION ($TARGET)..."
cargo build --release --target "$TARGET"

DIST_DIR="dist"
STAGE_DIR="$DIST_DIR/dmg-stage"
APP_BUNDLE="$STAGE_DIR/$APP_NAME.app"
DMG_PATH="$DIST_DIR/${BIN_NAME}-v${VERSION}-macos-${ARCH_LABEL}.dmg"

rm -rf "$STAGE_DIR" "$DMG_PATH"
mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"

cp "target/$TARGET/release/$BIN_NAME" "$APP_BUNDLE/Contents/MacOS/$BIN_NAME"
chmod +x "$APP_BUNDLE/Contents/MacOS/$BIN_NAME"
cp "assets/icon.icns" "$APP_BUNDLE/Contents/Resources/AppIcon.icns"

cat > "$APP_BUNDLE/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>gity x</string>
    <key>CFBundleDisplayName</key>
    <string>gity x</string>
    <key>CFBundleIdentifier</key>
    <string>com.williamjkc69.gity-x</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>gity-x</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# Ad-hoc sign so macOS doesn't complain about broken signature locally
codesign --force --deep --sign - "$APP_BUNDLE" 2>/dev/null || true

ln -s /Applications "$STAGE_DIR/Applications"

echo "==> Creating DMG at $DMG_PATH..."
hdiutil create \
  -volname "$APP_NAME" \
  -srcfolder "$STAGE_DIR" \
  -ov \
  -format UDZO \
  "$DMG_PATH"

rm -rf "$STAGE_DIR"
echo "==> Done: $DMG_PATH"
