# Upstream Contributions & Third-Party Notices

ASSDraw builds upon years of community development. This project preserves the original work, credits its contributors, and maintains the licenses of all incorporated third-party components.

---

## Upstream Contributions

The following projects were reviewed and their improvements integrated into ASSDraw.

### [amanosatosi/assdraw](https://github.com/amanosatosi/assdraw)

<sub>Reviewed revision: `f6c5447c5400f7f9ed13e9c167c45d7d49140fb4`</sub>

- **Contour rendering** — Independent contour coloring and improved selection.
- **ASS compatibility** — Styled import/export and run-position compensation.
- **Editing workflow** — Drawing-only undo/redo, improved point targets, and multiline command input.
- **Reliability** — Regression tests and additional fixes.

Original BSD notices are retained in the relevant source files.

### [biki-desu/assdraw](https://github.com/biki-desu/assdraw)

<sub>Reviewed revision: `633a9f2c94436944654474afc98ed3db6ec5b241`</sub>

- **Geometry** — Transformation-path and Bezier initialization fixes.
- **Color handling** — Safer property-grid color extraction.
- **Interface** — Background opacity, transformation-dialog layout, zoom slider sizing, and About dialog positioning.
- **Rendering** — Adapted wxGTK bitmap-refresh fix, reattaching locked pixels during painting instead of using a stale unlocked buffer.

### [MIRIMIRIM/assdraw](https://github.com/MIRIMIRIM/assdraw)

Reference fixes and upstream adaptations incorporated under the BSD 3-Clause License.

---

## Original Work & Project Maintenance

| Period | Contributor | Work |
|:---|:---|:---|
| 2007 | ai-chan / ASSDraw3 Team | Original ASSDraw3 implementation |
| 2008–2012 | Aegisub Project | Upstream maintenance |
| 2025–present | Kerim Demirkaynak | Modernization, build-system migration, and active maintenance |

Historical source files retain their original copyright notices. Modern contributions are credited separately.

## Third-Party Components

ASSDraw uses the following external libraries, resources, and assets. Their original notices and applicable license texts are preserved in the repository.

| Component | Version / Source | License |
|:---|:---:|:---|
| wxWidgets | 3.3.4 | wxWindows |
| wxWidgets translations | 3.3.1 | wxWindows |
| AGG spline code | 2.6 | AGG License |
| Font Awesome Free | 7.2.0 | CC BY 4.0 |
| Independent JPEG Group | IJG | IJG License |

### wxWidgets

The native application interface uses wxWidgets 3.3.4. The bundled build verifies the official source archive using SHA-256 and links wxWidgets statically.

Its license and bundled library notices are installed under `licenses/`.

Standard widget translations in `vendor/wxlocale` originate from wxWidgets 3.3.1. Original source files, contributor notices, the wxWindows license, and LGPL text are retained.

### Font Awesome

Toolbar icons are sourced from [Font Awesome Free 7.2.0](https://fontawesome.com/).

- Copyright © 2026 Fonticons, Inc.
- License: [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/)
- Source revision: `337dd2045d5621ce0f8567c33c256f3dedeed55c`

Original SVG files and license are retained in `vendor/fontawesome`. Icon paths remain unchanged; embedding only adds uniform padding and replaces `currentColor` with the native theme's text color.

### AGG

The project continues to use the pinned external AGG 2.6 spline implementation by Maxim Shemanarev (2002–2005).

Historical GPL-marked custom spline sources found in reference forks were deliberately excluded.

---

## Original Documentation

`docs/help/original` contains an HTML export of ai-chan's original `docs/ASSDraw3.chm` manual.

| Contents | Included |
|:---|---:|
| Original topics | 22 |
| Illustrations | 14 |
| Original text and credits | Preserved |

Encoding, local links, and navigation have been adapted for offline browsers. Export details are documented in `tools/export_manual.py`, and the manifest records source and exported-file hashes.

Normal application builds do not require a CHM extractor.

---

## Build & Distribution Notes

- The existing CMake Windows installer is retained.
- Original bitmap bundles and DPI handling are preserved.
- Internationalization automation and the shape library remain part of the project.
- Windows packages redistribute the required MSVC runtime.
- Third-party licenses and notices are included with the distributed application.

---

## License & Attribution

ASSDraw retains the **BSD 3-Clause License**.

Redistribution and use remain subject to the applicable license terms. No third-party file is relicensed by its inclusion in this repository.

See [`LICENSE`](LICENSE) and the corresponding notices under `licenses/` and `vendor/` for complete copyright and licensing information.

**Thank you to everyone who contributed to ASSDraw and its upstream projects.**
