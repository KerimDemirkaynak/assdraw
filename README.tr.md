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

## Diller

English, Türkçe, Français, Deutsch, Español, 繁體中文 ve Tiếng Việt. Dil, **Dil** menüsünden çizim kaybolmadan anında değiştirilebilir ve bir sonraki açılış için kaydedilir.

## İndirme

En güncel sürümü [Releases](https://github.com/KerimDemirkaynak/assdraw/releases) sayfasından indirebilirsiniz.

| Platform | Paket |
| --- | --- |
| Windows x64 | `ASSDraw3-Windows-x64-Setup.exe` (kurulum) veya `ASSDraw3-Windows-x64-Portable.zip` (taşınabilir) |
| Linux x64 | `ASSDraw3-Linux-x64.tar.gz` (Ubuntu 24.04; wxWidgets 3.2 GTK çalışma zamanı gerekir) |

## Derleme

### Windows (CMake + vcpkg)

Gereksinimler: Visual Studio 2022 veya üzeri (MSVC), [vcpkg](https://github.com/microsoft/vcpkg) ve Git.

```powershell
git submodule update --init --recursive vendor/agg
$env:VCPKG_ROOT = 'F:\vcpkg'
.\build\build-win-vcpkg.ps1 -Fresh
```

Alternatif olarak `release-x64`, `debug-x64` veya `debug-x86` preset'lerini kullanabilirsiniz:

```powershell
cmake --preset release-x64
cmake --build --preset release-x64 --config Release
```

### Linux

Gereksinimler: CMake 3.25+, C++17 derleyicisi, Git, Ninja ve wxWidgets geliştirme paketleri (Ubuntu: `libwxgtk3.2-dev ninja-build xvfb`).

```sh
cmake -S . -B build-linux -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build-linux
ctest --test-dir build-linux --output-on-failure
```

Arayüz testleri varsa Xvfb üzerinde çalışır. Yalnızca çekirdek testler için `-DASSDRAW_BUILD_APP=OFF` kullanın (wxWidgets ve AGG gerekmez). Çevrimdışı derlemede `ASSDRAW_AGG_ROOT` ile mevcut `agg-src` klasörünü belirtin.

### Bağımlılıklar

- wxWidgets 3.3.1 (vcpkg üzerinden, dinamik bağlama)
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
    <td align="center"><a href="https://github.com/tgoyne"><img src="https://avatars.githubusercontent.com/u/408944?v=4" width="80" alt="Thomas Goyne"><br><sub><b>Thomas Goyne</b></sub></a></td>
    <td align="center"><a href="https://github.com/MIRIMIRIM"><img src="https://avatars.githubusercontent.com/u/59959583?v=4" width="80" alt="MIRIMIRIM"><br><sub><b>MIRIMIRIM</b></sub></a></td>
    <td align="center"><a href="https://github.com/biki-desu"><img src="https://avatars.githubusercontent.com/u/6241180?v=4" width="80" alt="biki-desu"><br><sub><b>biki-desu</b></sub></a></td>
    <td align="center"><a href="https://github.com/amanosatosi"><img src="https://github.com/amanosatosi.png?size=160" width="80" alt="amanosatosi"><br><sub><b>amanosatosi</b></sub></a></td>
  </tr>
</table>

Diğer fork'lardan alınan düzeltmelerin kaynakları için [NOTICE.md](NOTICE.md) dosyasına bakın.

## Lisans

[BSD 3 Maddeli Lisans](LICENSE) ile lisanslanmıştır. Standart wxWidgets çeviri dosyaları kendi lisanslarıyla `vendor/wxlocale` altında korunur.
