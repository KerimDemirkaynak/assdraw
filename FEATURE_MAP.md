# Orijinal ASSDraw3 → ASSDraw-rs

## Satır sayısı hakkında

| | Orijinal (C++/wx) | ASSDraw-rs (Rust/egui) |
|--|-------------------|------------------------|
| Çekirdek kaynak | ~8800 satır | ~2300 satır (ve büyüyor) |
| UI framework | wxWidgets + AGG (binlerce satır bağlayıcı) | egui (crate) |
| Build | CMake + vcpkg | Cargo |

Rust + egui aynı işi daha az satırda yazar; wx event tabloları, AGG rasterizer sarmalayıcıları ve XPM ikon gömmeleri yok.
Ama **eksik özellik = eksik satır** da doğru: i18n, dolgu, context menu, library preview hâlâ ince.

## Motor (engine.cpp ~784 satır)

| Davranış | Durum |
|----------|--------|
| ParseASS greedy L/B | ✅ `engine/parse.rs` + test |
| GenerateASS | ✅ |
| S/P/C spline | ✅ |
| PointSystem scale/origin | ✅ |
| Undo/Redo snapshot | ✅ |
| Scale/Rotate | ✅ |
| Bilinear | ✅ API |
| Line→Bezier / Bezier→Line | ✅ `engine/convert.rs` |
| Delete command/point | ✅ |

## styled_ass (~248)

| | |
|--|--|
| Contour runs + renk | ✅ `export/` |
| Multiline import offsets | ⚠ kısmi |

## library / settings / canvas

| | |
|--|--|
| Library add/load/json | ✅ temel |
| Settings disk | ✅ |
| BG image + alpha | ✅ |
| Pan/zoom/grid/hit-test | ✅ |
| AGG scanline fill | ✅ lyon even-odd tessellation + egui Mesh |
| i18n 7 dil | ❌ |
| Context menu | ❌ |
| CHM/HTML help | ❌ |

## Test / CI

- Birim: greedy parse, roundtrip, bounds, scale_rotate, styled runs
- GitHub Actions: check + test + release (Linux/Win/macOS)


## Canvas fare modeli (bu sprint)

Orijinal `OnMouseLeftDown/Up/Move`, `OnMouseRight*`, `ProcessOnMouseLeftUp` davranışları:

| Davranış | Rust |
|----------|------|
| Hover: control önce, main öncelikli | `find_point_at` |
| Sol sürükle: nokta taşı / marquee | `LeftDragKind` |
| Sağ sürükle + orta tuş: pan | ✅ |
| Delete: down==up aynı nokta | ✅ |
| Shift/Ctrl seçim modları | New/Add/Toggle |
| Transform köşe tutamaçları | `TransformHandle` + bilinear canlı |
| Context menu: L↔B, move00 | ✅ |
| Status: komut altında imleç | `status_cmd` |
| Double-click renk | ✅ |


## Dolgu (AGG kalitesi hedefi)

- `src/canvas/fill.rs`: cubic flatten, Catmull, even-odd `lyon` tessellate → `egui::Mesh`
- Holes: aynı renk grubundaki konturlar tek path’te even-odd
- UI: Dolgu aç/kapa + opaklık
