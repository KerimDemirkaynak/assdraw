# Upstream contributions

This integration retains the existing BSD 3-Clause license and source notices.

- [amanosatosi/assdraw](https://github.com/amanosatosi/assdraw), reviewed at
  `f6c5447c5400f7f9ed13e9c167c45d7d49140fb4`: independently colored contours,
  styled ASS import/export and run-position compensation, coloring selection,
  drawing-only undo/redo, usable point targets, multiline command input, and
  regression tests. The new style and history files retain their BSD notices.
- [biki-desu/assdraw](https://github.com/biki-desu/assdraw), reviewed at
  `633a9f2c94436944654474afc98ed3db6ec5b241`: transformation-path and uninitialized
  Bezier fixes, safe property-grid color extraction, default background opacity,
  transformation-dialog layout, zoom slider sizing, and about-dialog positioning.
  The wxGTK bitmap-refresh fix was adapted to reattach locked pixels during
  painting instead of drawing through a stale unlocked buffer.

The existing CMake Windows installer, bitmap bundles, DPI handling,
internationalization automation, and shape library are retained. The historical
GPL-marked custom spline sources present in the reference forks were not added;
this repository continues to use the pinned external AGG 2.6 spline code.

Standard widget translations in `vendor/wxlocale` come from wxWidgets v3.3.1.
Their original source files, contributor notices, wxWindows license, and LGPL
text are retained there. They are separate from the application catalogs in `po`.

`docs/help/original` is an HTML export of ai-chan's included `docs/ASSDraw3.chm`,
with all 22 original topics and 14 illustrations. Original text and credits are
retained; encoding, local links, and navigation are adapted for offline browsers.
`tools/export_manual.py` documents the export; the manifest records source and
exported-file hashes. Normal application builds do not need a CHM extractor.

Native application appearance uses wxWidgets 3.3.4. The bundled build verifies
the official source archive by SHA-256 and links wxWidgets statically. Its
wxWindows license and bundled library notices are installed under `licenses/`.
The MSVC runtime is redistributed with Windows packages.
This software is based in part on the work of the Independent JPEG Group.
