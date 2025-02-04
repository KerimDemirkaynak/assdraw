# ASSDraw

[![Lisans](https://img.shields.io/badge/LİSANS-MIT-blue.svg?color=97CA01&logoColor=blue&style=for-the-badge)](https://opensource.org/license/mit/)  
[![GitHub Sürümü](https://img.shields.io/github/v/release/KerimDemirkaynak/assdraw?style=for-the-badge&color=8DDFCB&label=Sürüm)](https://github.com/KerimDemirkaynak/assdraw/releases)  
[![Web Sitesi](https://img.shields.io/badge/Website-kerimdemirkaynak.github.io/assdraw-00215E?style=for-the-badge)](https://kerimdemirkaynak.github.io/assdraw/)  

[**English**](README.md) | [**Türkçe**](README.tr.md)  

**ASSDraw**, **ASS (Advanced SubStation Alpha)** altyazı dosyaları için vektörel şekiller oluşturup düzenlemeye yönelik bir araçtır. C++ ile geliştirilmiş olup, grafik çizim işlemlerini yönetmek için **AGG (Anti-Grain Geometry)** kütüphanesini kullanır. Araç, hassas ve karmaşık şekiller oluşturmak için sezgisel bir arayüz sunar ve bu şekilleri **ASS altyazı formatına** aktarmanıza olanak tanır.  

## Özellikler
- **Vektörel Çizim**: ASS altyazı biçimlendirmelerinde yaygın olarak kullanılan dikdörtgen, daire ve çizgi gibi şekiller oluşturabilirsiniz.  
- **ASS Dışa Aktarma**: Oluşturduğunuz şekilleri doğrudan **ASS altyazı formatında** dışa aktarabilirsiniz.  
- **Özelleştirilebilir Arayüz**: wxWidgets entegrasyonu sayesinde kullanıcı arayüzünü esnek bir şekilde ayarlayabilirsiniz.  
- **Çoklu Platform Desteği**: Farklı platformlarla uyumlu olacak şekilde tasarlanmıştır.  

## Kurulum

1. Depoyu yerel makinenize klonlayın:  
   ```bash
   git clone https://github.com/KerimDemirkaynak/ASSDraw.git
   ```

2. Gerekli bağımlılıkları yükleyin:  
   - **AGG (Anti-Grain Geometry)**: Vektör grafiklerin işlenmesi için yüksek kaliteli bir grafik kütüphanesi.  
   - **wxWidgets**: Grafiksel kullanıcı arayüzü oluşturmak için kullanılan bir C++ kütüphanesi.  

3. Projeyi derlemek için bir C++ derleyicisine (örn. **GCC**) ve bir derleme sistemine (örn. **Make** veya **CMake**) ihtiyacınız olacak. Kendi platformunuz için standart derleme talimatlarını takip edin:  
   ```bash
   make
   ```  

## Kullanım

Projeyi derledikten sonra:

1. **ASSDraw** çalıştırılabilir dosyasını açın.  
2. Grafik arayüzü kullanarak şekiller oluşturun ve özelliklerini ayarlayın.  
3. Şekilleri, altyazı biçimlendirme amacıyla **ASS altyazı dosyasına** aktarın.  

## Bağımlılıklar
- **wxWidgets** (v3.0 veya üstü)  
- **AGG Kütüphanesi** (v2.5)  
- **C++11 veya daha yeni bir sürüm**  

## Kaynaktan Derleme

### Windows:
- **wxWidgets** ve **AGG** kütüphanelerini indirip yükleyin.  
- Derleyiciniz için uygun ortam değişkenlerini ayarlayın.  
- `make` komutunu çalıştırın veya IDE'nizin derleme araçlarını kullanın.  

### Linux/macOS:
- Gerekli paketleri paket yöneticisi aracılığıyla yükleyin:  
   ```bash
   sudo apt-get install libwxgtk3.0-dev libagg-dev
   ```  
- Derleme komutunu çalıştırın.  

## Yapılandırma

Proje, kütüphane yolları ve çıktı dosyalarını ayarlayan bir **Varsayılan Profil** kullanır. Ayarları değiştirmek için `Default Profile` yapılandırmasını düzenleyebilir veya yeni bir profil oluşturabilirsiniz.  

```plaintext
[Profil1]
Include Paths: /kütüphane/yolları
Output Paths: /çıktı/dosyaları
```

## Lisans

ASSDraw, **MIT Lisansı** altında lisanslanmıştır. Daha fazla bilgi için [LICENSE](LICENSE) dosyasına bakabilirsiniz.  

## Katkıda Bulunma

Projeye katkıda bulunmak istiyorsanız, depoyu **fork** edip değişikliklerinizi yaptıktan sonra bir **pull request** gönderebilirsiniz. Lütfen kodlama standartlarına uyduğunuzdan emin olun ve mümkün olduğunca testler ekleyin.  

## Teşekkürler
- **AGG** Kütüphanesi: Grafik işleme yetenekleri sağladığı için.  
- **wxWidgets**: Çapraz platform kullanıcı arayüzü desteği sunduğu için.ASSDraw
