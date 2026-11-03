<div align="center">

<img src="https://kerimdemirkaynak.github.io/resimler/assdraw.svg" alt="ASSDraw3 logosu" width="128" height="128">

# ASSDraw3

**ASS (Advanced SubStation Alpha) altyazıları için vektör şekil tasarım aracı.**

[![Lisans](https://img.shields.io/badge/Lisans-BSD_3--Clause-97CA01?style=for-the-badge)](LICENSE)
[![Sürüm](https://img.shields.io/github/v/release/KerimDemirkaynak/assdraw?style=for-the-badge&color=8DDFCB&label=Sürüm)](https://github.com/KerimDemirkaynak/assdraw/releases)
[![Web Sitesi](https://img.shields.io/badge/Web_Sitesi-assdraw-00215E?style=for-the-badge)](https://kerimdemirkaynak.github.io/assdraw/)

[**English**](README.md) · [**Türkçe**](README.tr.md)

</div>

---

## Özellikler

- **Şekil bazlı renklendirme** – dolu bir alt şekle çift tıklayarak rengini belirleyin. İç boşluklar dış kontura bağlı kalır; renkler geri alma, kitaplığa kaydetme ve ayar değişikliklerinde korunur.
- **ASS kopyala** – komut alanını dolgu, kenarlık ve alfa etiketleriyle, standart ASS BGR biçiminde tek satır olarak kopyalar.
- **Güvenilir düzenleme** – geri alma/yineleme, arka plan görselini ve kaydırma/yakınlaştırma durumunu değiştirmez; çok satırlı yapıştırmalarda komutlar birbirine karışmaz.
- **Yüksek DPI uyumu** – noktalar ve tutamaçlar uzaklaştırıldığında kullanılabilir kalır, ekran DPI değerine göre ölçeklenir.
- **Ayarlanabilir arka plan** – varsayılan opaklık 0–255 aralığında belirlenebilir.
- **Dönüşümler** – ölçekleme/döndürme ve bilineer dönüşüm; wxGTK çizim düzeltmeleri.
- **Yerleşik yardım** – resimli kılavuzun tamamı (çevrimdışı HTML ve orijinal CHM) her platformda açılır.

<div align="left">
  <img
    src="https://github.com/user-attachments/assets/3d71c12a-763e-4b61-93e0-26d74e654b8f"
    width="400"
    alt="Application Screenshot"
    style="border: 2px solid #444; border-radius: 12px; padding: 5px;"
  />
  <p><b>Uygulamanın Ekran Görüntüsü</b></p>
</div>

### Uygulama Teması

**Çalışma Alanı → Tema → Sistem Varsayılanı / Açık / Koyu** yolunu izleyin.

Tema değişiklikleri anında tüm kullanıcı arayüzüne uygulanır ve kaydedilir. Özelleştirilebilir açık ve koyu tema renkleri birbirinden bağımsız olarak saklanır.

17 adet DPI duyarlı Font Awesome SVG simgesi, altı dil çevirisi ve GTK için alternatif destek içerir.

**wxWidgets 3.3.4 gerektirir.**

## Diller

English, Türkçe, Français, Deutsch, Español, 繁體中文 ve Tiếng Việt. Dil, **Dil** menüsünden çizim kaybolmadan anında değiştirilebilir ve bir sonraki açılış için kaydedilir.

## İndirme

- **Kararlı** – test edilmiş sürümler [Releases](https://github.com/KerimDemirkaynak/assdraw/releases) sayfasında yayınlanır.
- **En son (geliştirme)** – en yeni derlemelere [Actions](https://github.com/KerimDemirkaynak/assdraw/actions) sayfasından ulaşabilirsiniz. Başarılı bir çalıştırmayı açıp paketleri artifacts bölümünden indirin. Bu derlemeler kararsız olabilir.

| Platform | Paket |
| --- | --- |
| Windows x64 | `ASSDraw3-Windows-x64-Setup.exe` (kurulum) veya `ASSDraw3-Windows-x64-Portable.zip` (taşınabilir) |
| Linux x64 | `ASSDraw3-Linux-x64.tar.gz` (Ubuntu 24.04; GTK 3 çalışma zamanını kullanır) |

## Derleme

### Windows (CMake + MSVC)

Gereksinimler: Visual Studio 2022 veya üzeri (MSVC), CMake 3.25+, Python 3 ve Git.

```powershell
git submodule update --init --recursive vendor/agg
.\build\build-win-vcpkg.ps1 -Fresh
```

Alternatif olarak `release-x64`, `debug-x64` veya `debug-x86` preset'lerini kullanabilirsiniz:

```powershell
cmake --preset release-x64
cmake --build --preset release-x64 --config Release
```

### Linux

Gereksinimler: CMake 3.25+, C++17 derleyicisi, Git, Ninja ve GTK 3 geliştirme paketleri (Ubuntu: `libgtk-3-dev pkg-config ninja-build xvfb`).

```sh
cmake -S . -B build-linux -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build-linux
ctest --test-dir build-linux --output-on-failure
```

Arayüz testleri varsa Xvfb üzerinde çalışır. Yalnızca çekirdek testler için `-DASSDRAW_BUILD_APP=OFF` kullanın (wxWidgets ve AGG gerekmez). Çevrimdışı derlemede `ASSDRAW_AGG_ROOT` ile mevcut `agg-src` klasörünü belirtin.

CMake her iki platformda aynı wxWidgets sürümünü derler. Çevrimdışı derlemede
`FETCHCONTENT_SOURCE_DIR_WXWIDGETS` ile açılmış resmi wxWidgets 3.3.4 kaynak
klasörünü belirtin. Kurulu wxWidgets 3.3.4+ için `-DASSDRAW_BUNDLED_WX=OFF`
kullanın. Windows paketleri MSVC çalışma zamanı DLL dosyalarını ve lisansları içerir.

### Bağımlılıklar

- wxWidgets 3.3.4 (SHA-256 ile doğrulanmış resmi sürüm, statik derleme ve bağlama)
- AGG 2.6, `vendor/agg` submodule'ü olarak sabitlenmiştir ([ghaerr/agg-2.6](https://github.com/ghaerr/agg-2.6), commit `c4f36b4`)

## Çeviri

Çeviriler `po/` altındaki UTF-8 gettext dosyalarıdır ve `assdraw.pot` şablonuna dayanır. MO dosyaları derleme sırasında Python 3 (yalnızca standart kütüphane) ile otomatik oluşturulur.

```sh
python i18n_setup.py --check    # çevirileri ve yer tutucuları doğrula
python i18n_setup.py --update   # metinleri çıkar ve birleştir (GNU gettext gerekir)
```

## Sürekli Entegrasyon

GitHub Actions; `master`/`main` dallarına açılan pull request'lerde veya elle başlatıldığında Windows kurulum dosyasını, taşınabilir paketi ve Linux arşivini derler. Dil dosyaları ve kılavuzlar her çalıştırmada doğrulanır.

## Katkıda Bulunanlar

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

Diğer fork'lardan alınan düzeltmelerin kaynakları için [NOTICE.md](NOTICE.md) dosyasına bakın.

## Lisans

[BSD 3 Maddeli Lisans](LICENSE) ile lisanslanmıştır. Standart wxWidgets çeviri dosyaları kendi lisanslarıyla `vendor/wxlocale` altında korunur.
