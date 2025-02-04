# ASSDraw3

[![Lisans](https://img.shields.io/badge/LİSANS-bsd_3_clause-bsd.svg?color=97CA01&logoColor=blue&style=for-the-badge)](https://opensource.org/license/bsd-3-clause/)
[![GitHub Sürümü](https://img.shields.io/github/v/release/KerimDemirkaynak/assdraw?style=for-the-badge&color=8DDFCB&label=Sürüm)](https://github.com/KerimDemirkaynak/assdraw/releases)
[![Web Sitesi](https://img.shields.io/badge/Website-kerimdemirkaynak.github.io/assdraw-00215E?style=for-the-badge)](https://kerimdemirkaynak.github.io/assdraw/)

[**English**](README.md) | [**Türkçe**](README.tr.md)

ASSDraw3, ASS altyazı dosyalarında kullanılacak şekiller tasarlamak için bir araçtır.

## ASSDraw3'ü Derleme

### Windows

Gereksinimler:

1. M$ Visual C++ 2010 veya üzeri (Express sürümü yeterlidir).
2. wxWidgets (3.0 veya üzeri). `nmake` kullanarak derleyin. Yardım için [buraya](http://wiki.wxwidgets.org/Compiling_Using_MSVC_On_The_Commandline) veya dökümantasyona bakabilirsiniz.
3. AGG (2.4.* veya 2.5.* yeterlidir). VSC++ kullanarak derleyin. Bir proje dosyası sağlanmıştır, bu repodaki `./tools/agg25.vcxproj` dosyasına bakabilirsiniz.

Derleme:

1. VC++ proje dosyasını açın.
2. Uygun include ve kütüphane dizinlerini ekleyin.
3. ??? ~~Windows kullanıcısısınız, bu adımı kendiniz çözebilirsiniz~~
4. Kar elde edin.

### Linux

Gereksinimler:

1. AGG: `sudo apt-get install libagg-dev`
2. wxWidgets: `git clone https://github.com/wxWidgets/wxWidgets.git; ./configure; make; sudo make install`

Derleme:

Standart işlem --> `./autogen.sh; ./configure; make`

### OSX

ASSDraw3 henüz OSX üzerinde test edilmedi. OSX üzerinde *çalışması gerekiyor*, deneyin ve çalışıp çalışmadığını bana bildirin, böylece README'yi güncelleyebilirim.

## Lisans

Bu depodaki tüm kaynak dosyalar 3 maddeli BSD lisansı altındadır. Daha fazla bilgi için [LICENSE](LICENSE) dosyasına bakabilirsiniz.
