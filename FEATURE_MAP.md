# ASSDraw3 → ASSDraw-rs özellik haritası

Durum: ✅ orijinalle aynı davranış · ➕ orijinalde olmayan ek · ⚠️ bilinen fark

| Orijinal ASSDraw3 özelliği | ASSDraw-rs | Not |
|---|---|---|
| Araçlar: Drag, M, L, B, Delete, Scale/Rotate, Bilinear, Coloring | ✅ | F1–F8 |
| M/L/B: basılı tutup sürükleyerek yerleştirme; mevcut noktaya tıklayınca o komuttan sonra ekleme | ✅ | B kontrol noktaları 1/3–2/3'e konur |
| Kontrol noktasına tıklayıp sürükleme (M/L/B kipinde) | ✅ | |
| Seçim: tıkla, Ctrl = ekle, Alt = çıkar, çerçeve seçimi, çoklu sürükleme | ✅ | |
| Shift basılıyken geçici sürükleme kipi | ✅ | |
| Delete aracı: bas-bırak ile komutu sil; ilk komut en son silinir | ✅ | uyarı mesajı gösterilir |
| Scale/Rotate: köşe + kenar tutamaçları, Shift ile oran, sağ tuşla döndürme | ✅ | şekil her zaman araca girişteki hâlinden eşlenir (birikim hatası yok) |
| Bilinear: köşe ve **kenar** tutamaçları | ✅ | |
| Coloring: tek tık alt şekli seçer, çift tık renk penceresi; delikler üst şekle bağlı | ✅ | |
| Dolgu kuralı: non-zero winding (AGG / libass) | ✅ | |
| Sürükleyerek [0,0] orijinini taşıma, sağ menüde "Move [0,0] here" | ✅ | |
| Sağ tuşla kaydırma, tekerlekle zoom (1–50, ±1), PageUp/PageDown, ok tuşları (1 / Ctrl 10) | ✅ | zoom imleç altında kalır |
| Reposition [0,0]: tuval köşeleri/merkez (Ctrl+1/3/5/7/9) ve arka plan köşeleri (Ctrl+Shift+…) | ✅ | görünümü kaydırır, çizime dokunmaz |
| Tab: noktadan noktaya imleç atlatma | ✅ | |
| Bezier ↔ Line dönüşümü, C1 "smooth connection" | ✅ | yalnızca uygun noktalarda menüde çıkar |
| Komut kutusu (Enter ile uygula), ASS kopyala, çok satırlı yapıştırma | ✅ | |
| ParseASS: toleranslı ayrıştırma, `{}` blokları, greedy l/b, eksik komutlar yok sayılır | ✅ | ondalıklar yuvarlanır (orijinal yok sayardı) |
| "Parse S/P/C" ayarı | ✅ | varsayılan açık (orijinal: kapalı) |
| Çok renkli ASS (`\1c \1a \3c \3a \bord`), x-kaydırma telafisi | ✅ | |
| Matris dönüşümü penceresi + şablonlar | ✅ | |
| Arka plan resmi: dosya, panodan yapıştırma, sürükle-bırak, opaklık, Shift+Del | ✅ | |
| Çizim / arka plan / ikisi birden kaydırma kipi (Shift+F1/F2/F3) | ✅ | |
| Önizleme kipi (Ctrl+P) | ✅ | |
| Undo/redo (açıklamalı) | ✅ | |
| Şekil kitaplığı: kaydet, çift tıkla yükle, kopyala, üzerine yaz, sil | ✅ | yeni şekil en üste eklenir |
| Ayarlar: tüm renkler/opaklıklar, Parse S/P/C, resim opaklığı, splash | ✅ | Revert son uygulanan değerlere döner |
| 7 dil, açık/koyu/sistem teması | ✅ | dil varsayılanı işletim sisteminden |
| Kılavuz | ⚠️ | HTML kılavuz (CHM değil) |
| N / S / P / C araçları | ➕ | N açık kontur; S/P/C yalnızca spline sonrası geçerli |
| AGG interpolasyonlu spline çizimi | ⚠️ | uniform kübik B-spline (libass'in gerçek çıktısına daha yakın) |
