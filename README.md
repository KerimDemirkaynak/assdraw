# ASSDraw3

[![License](https://img.shields.io/badge/License-bsd_3_clause-bsd.svg?color=97CA01&logoColor=blue&style=for-the-badge)](https://opensource.org/license/bsd-3-clause/)
[![GitHub Version](https://img.shields.io/github/v/release/KerimDemirkaynak/assdraw?style=for-the-badge&color=8DDFCB&label=Release)](https://github.com/KerimDemirkaynak/assdraw/releases)
[![Website](https://img.shields.io/badge/Website-kerimdemirkaynak.github.io/assdraw-00215E?style=for-the-badge)](https://kerimdemirkaynak.github.io/assdraw/)

**ASSDraw3** is a tool for designing vector shapes to be used in ASS (Advanced SubStation Alpha) subtitle files.

[**English**](README.md) | [**Türkçe**](README.tr.md)

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

## License

All source files in this repository are licensed under a **3-clause BSD license**.  
See [LICENSE](LICENSE) for more information.
