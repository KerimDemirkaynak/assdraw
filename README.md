# ASSDraw-rs

Modern **Rust + egui** rewrite of [ASSDraw3](https://github.com/KerimDemirkaynak/assdraw).

wxWidgets yok. Hedef: orijinal özellik seti + GitHub’da temiz CI.

## Build

```bash
# Rust 1.80+
cargo run --release
```

Linux deps: `libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev`

## Yapı

```
src/
  engine/     # Point, DrawCmd, parse, history, transform  (← engine.cpp)
  library/    # Shape library                              (← library.cpp)
  settings/   # Persistent settings                        (← settings.cpp)
  app/        # Application state
  ui/         # egui panels + canvas                       (← canvas.cpp / assdraw.cpp)
```

Ayrıntılı eşleme: [FEATURE_MAP.md](FEATURE_MAP.md)

## License

BSD 3-Clause (orijinal ile uyumlu)
