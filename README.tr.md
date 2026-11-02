# ASSDraw3

[![Lisans](https://img.shields.io/badge/LİSANS-bsd_3_clause-bsd.svg?color=97CA01&logoColor=blue&style=for-the-badge)](https://opensource.org/license/bsd-3-clause/)
[![GitHub Sürümü](https://img.shields.io/github/v/release/KerimDemirkaynak/assdraw?style=for-the-badge&color=8DDFCB&label=Sürüm)](https://github.com/KerimDemirkaynak/assdraw/releases)
[![Web Sitesi](https://img.shields.io/badge/Website-kerimdemirkaynak.github.io/assdraw-00215E?style=for-the-badge)](https://kerimdemirkaynak.github.io/assdraw/)

**ASSDraw3**, ASS (Advanced SubStation Alpha) altyazı dosyalarında kullanılmak üzere vektör şekiller tasarlamaya yarayan bir araçtır.

[**English**](README.md) | [**Türkçe**](README.tr.md)

---

## Dil desteği ve 3.1 sürümü

Dil menüsünde English, Türkçe, Français, Deutsch, Español, 繁體中文 ve Tiếng Việt
bulunur. Dil değişimi anında uygulanır; menü başlığı, araç çubukları, ipuçları,
ayarlar, geri alma açıklamaları ve dönüşüm şablonları da yenilenir. Çizim, arka
plan, görünüm ve henüz uygulanmamış komut metni korunur. Dil tercihi bir sonraki
açılış için kaydedilir. İngilizce kaynak metinleri kullandığı için `po/en.po` yoktur.

`po/` altında altı tam UTF-8 çeviri dosyası, `assdraw.pot` içinde İngilizce kaynak
şablonu bulunur. Derleme, Python 3'ün standart kütüphanesiyle MO dosyalarını
oluşturup uygulamayla paketler. İşletim sistemine ek yerel ayar kurulması veya
sayı biçiminin değiştirilmesi gerekmez. Hakkında penceresinde orijinal katkılar
ve “Bu sürüm Kerim Demirkaynak tarafından oluşturuldu.” bilgisi yer alır.
Uygulama, Windows dosya bilgisi ve kurulum sürümü 3.1 olarak güncellenmiştir.

Orijinal `ASSDraw3.chm` ile taşınabilir HTML hızlı kılavuz pakete dahildir.
Kılavuz, çalışma klasöründen bağımsız olarak uygulamanın bulunduğu yerde aranır.
Windows kurulumu uygulamayı, DLL'leri, dilleri ve kılavuzları `bin` klasöründe
tutar; taşınabilir paketler kendi klasörlerinden çalışır.

Çevirileri denetlemek için `python i18n_setup.py --check` kullanın. GNU gettext
kuruluysa `--update` kaynak metinleri çıkarıp mevcut çevirilerle birleştirir;
C++ kaynaklarını, menüleri, CMake veya kurulum dosyalarını yeniden yazmaz.
CI, pakette dil dosyaları ve kılavuzların bulunduğunu da doğrular.

## Çizim araçları

- **Coloring (Renklendirme)**: Dolu bir alt şekli seçip çift tıklayarak rengini
  belirleyebilirsiniz. İç boşluklar dış kontura bağlı kalır. Renkler, genel
  ayarlar uygulanırken, kitaplığa kaydederken ve geri alırken korunur.
- **Copy ASS (ASS kopyala)**: Komut alanını dolgu, kenarlık ve alfa etiketleriyle
  tek satır olarak kopyalar. Renkler standart ASS BGR biçiminde yazılır.
  Enter komutları uygular; çok satırlı yapıştırmada komutlar birbirine karışmaz.
- Farklı renkteki çizim parçaları dışa aktarılıp tekrar içe alındığında
  ortak çizim alanındaki konumları korunur.
- Çizimi geri almak veya yinelemek, arka plan görselini ve görünümün kaydırma/
  yakınlaştırma durumunu değiştirmez.
- Noktalar ve tutamaçlar uzaklaştırıldığında kullanılabilir boyutta kalır;
  ekranın DPI değerine uyum sağlar. Arka plan için varsayılan opaklık
  ayarlanabilir (0–255).
- Ölçekleme/döndürme, bilineer dönüşüm, kaydırma düğmeleri ve Linux çizim
  yenilemesi için düzeltmeler eklendi. Kaynaklar: [NOTICE.md](NOTICE.md).

## Linux ve testler

CMake 3.25+, C++17 derleyicisi ve Git gereklidir. Ubuntu üzerinde
`libwxgtk3.2-dev ninja-build xvfb` paketlerini kurduktan sonra:

```sh
cmake -S . -B build-linux -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build-linux
ctest --test-dir build-linux --output-on-failure
```

Arayüz testleri, varsa Xvfb üzerinden çalışır. Yalnızca temel renk/geçmiş
testleri için `-DASSDRAW_BUILD_APP=OFF` kullanabilirsiniz; wxWidgets ve AGG
gerekmez. Yerel AGG kopyası yoksa sabitlenmiş sürüm otomatik indirilir.
Çevrimdışı derleme için `ASSDRAW_AGG_ROOT` ile mevcut `agg-src` klasörünü
belirtebilirsiniz.

## Otomatik Derlemeler (CI/CD)

Bu depo, otomatik derleme ve paketleme işlemleri için **GitHub Actions** kullanmaktadır.

* **Tetikleyiciler**: Derlemeler, `master` veya `main` dallarına açılan pull request (çekme isteği) işlemlerinde çalışır. Ayrıca Actions sekmesinden manuel olarak tetikleyebilirsiniz.
* **Süreç**: İş akışı repoyu klonlar (AGG alt modülü için otomatik bir yedekleme adımı içerir), `vcpkg` önbelleklerini geri yükler, PowerShell derleme betiğini kullanarak uygulamayı derler ve Inno Setup ile çalıştırılabilir bir kurulum dosyası (installer) oluşturur.
* **Çıktılar (Artifacts)**: `ASSDraw3-Linux-x64`, çalıştırma iznini koruyan `.tar.gz` içinde Linux uygulamasını içerir. `ASSDraw3-Windows-x64`, Windows uygulamasını, gerekli DLL'leri ve belgeleri içerir. Bu iki paket derleme tamamlanınca, testlerden ve kurulum paketi hazırlanmadan önce yüklenir. `ASSDraw3-Windows-Setup` ise paketleme başarılı olduğunda Windows kurulum dosyasını içerir. Paketleri Actions çalışma sayfasından indirebilirsiniz. Linux uygulaması Ubuntu 24.04 x64 içindir ve wxWidgets 3.2 GTK çalışma zamanı gerektirir.

---

## ASSDraw3'ü Derleme

ASSDraw3 artık Windows üzerinde **CMake + vcpkg** sistemiyle derlenmektedir. Eski `autotools`, Visual C++ 2008 `.vcproj` ve Dev-C++ makefile yapıları terk edilmiştir.

### Bağımlılıklar
- `wxWidgets` kütüphanesi 2.8.x sürümünden **3.3.1** sürümüne (vcpkg üzerinden) yükseltildi.
- `wxWidgets` dinamik olarak (DLL) derlenip bağlanırken, diğer üçüncü taraf kütüphaneler statik olarak kalmaktadır.
- `AGG` kütüphanesi eski 2.5 sürümünden **2.6** sabitlenmiş sürümüne yükseltilmiş ve `vendor/agg` git submodule’ü olarak eklenmiştir.

vcpkg manifest dosyası, Aegisub wangqr fork’unun kullandığı baseline ile aynı tutulmuştur.

### Gereksinimler
- Visual Studio 2022 veya daha yeni MSVC derleme araçları
- `vcpkg` klonu (önerilen konum: `F:\vcpkg`)
  - Alternatif olarak `VCPKG_ROOT` ortam değişkenini kendi vcpkg klasörünüze göre ayarlayabilirsiniz.
- Git submodule’lerin güncellenmesi

```powershell
git submodule update --init --recursive vendor/agg
```

### Derleme

#### 1. PowerShell Betiği (Önerilen)

```powershell
$env:VCPKG_ROOT = 'F:\vcpkg'
.\build\build-win-vcpkg.ps1 -Fresh
```

**Kullanışlı komutlar:**

```powershell
# Debug x64
.\build\build-win-vcpkg.ps1 -Triplet x64-windows -Configuration Debug -BuildDir build-debug-x64 -Fresh

# Debug x86
.\build\build-win-vcpkg.ps1 -Triplet x86-windows -Configuration Debug -Fresh
```

#### 2. CMake Presets ile Derleme

```powershell
$env:VCPKG_ROOT = 'F:\vcpkg'

cmake --preset release-x64
cmake --build --preset release-x64 --config Release
```

**Kullanılabilir preset’ler:**
- `release-x64` (önerilen)
- `debug-x64`
- `debug-x86`

### AGG Submodule

CMake yalnızca `vendor/agg` submodule’ündeki sabitlenmiş sürümü kullanır:

- Depo: https://github.com/ghaerr/agg-2.6.git
- Commit: `c4f36b4432142f22c0bf82c6fbdb41567a236be2`

Sadece `CMakeLists.txt` dosyasında belirtilen kaynak dosyalar derlenmektedir.

---

## Katkıda Bulunanlar

Bu projenin hayata geçmesine yardımcı olan katkıda bulunanlara özel teşekkürler:

| [<img src="https://avatars.githubusercontent.com/u/408944?v=4" width="80" style="border-radius:50%;"/><br><sub><b>Thomas Goyne</b></sub>](https://github.com/tgoyne) | [<img src="https://avatars.githubusercontent.com/u/59959583?v=4" width="80" style="border-radius:50%;"/><br><sub><b>MIRIMIRIM</b></sub>](https://github.com/MIRIMIRIM) | [<img src="https://avatars.githubusercontent.com/u/6241180?v=4" width="80" style="border-radius:50%;"/><br><sub><b>biki-desu</b></sub>](https://github.com/biki-desu) |
| :---: | :---: | :---: |

---

## Lisans

Uygulama kaynakları **3 maddeli BSD lisansı** ile lisanslanmıştır. Standart wxWidgets çevirilerinin kendi lisansları `vendor/wxlocale` altında korunur.
Daha fazla bilgi için [LICENSE](LICENSE) dosyasına bakınız.
