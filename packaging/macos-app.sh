#!/usr/bin/env bash
# Build a minimal .app bundle for macOS.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/packaging/dist"
APP="$DIST/ASSDraw-rs.app"
BIN="$ROOT/target/release/assdraw-rs"

cd "$ROOT"
cargo build --release

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp "$BIN" "$APP/Contents/MacOS/assdraw-rs"
chmod +x "$APP/Contents/MacOS/assdraw-rs"

# Info.plist
cat > "$APP/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>assdraw-rs</string>
  <key>CFBundleIdentifier</key>
  <string>rs.assdraw.app</string>
  <key>CFBundleName</key>
  <string>ASSDraw-rs</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.3.0</string>
  <key>CFBundleVersion</key>
  <string>0.3.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
PLIST

# Icon (png is fine for basic; for proper icns use iconutil)
if [[ -f "$ROOT/assets/assdraw.png" ]]; then
  cp "$ROOT/assets/assdraw.png" "$APP/Contents/Resources/assdraw.png"
fi

# Optional DMG
if command -v create-dmg >/dev/null 2>&1; then
  create-dmg \
    --volname "ASSDraw-rs" \
    --window-pos 200 120 \
    --window-size 600 400 \
    --icon-size 100 \
    --app-drop-link 400 200 \
    "$DIST/ASSDraw-rs-macOS.dmg" \
    "$APP"
  echo "Created $DIST/ASSDraw-rs-macOS.dmg"
else
  echo "Created $APP"
  echo "Optional: brew install create-dmg  → re-run for .dmg"
fi
