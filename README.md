<div align="center">

<img src="https://kerimdemirkaynak.github.io/resimler/assdraw.svg" alt="ASSDraw3 logo" width="128" height="128">

# ASSDraw3

**Design vector shapes for ASS (Advanced SubStation Alpha) subtitles.**

[![License](https://img.shields.io/badge/License-BSD_3--Clause-97CA01?style=for-the-badge)](LICENSE)
[![Release](https://img.shields.io/github/v/release/KerimDemirkaynak/assdraw?style=for-the-badge&color=8DDFCB&label=Release)](https://github.com/KerimDemirkaynak/assdraw/releases)
[![Website](https://img.shields.io/badge/Website-assdraw-00215E?style=for-the-badge)](https://kerimdemirkaynak.github.io/assdraw/)

[**English**](README.md) · [**Türkçe**](README.tr.md)

</div>

---

## Features

- **Per-shape coloring** – double-click a filled sub-shape to set its color. Holes stay attached to their parent contour, and colors are preserved on undo, library save, and preference changes.
- **Copy ASS** – copies the command pane as a single line with fill, outline, and alpha tags in standard ASS BGR notation.
- **Reliable editing** – undo/redo never alters the tracing image or the canvas pan/zoom; pasted multiline commands keep their token boundaries.
- **HiDPI-friendly** – point markers and drag handles remain usable when zoomed out and scale with display DPI.
- **Configurable background** – default opacity can be set from 0 to 255.
- **Transforms** – scale/rotate and bilinear transforms, with painting fixes for wxGTK.
- **Built-in help** – the full illustrated manual (offline HTML, plus the original CHM) opens on every platform.

<div align="left">
  <img
    src="https://github.com/user-attachments/assets/3d71c12a-763e-4b61-93e0-26d74e654b8f"
    width="400"
    alt="Application Screenshot"
    style="border: 2px solid #444; border-radius: 12px; padding: 5px;"
  />
  <p><b>Application Screenshot</b></p>
</div>

## Application Theme

Select **Workspace → Theme → System Default / Light / Dark**. Themes update the entire UI instantly and are saved. Custom light/dark colors stay separate. Includes 17 DPI-aware Font Awesome SVG icons, six translations, and GTK fallback. Requires wxWidgets 3.3.4.

## Languages

English, Türkçe, Français, Deutsch, Español, 繁體中文, and Tiếng Việt. The language can be switched at runtime from the **Language** menu without losing your drawing, and the choice is remembered on the next launch.

## Download

- **Stable** – tested releases are published on the [Releases](https://github.com/KerimDemirkaynak/assdraw/releases) page.
- **Latest (development)** – the newest builds are available from the [Actions](https://github.com/KerimDemirkaynak/assdraw/actions) page. Open a successful workflow run and download the packages from its artifacts. These builds may be unstable.

| Platform | Package |
| --- | --- |
| Windows x64 | `ASSDraw3-Windows-x64-Setup.exe` (installer) or `ASSDraw3-Windows-x64-Portable.zip` |
| Linux x64 | `ASSDraw3-Linux-x64.tar.gz` (Ubuntu 24.04, uses the GTK 3 runtime) |

## Building

### Windows (CMake + MSVC)

Requirements: Visual Studio 2022 or newer (MSVC), CMake 3.25+, Python 3, and Git.

```powershell
git submodule update --init --recursive vendor/agg
.\build\build-win-vcpkg.ps1 -Fresh
```

Alternatively, use the presets `release-x64`, `debug-x64`, or `debug-x86`:

```powershell
cmake --preset release-x64
cmake --build --preset release-x64 --config Release
```

### Linux

Requirements: CMake 3.25+, a C++17 compiler, Git, Ninja, and GTK 3 development packages (on Ubuntu: `libgtk-3-dev pkg-config ninja-build xvfb`).

```sh
cmake -S . -B build-linux -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build-linux
ctest --test-dir build-linux --output-on-failure
```

GUI tests run under Xvfb when available. Use `-DASSDRAW_BUILD_APP=OFF` to build only the core tests (no wxWidgets or AGG needed). For offline builds, point `ASSDRAW_AGG_ROOT` to an existing `agg-src` directory.

CMake builds the same wxWidgets version on both platforms. For offline builds,
set `FETCHCONTENT_SOURCE_DIR_WXWIDGETS` to an extracted official wxWidgets 3.3.4
source tree. Use `-DASSDRAW_BUNDLED_WX=OFF` for an installed wxWidgets 3.3.4+.
Windows packages include MSVC runtime DLLs and third-party notices.

### Dependencies

- wxWidgets 3.3.4 (official release verified by SHA-256, built and linked statically)
- AGG 2.6, pinned as the `vendor/agg` submodule ([ghaerr/agg-2.6](https://github.com/ghaerr/agg-2.6), commit `c4f36b4`)

## Translating

Translations are UTF-8 gettext catalogs in `po/`, based on `assdraw.pot`. MO files are compiled automatically during the build using Python 3 (standard library only).

```sh
python i18n_setup.py --check    # validate catalogs and placeholders
python i18n_setup.py --update   # extract and merge messages (requires GNU gettext)
```

## Continuous Integration

GitHub Actions builds the Windows installer, the portable package, and the Linux archive on pull requests to `master`/`main` or on manual dispatch. Packaged catalogs and manuals are verified in every run.

## Contributors

<table>
  <tr>
    <td align="center" valign="top" width="50%"><a href="https://github.com/tgoyne"><img src="https://wsrv.nl/?url=github.com/tgoyne.png&w=96&h=96&fit=cover&mask=circle" width="48" height="48" alt="Thomas Goyne"><br><b>Thomas Goyne</b></a></td>
    <td align="center" valign="top" width="50%"><a href="https://github.com/MIRIMIRIM"><img src="https://wsrv.nl/?url=github.com/MIRIMIRIM.png&w=96&h=96&fit=cover&mask=circle" width="48" height="48" alt="MIRIMIRIM"><br><b>MIRIMIRIM</b></a></td>
  </tr>
  <tr>
    <td align="center" valign="top" width="50%"><a href="https://github.com/biki-desu"><img src="https://wsrv.nl/?url=github.com/biki-desu.png&w=96&h=96&fit=cover&mask=circle" width="48" height="48" alt="biki-desu"><br><b>biki-desu</b></a></td>
    <td align="center" valign="top" width="50%"><a href="https://github.com/amanosatosi"><img src="https://wsrv.nl/?url=github.com/amanosatosi.png&w=96&h=96&fit=cover&mask=circle" width="48" height="48" alt="amanosatosi"><br><b>amanosatosi</b></a></td>
  </tr>
</table>

See [NOTICE.md](NOTICE.md) for attribution of fixes adopted from other forks.

## License

Licensed under the [BSD 3-Clause License](LICENSE). Standard wxWidgets translation catalogs keep their separate license in `vendor/wxlocale`.
