# ASSDraw3

[![Lisans](https://img.shields.io/badge/LİSANS-bsd_3_clause-bsd.svg?color=97CA01&logoColor=blue&style=for-the-badge)](https://opensource.org/license/bsd-3-clause/)
[![GitHub Sürümü](https://img.shields.io/github/v/release/KerimDemirkaynak/assdraw?style=for-the-badge&color=8DDFCB&label=Sürüm)](https://github.com/KerimDemirkaynak/assdraw/releases)
[![Web Sitesi](https://img.shields.io/badge/Website-kerimdemirkaynak.github.io/assdraw-00215E?style=for-the-badge)](https://kerimdemirkaynak.github.io/assdraw/)

**ASSDraw3**, ASS (Advanced SubStation Alpha) altyazı dosyalarında kullanılmak üzere vektör şekiller tasarlamaya yarayan bir araçtır.

[**English**](README.md) | [**Türkçe**](README.tr.md)

---

## Otomatik Derlemeler (CI/CD)

Bu depo, otomatik derleme ve paketleme işlemleri için **GitHub Actions** kullanmaktadır.

* **Tetikleyiciler**: Derlemeler, `master` veya `main` dallarına yapılan push ve pull request (çekme isteği) işlemlerinde otomatik olarak çalışır. Ayrıca Actions sekmesinden manuel olarak da tetikleyebilirsiniz.
* **Süreç**: İş akışı repoyu klonlar (AGG alt modülü için otomatik bir yedekleme adımı içerir), `vcpkg` önbelleklerini geri yükler, PowerShell derleme betiğini kullanarak uygulamayı derler ve Inno Setup ile çalıştırılabilir bir kurulum dosyası (installer) oluşturur.
* **Çıktılar (Artifacts)**: Derlenen Windows kurulum dosyası (`ASSDraw3_Setup.exe`), doğrudan Actions çalışma sayfasından kolayca indirebileceğiniz bir artifact olarak yüklenir.

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

Bu depodaki tüm kaynak dosyalar **3 maddeli BSD lisansı** ile lisanslanmıştır.  
Daha fazla bilgi için [LICENSE](LICENSE) dosyasına bakınız.
