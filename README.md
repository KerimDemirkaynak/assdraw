# ASSDraw3

[![License](https://img.shields.io/badge/License-bsd_3_clause-bsd.svg?color=97CA01&logoColor=blue&style=for-the-badge)](https://opensource.org/license/bsd-3-clause/)
[![GitHub Version](https://img.shields.io/github/v/release/KerimDemirkaynak/assdraw?style=for-the-badge&color=8DDFCB&label=Release)](https://github.com/KerimDemirkaynak/assdraw/releases)
[![Website](https://img.shields.io/badge/Website-kerimdemirkaynak.github.io/assdraw-00215E?style=for-the-badge)](https://kerimdemirkaynak.github.io/assdraw/)

**ASSDraw3** is a tool for designing vector shapes to be used in ASS (Advanced SubStation Alpha) subtitle files.

[**English**](README.md) | [**Türkçe**](README.tr.md)

---

## Languages and version 3.1

The Language menu offers English, Türkçe, Français, Deutsch, Español, 繁體中文,
and Tiếng Việt. Language changes apply immediately, including the menu title,
toolbars, tooltips, settings, undo labels, and transformation templates. Drawing,
background, view, and unapplied command text stay intact. The preference is saved
for the next launch. English uses the original source strings, so there is no
`po/en.po`.

Six complete UTF-8 gettext catalogs live in `po/`; `assdraw.pot` is the English
source template. The build compiles MO files with Python 3 (standard library
only) and packages them beside the executable. No installed OS locale or change
to numeric formatting is required. The About window includes the original
credits and the localized statement “This version was created by Kerim
Demirkaynak.” Windows file and installer versions are 3.1.

The original `ASSDraw3.chm` and a portable HTML quick guide are included. Manual
paths are resolved relative to the executable, independent of the working
directory. The Windows installer keeps the executable, DLLs, catalogs, and
manuals in its `bin` directory; portable packages remain self-contained folders.

For translators: run `python i18n_setup.py --check` to validate all catalogs,
placeholders, and MO round trips. Use `--update` to extract and merge messages
with GNU gettext; it never rewrites C++ code, language menus, CMake, or installer
files. CI also checks that packaged catalogs and manuals exist.

## Drawing tools

- **Coloring**: select a filled sub-shape, then double-click to choose its color.
  Holes stay attached to their enclosing contour. Each sub-shape retains its
  own color when applying editor preferences, saving to the library, or undoing.
- **Copy ASS**: copy the command pane as one line, including fill/outline/alpha
  tags in canonical ASS BGR notation. Enter applies the command pane; pasted
  multiline drawing commands keep their token boundaries.
- Styled drawing runs are offset on export and restored on import so separate
  colors retain their positions in the shared drawing coordinate system.
- Drawing undo/redo leaves the tracing image and canvas pan/zoom unchanged.
- Point markers and drag targets remain usable when zoomed out and scale with
  the display DPI. Background opacity has a configurable default (0–255).
- Scale/rotate, bilinear transforms, pan toggles, and wxGTK painting include
  fixes from the reference forks. See [NOTICE.md](NOTICE.md) for attribution.

## Linux and tests

With CMake 3.25+, a C++17 compiler, Git, wxWidgets development packages, and
Ninja installed:

```sh
cmake -S . -B build-linux -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build-linux
ctest --test-dir build-linux --output-on-failure
```

On Ubuntu, install `libwxgtk3.2-dev ninja-build xvfb`. The GUI regressions run
under Xvfb when available. Core style/history tests can also be built without
wxWidgets or AGG using `-DASSDRAW_BUILD_APP=OFF`. A missing local AGG checkout
is fetched automatically at the pinned revision; `ASSDRAW_AGG_ROOT` can point
to an existing `agg-src` directory for offline builds.

## Automated Builds (CI/CD)

This repository uses **GitHub Actions** for automated builds and packaging.

* **Triggers**: Builds run on pull requests to `master` or `main`, or manually from the Actions tab.
* **Process**: The workflow checks out the repository (with an automated fallback for the AGG submodule), restores `vcpkg` caches, compiles the application using the PowerShell build script, and creates an executable installer using Inno Setup.
* **Artifacts**: `ASSDraw3-Linux-x64` contains the Linux executable in a `.tar.gz` that preserves its executable permission. `ASSDraw3-Windows-x64` contains the Windows executable, required DLLs, and documentation. Both are uploaded immediately after compilation, before regression tests and installer packaging. `ASSDraw3-Windows-Setup` contains the Windows installer after packaging succeeds. Download them from the Actions run page. The Linux binary targets Ubuntu 24.04 x64 and requires the wxWidgets 3.2 GTK runtime.

---

## Building ASSDraw3

ASSDraw3 now builds with **CMake + vcpkg** on Windows. The old `autotools`, Visual C++ 2008 `.vcproj`, and Dev-C++ makefile systems have been deprecated.

### Dependency Baseline
- `wxWidgets` upgraded from 2.8.x to **3.3.1** via vcpkg.
- `wxWidgets` is built and linked dynamically; other third-party dependencies remain static.
- `AGG` upgraded from the old 2.5 version to a pinned **2.6** snapshot stored in the `vendor/agg` submodule.

The vcpkg manifest is pinned to the same baseline used by the Aegisub wangqr fork.

### Requirements
- Visual Studio 2022 or newer (MSVC build tools)
- `vcpkg` checkout (recommended location: `F:\vcpkg`)
  - Or set the `VCPKG_ROOT` environment variable to your vcpkg path
- Git submodules initialized

```powershell
git submodule update --init --recursive vendor/agg
```

### Build

#### 1. PowerShell Helper Script (Recommended)

```powershell
$env:VCPKG_ROOT = 'F:\vcpkg'
.\build\build-win-vcpkg.ps1 -Fresh
```

**Useful variations:**

```powershell
# Debug x64
.\build\build-win-vcpkg.ps1 -Triplet x64-windows -Configuration Debug -BuildDir build-debug-x64 -Fresh

# Debug x86
.\build\build-win-vcpkg.ps1 -Triplet x86-windows -Configuration Debug -Fresh
```

#### 2. CMake Presets

```powershell
$env:VCPKG_ROOT = 'F:\vcpkg'

cmake --preset release-x64
cmake --build --preset release-x64 --config Release
```

**Available presets:**
- `release-x64` (recommended)
- `debug-x64`
- `debug-x86`

### AGG Submodule

CMake only uses the pinned `vendor/agg` submodule:

- Repository: https://github.com/ghaerr/agg-2.6.git
- Commit: `c4f36b4432142f22c0bf82c6fbdb41567a236be2`

Only the source files listed in `CMakeLists.txt` are compiled into the project.

---

## Contributors

A special thanks to the contributors who helped make this project possible:

| [<img src="https://avatars.githubusercontent.com/u/408944?v=4" width="80" style="border-radius:50%;"/><br><sub><b>Thomas Goyne</b></sub>](https://github.com/tgoyne) | [<img src="https://avatars.githubusercontent.com/u/59959583?v=4" width="80" style="border-radius:50%;"/><br><sub><b>MIRIMIRIM</b></sub>](https://github.com/MIRIMIRIM) | [<img src="https://avatars.githubusercontent.com/u/6241180?v=4" width="80" style="border-radius:50%;"/><br><sub><b>biki-desu</b></sub>](https://github.com/biki-desu) | [<img src="https://github.com/amanosatosi.png?size=160" width="80" style="border-radius:50%;"/><br><sub><b>amanosatosi</b></sub>](https://github.com/amanosatosi) |
| :---: | :---: | :---: | :---: |

---

## License

Application source is licensed under a **3-clause BSD license**. Standard wxWidgets translation catalogs retain their separate license in `vendor/wxlocale`.
See [LICENSE](LICENSE) for more information.
