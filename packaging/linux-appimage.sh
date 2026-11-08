#!/usr/bin/env bash
# Build a portable AppImage for Linux (x86_64).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/packaging/dist"
APPDIR="$DIST/ASSDraw-rs.AppDir"
BIN="$ROOT/target/release/assdraw-rs"

cd "$ROOT"
cargo build --release

rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" "$APPDIR/usr/share/icons/hicolor/256x256/apps"

cp "$BIN" "$APPDIR/usr/bin/assdraw-rs"
chmod +x "$APPDIR/usr/bin/assdraw-rs"

# Desktop entry
cat > "$APPDIR/usr/share/applications/assdraw-rs.desktop" << 'DESK'
[Desktop Entry]
Type=Application
Name=ASSDraw-rs
Comment=ASS vector shape designer
Exec=assdraw-rs
Icon=assdraw-rs
Categories=Graphics;VectorGraphics;
Terminal=false
DESK

# Icon
if [[ -f "$ROOT/assets/assdraw.png" ]]; then
  cp "$ROOT/assets/assdraw.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/assdraw-rs.png"
  cp "$ROOT/assets/assdraw.png" "$APPDIR/assdraw-rs.png"
fi

cp "$APPDIR/usr/share/applications/assdraw-rs.desktop" "$APPDIR/assdraw-rs.desktop"

# AppRun
cat > "$APPDIR/AppRun" << 'RUN'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/assdraw-rs" "$@"
RUN
chmod +x "$APPDIR/AppRun"

mkdir -p "$DIST"
if command -v appimagetool >/dev/null 2>&1; then
  ARCH=x86_64 appimagetool "$APPDIR" "$DIST/ASSDraw-rs-x86_64.AppImage"
  echo "Created $DIST/ASSDraw-rs-x86_64.AppImage"
elif command -v linuxdeploy >/dev/null 2>&1; then
  linuxdeploy --appdir "$APPDIR" --output appimage
  mv ./*.AppImage "$DIST/" 2>/dev/null || true
  echo "Created AppImage under $DIST"
else
  echo "AppDir ready at $APPDIR"
  echo "Install appimagetool (https://github.com/AppImage/appimagetool) and re-run to produce .AppImage"
  echo "Or distribute the AppDir / use: cargo install --path ."
fi
