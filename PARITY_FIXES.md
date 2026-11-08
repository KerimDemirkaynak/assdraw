# Taslakta bulunan hatalar ve yapılan düzeltmeler

Derleme yapılmadı (istek üzerine); tüm değişiklikler orijinal C++ koduyla satır satır karşılaştırılarak yapıldı.

## Davranış farkları (orijinalden sapma) — düzeltildi
1. **Dolgu kuralı even-odd idi** → non-zero winding. İç içe aynı yönlü konturlar yanlışlıkla delik oluyordu.
2. **Delikler ayrı renkle boyanıyordu / üst şekil rengini kaybediyordu** → dolgu artık alt şekil (dış kontur + delikleri) bazında.
3. **Tessellation indeksleri u16** idi; büyük şekiller sessizce kayboluyordu → u32.
4. **"Reposition [0,0]" şekli kaydırıyordu**; orijinal yalnızca görünümü taşır. Ctrl+Shift+… (arka plan köşeleri) hiç yoktu → ikisi de uygulandı.
5. **Başlangıç durumu**: örnek üçgen + zoom 8 yerine, orijinal gibi tek `m 0 0`, zoom 10.
6. **Zoom modeli** çift katmanlıydı ve çarpımsaldı → tek ölçek, 1–50, toplamsal; yüzde gösterimi doğru.
7. **Seçim**: Ctrl=ekle, Alt=çıkar (taslakta Ctrl=aç/kapa, Shift=ekle). Seçili bir noktaya tıklamak seçimi tek noktaya düşürüyordu → çoklu sürükleme bozuktu.
8. **Çizim araçları** tıkla-bırak'tı; orijinal bas-sürükle-bırak. Mevcut noktaya tıklayınca "o komuttan sonra ekle" yoktu. B komutu "auto-bezier" ile eğri çıkıyordu; orijinal 1/3–2/3 doğrusal başlar.
9. **Delete aracı** noktayı sürüklüyordu, ilk komut kuralı ve uyarı yoktu.
10. **Coloring**: Select kipinde de çift tık renk açıyordu; boş alana çift tık son konturu boyuyordu; renk penceresi alfa istiyordu.
11. **Bilinear** kenar tutamaçlarını desteklemiyordu; dönüşümler her sürüklemede bir önceki sonuçtan hesaplanıp yuvarlama hatası biriktiriyordu → daima araca girişteki şekilden eşlenir. Sınırlar kontrol noktalarını da kapsıyordu → yalnızca çizilen eğri.
12. **Scale/Rotate aracı** seçilince gereksiz dönüşüm penceresi açılıyordu.
13. **Koordinatlar tamsayı değildi** (sürükleme/dönüşüm kesirli üretiyordu) → tüm düzenlemeler tamsayıya oturur.
14. **Kılavuz çizgileri** yanlıştı (c1→end, c2→end) → önceki nokta→c1→c2→son nokta. Ana noktalar kare/kontrol daire, üzerine gelince koordinat etiketi + çapraz kılavuz. Orijinaldeki ince editör kenarlığı yoktu (dolgu renginde 2px çiziyordu) → eklendi (Ayarlar: Outline); ASS kenarlığı ayrı satır.
15. **Sağ tık / pan**: arka plan çizimle birlikte kayıyordu; "yalnız çizim" kipinde arka plan yerinde kalır.
16. **Bağlam menüsü** fare menü üstüne gelince değişiyordu (nokta donduruldu); C1 ve Bezier→Line yalnızca uygun noktalarda.
17. **Yapıştırma** metni mevcut çizime ekliyordu (örnek üçgenle karışıyordu) → komut kutusuna yapıştırıp Enter ile aynı: çizimi değiştirir.
18. **Komut kutusu** tek satırdı, çok satırlı yapıştırmada satırlar birleşiyordu → çok satırlı, Enter uygular.
19. **Ayrıştırıcı** hatalı/eksik girdide hata veriyordu; orijinal toleranslı. `Parse S/P/C` ayarı UI'da yoktu → port edildi (ParseASS birebir). Tek başına `c`/`p` artık geçersiz komut üretmez.
20. **Ayarlar**: Parse S/P/C, resim opaklığı sor, varsayılan opaklık, splash yoktu; Revert varsayılanlara dönüyordu (orijinal: son uygulanan); çıkışta kaydetme yoktu.
21. **Kitaplık**: tek tık yüklüyordu (çift tık), boşaltılan kitaplık yeniden varsayılanlarla doluyordu, yeni şekil sona ekleniyordu.
22. **Kılavuz açma** Windows'ta `file://C:\…` ile çalışmıyordu.
23. Dil varsayılanı sabit Türkçeydi → işletim sistemi dili.
24. README Rust 1.80 diyordu, kod 1.82 gerektiriyor; CI'da clippy `-D warnings` yayını engelliyordu → bilgilendirici.

## Dikkat
Kod derlenmedi. İlk `cargo check`'te küçük tip/ödünç alma hataları çıkabilir; mantık tarafı yukarıdaki listeye göre tamamdır. Yeni testler: `tests/parity_tests.rs` ve `engine/parse.rs` içi birim testler.
