# Diğer ASSDraw sürümlerinden aktarılan özellikler

Başlangıç sürümü: `KerimDemirkaynak/assdraw@a988b83d70d27df89b434b1aee3f46f52f3e9548`.

| Kaynak | Eklenen veya uyarlanan özellik |
| --- | --- |
| amanosatosi | Dolu alt şekli seçip çift tıklayarak ayrı renklendirme; iç boşlukların dış şekle bağlı kalması |
| amanosatosi | Dolgu, kenarlık ve saydamlık etiketleri içeren ASS metnini içe/dışa aktarma; ayrı renklerdeki parçaların konum telafisi |
| amanosatosi | Komut alanına Copy ASS düğmesi; Enter ile uygulama; çok satırlı komutların doğru ayrıştırılması |
| amanosatosi | Arka plan görselini ve görünümü değiştirmeyen çizim geri alma/yineleme geçmişi |
| amanosatosi | Uzaklaştırmada kullanılabilir büyüklükte kalan kontrol noktaları ve seçim alanları |
| amanosatosi | Araç çubuğu ve menüdeki çizim/arka plan kaydırma düğmelerinin olay bağlantıları |
| biki-desu | Ölçekleme/döndürme ve bilineer dönüşümde yanlış çizim yolu işaretçisinin düzeltilmesi |
| biki-desu | Henüz tamamlanmamış Bézier komutunun yanlış switch dalına geçmesinin önlenmesi |
| biki-desu | Özellik panelinde güvenli renk okuma; dönüşüm penceresinin 4×4 yerleşimi |
| biki-desu | Varsayılan arka plan opaklığı ayarı; Linux yakınlaştırma çubuğunun görünür genişliği |
| biki-desu | Hakkında penceresinin ekranda ortalanması; wxGTK bitmap yenileme düzeltmesinin uyarlanması |

Entegrasyon sırasında ayrıca:

- Mevcut yüksek DPI desteği, bitmap bundle kullanımı, Windows kurulum akışı ve
  i18n otomasyon betiği korundu. Nokta çizimi ve seçim yarıçapı birlikte DPI'a uyarlandı.
- Genel görünüm ayarlarının bağımsız alt şekil renklerini ezmesi önlendi.
- Kitaplığa kaydederken, kopyalarken ve yeniden yüklerken renkli ASS metni
  korundu. Kitaplıktaki önizlemeler mevcut tek renkli görünümlerini kullanır.
- Türkçe gibi ondalık virgül kullanan yerel ayarlarda kenarlık genişliğinin
  ASS'e noktalı ondalık olarak yazılması sağlandı.
- Büyük harfli komut seçeneği yalnızca çizim komutlarına uygulanır; ASS'in
  küçük harfli stil etiketleri bozulmaz. Renk/alfa değerleri büyük harfle yazılır.
- Her yeniden çizimde oluşturulan AGG yollarının tamamı serbest bırakılır.
- AGG yerelde bulunmazsa CMake sabitlenmiş sürümü indirir. Kullanılmayan ve
  manifestte tanımlı olmayan doğrudan WebP bağlantısı kaldırıldı.
- Linux derlemesi ve CTest testleri Windows iş akışına ek olarak eklendi.

## Doğrulama

- GCC 13.3 ve wxWidgets 3.2.4 ile Linux uygulaması ve arayüz test programı derlendi.
- CTest temel renk/geçmiş ve kaydırma bağlantısı testleri geçti.
- Temel testler AddressSanitizer ve UndefinedBehaviorSanitizer ile de geçti;
  bu ortamda LeakSanitizer çalıştırılamadığı için sızıntı algılama kapatıldı.
- Arayüz testleri derlendi ancak çalışma ortamı X sunucusu için dinleme
  soketi oluşturamadığından çalıştırılamadı. Bu nedenle arayüz davranışları
  otomatik çalışma testiyle doğrulanmış sayılmaz.
- Windows derlemesi bu ortamda çalıştırılmadı. İş akışında uygulama, temel
  testler, arayüz testleri ve mevcut Inno Setup kurulum adımları bulunur.

## Kaynak sürümleri

- [amanosatosi/assdraw](https://github.com/amanosatosi/assdraw/tree/f6c5447c5400f7f9ed13e9c167c45d7d49140fb4)
- [biki-desu/assdraw](https://github.com/biki-desu/assdraw/tree/633a9f2c94436944654474afc98ed3db6ec5b241)

Kaynak atıfları ve lisans notları için `NOTICE.md` dosyasına bakın.
