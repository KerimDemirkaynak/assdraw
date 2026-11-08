# ASSDraw-rs

Modern **Rust + egui** rewrite of [ASSDraw3](https://github.com/KerimDemirkaynak/assdraw).

wxWidgets yok. Hedef: orijinal özellik seti + GitHub’da temiz CI.

## Build

```bash
# Rust 1.82+
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

## Kısayollar (özet)

| Tuş | İşlev |
|-----|-------|
| Ctrl+C / Ctrl+V | ASS kopyala / panodan yapıştır (metin) |
| Ctrl+I | Panodaki **resmi** arka plan olarak yapıştır |
| Ctrl+Z / Ctrl+Y | Geri al / yinele |
| Ok tuşları / PageUp-PageDown | Tuvali kaydır / yakınlaştır (Ctrl = 10x) |
| Tab | İmleci en yakın noktaya götürür |
| Shift (basılı) | M/L/B aracını geçici olarak sürükleme aracına çevirir |
| F1–F8 (F5 = Sil), M N L B S P C R H | Araç seçimi |
| Shift+F1/F2/F3 | Çizim / arka plan / ikisi birden kaydır |

> Not: Ctrl+V, pano yalnızca resim içeriyorsa egui tarafından olay olarak iletilmez; bu yüzden
> resim yapıştırmak için Ctrl+I veya menü (Arka plan → Panodan Resim Yapıştır) kullanılır.

## CI / Artifact

Her push'ta `clippy` ve (Linux / Windows / macOS) `check + test + release build` çalışır.
Derlenen programlar **Actions → ilgili çalıştırma → Artifacts** altında indirilebilir.
`v*` etiketi (ör. `git tag v0.3.1 && git push --tags`) atılırsa üç sürüm ayrıca bir GitHub Release'e eklenir.
