# Packaging ASSDraw-rs

## Windows
1. `cargo build --release`
2. Open `setup.iss` in [Inno Setup](https://jrsoftware.org/isinfo.php) and compile.
3. Output: `installer/ASSDraw-rs-Windows-x64-Setup.exe`

## Linux (AppImage — portable, no root)
```bash
# requires: cargo, appimagetool (or linuxdeploy)
./packaging/linux-appimage.sh
# produces: packaging/dist/ASSDraw-rs-x86_64.AppImage
```

Alternatively install from source:
```bash
cargo install --path .
# binary lands in ~/.cargo/bin/assdraw-rs
```

Debian/Ubuntu `.deb` (optional, needs `cargo-deb`):
```bash
cargo install cargo-deb
cargo deb --release
```

## macOS (.app + optional .dmg)
```bash
./packaging/macos-app.sh
# produces: packaging/dist/ASSDraw-rs.app
# optional DMG if `create-dmg` is installed
```

Or:
```bash
cargo install cargo-bundle
cargo bundle --release
```

## Notes
- Icons: `assets/assdraw.png` / `assets/assdraw.ico`
- Help docs are copied when present under `docs/`
- Cross-compiling is not required; build on each target OS for best results
