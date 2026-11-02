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

The existing CMake/vcpkg Windows installer, bitmap bundles, DPI handling,
internationalization automation, and shape library are retained. The historical
GPL-marked custom spline sources present in the reference forks were not added;
this repository continues to use the pinned external AGG 2.6 spline code.

Standard widget translations in `vendor/wxlocale` come from wxWidgets v3.3.1.
Their original source files, contributor notices, wxWindows license, and LGPL
text are retained there. They are separate from the application catalogs in `po`.
