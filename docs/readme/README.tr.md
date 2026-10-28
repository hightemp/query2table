<p align="center">
  <img src="../../images/query2table-logo.png" alt="Query2Table logosu" width="160">
</p>

<h1 align="center">Query2Table</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="README.ru.md">Русский</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português (Brasil)</a> · <a href="README.pl.md">Polski</a> · <a href="README.uk.md">Українська</a> · <b>Türkçe</b> · <a href="README.zh.md">简体中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a></p>

<p align="center">
  <a href="https://github.com/hightemp/query2table/releases/latest"><img src="https://img.shields.io/github/v/release/hightemp/query2table?style=flat-square" alt="GitHub sürümü"></a>
  <a href="https://github.com/hightemp/query2table/releases"><img src="https://img.shields.io/github/downloads/hightemp/query2table/total?style=flat-square" alt="GitHub indirmeleri"></a>
  <a href="#lisans"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="Lisans: MIT"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8D8?style=flat-square&amp;logo=tauri" alt="Tauri v2 ile geliştirildi"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust-orange?style=flat-square&amp;logo=rust" alt="Rust arka uç"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&amp;logo=svelte" alt="Svelte 5 ön uç"></a>
  <br>
  <img src="https://asdertasd.site/counter/query2table" alt="Query2Table görüntülenmeleri">
</p>

**Query2Table**, masaüstü (Linux, Windows, macOS) ve Android için yerel öncelikli bir araştırma aracıdır. Hangi verilere ihtiyacınız olduğunu tarif edersiniz; uygulama interneti birden çok kaynakta ve dilde kendi başına arar, sayfaları indirir, LLM’lerle yapılandırılmış varlıkları çıkarır, yinelenenleri ayıklar ve sonucu satır düzeyinde kaynak bağlantılarıyla canlı güncellenen bir tablo olarak sunar. Hepsi bulut arka ucu olmadan kendi bilgisayarınızda çalışır.

Tabloların yanı sıra üç ayrı mod daha sunar: ilgili görselleri bulup LLM ile görsel ilgililiğe göre sıralayan **görsel arama**; aday sayfaların tam içeriğini okuyup yalnızca en ilgili bağlantıları LLM tarafından yazılmış açıklamalar ve ilgililik puanlarıyla döndüren **bağlantı arama**; ve modelin kendi başına arama yaptığı, sayfa okuduğu, akıl yürüttüğü ve kaynaklı bir Markdown yanıtı yazdığı ajan tabanlı **araştırma** modu.

Uygulama **Android** telefonlar için de mevcuttur: aynı modlar, ayarlar ve geçmişe sahip aynı uygulama, aynı koddan derlenir ve alt sekme çubuğuyla telefon ekranına uyarlanmıştır.

*"Açık kaynak vektör arama veritabanları ve kütüphanelerinin dil, lisans, dağıtım türü, kullanım alanı ve web sitesiyle bir karşılaştırma tablosunu oluştur"* gibi bir şey sorun ve tablonun gerçek zamanlı olarak dolmasını izleyin.

![Kanıt ve kaynaklarla açık kaynak vektör veritabanlarının karşılaştırma tablosu](../../screenshots/test-en-2026-10-06/44-table-results.png)

## Tur

### Gündelik dille sorun

Bir sonuç biçimi seçin, neye ihtiyacınız olduğunu tarif edin ve durdurma koşullarını ayarlayın. Tablolar için uygulama, arama başlamadan önce düzenleyebileceğiniz sütunlar önerir.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/01-query-table.png" alt="Dört sonuç biçimiyle yeni sorgu"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/05-schema-review.png" alt="İncelemeye hazır önerilen tablo şeması"></td>
  </tr>
</table>

### Her satır kanıtını saklar

Tam değerleri okumak, kopyalamak ve çıkarıldıkları sayfaları denetlemek için bir satırı açın.

![Çıkarma güvenilirliği ve kaynaklarla satır ayrıntıları](../../screenshots/test-en-2026-10-06/45-row-details-and-sources.png)

### Ek sorularla araştırma

Ajan arama yapar, sayfaları okur ve içindekiler tablosuyla kaynaklı bir yanıt yazar. Her adım Etkinlik’te görünür; ek sorular konuşmayı geçmişiyle birlikte sürdürür.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/21-research-answer.png" alt="İçindekiler ve önerilen ek sorularla araştırma yanıtı"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/22-research-activity.png" alt="Numaralı arama ve okuma adımlarıyla araştırma etkinliği"></td>
  </tr>
</table>

### Görseller ve bağlantılar

Görsel arama, sonuçları ilgililiğe göre sıralar ve site ile boyuta göre filtreler sunar; bağlantı arama her sayfayı okur ve neden eşleştiğini açıklar.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/28-images-grid.png" alt="İki yana yaslanmış ızgarada görsel arama sonuçları"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/32-links-results.png" alt="Açıklamalar ve eşleşme düzeyleriyle ilgili bağlantılar"></td>
  </tr>
</table>

### Geçmiş ve ayarlar

Geçmiş çalıştırmaları arayın, filtreleyin ve yeniden açın, yeniden çalıştırın veya birkaçını birden dışa aktarın. Ayarlar, başlamadan önce LLM ve arama bağlantılarınızı denetler.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/18-history.png" alt="Sonuçlar, süre, maliyet ve modelle güne göre gruplanmış çalıştırma geçmişi"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/43-settings-search-connection.png" alt="Başarılı bağlantı sınamasıyla arama ayarları"></td>
  </tr>
</table>

### Açık ve koyu temalar

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/53-research-dark.png" alt="Koyu temada araştırma yanıtı"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/50-table-dark.png" alt="Koyu temada sonuç tablosu"></td>
  </tr>
</table>

## İndirme

En son sürümü [Releases](https://github.com/hightemp/query2table/releases/latest) sayfasından edinin:

| Platform | Dosya |
|----------|-------|
| Linux | `.AppImage`, `.deb`, `.rpm` |
| Windows | `.msi`, `.exe` |
| macOS (Apple Silicon) | `.dmg` |
| Android 7.0+ | `Query2Table_<version>_android.apk` |

Kurulumunuzu bilgisayardan telefona taşımak için **Ayarlar → Ayarları dışa aktar…** seçeneğini kullanın, **API anahtarlarını ve proxy’leri dahil et** kutusunu işaretleyin ve dosyayı telefonda içe aktarın. Böyle bir dosya anahtarlarınızı içerir: gizli tutun ve içe aktardıktan sonra silin.

Android’de indirilen APK’yı açın ve sorulduğunda tarayıcınızdan veya dosya yöneticinizden uygulama yüklemeye izin verin. Android uygulaması verilerini telefonda tutar; sistem tepsisi yoktur, bildirimler normal Android bildirimleri olarak görünür. Yerel model sunucuları (Ollama, llama.cpp) ağınız üzerinden adresleriyle kullanılabilir, ör. `http://192.168.1.10:11434`.

## Özellikler

- **Doğal dilde sorgular** — ne istediğinizi gündelik dille tarif edin
- **Otomatik şema çıkarımı** — uygulama tablo sütunları önerir; çalıştırmadan önce onaylar veya düzenlersiniz
- **Çok sağlayıcılı arama** — otomatik yedeklemeyle Brave Search + Serper
- **Çok dilli sorgu genişletme** — farklı diller ve coğrafyalarda arama
- **LLM destekli çıkarma** — OpenRouter, Anthropic Claude (istem önbelleği, isteğe bağlı bütün PDF okuma), Ollama (yerel veya Cloud), DeepSeek, Groq, Mistral, Gemini, xAI, OpenAI, LM Studio, llama.cpp ya da herhangi bir OpenAI uyumlu adres
- **Akışlı sonuçlar** — satırlar bulundukça canlı tabloda görünür
- **PDF çıkarma** — arama sonuçlarında bulunan PDF belgelerini otomatik algılar ve metinlerini çıkarır
- **Görsel arama modu** — metin yerine görsel arayan ayrı bir işlem hattı: 6–12 farklı sorgu varyasyonu üretir (LLM veya statik yedekler), Brave / Serper üzerinden görsel aramaları yürütür, ardından her sonucu katı bir ilgililik ölçütüyle (0.0–1.0) LLM ile sıralar; 0.7’nin altında puan alan görseller elenir, kalanlar puana göre sıralanarak saklanır ve arayüz her görsel için gerçek zamanlı güncellenir; bütçe, iptal ve durdurma koşulları (sayı / maliyet / süre) baştan sona gözetilir
- **Bağlantı arama modu** — tablo yerine en ilgili sayfaları döndüren ayrı bir işlem hattı: isteğinizden arama sorgusu varyasyonları üretir, Brave / Serper üzerinden web aramaları yürütür (URL’ye göre yinelenenler ayıklanır), her sayfayı indirip ayrıştırır, ardından bir LLM tam içeriği okuyup sorgunuza ilgililiğini (0.0–1.0) puanlar ve kısa bir açıklama yazar; güven eşiğinin altındaki sayfalar elenir, kalanlar puana göre sıralanarak saklanır ve arayüz her bağlantı için gerçek zamanlı güncellenir; bütçe, iptal ve durdurma koşulları baştan sona gözetilir
- **Araştırma modu** — açık uçlu soruları tablo yerine Markdown ile yanıtlayan ajan tabanlı bir mod: LLM, belirli bir adım sayısına kadar bir araç çağırma döngüsünü yönetir (web’de ara, bir sayfayı Markdown olarak indir, düşün, yanıtla) ve her adımı canlı bir zaman çizelgesine aktarır; süreç şeffaf kalsın diye akıl yürütme (`think`) adımları ve hatalar (kırmızıyla gösterilen `error` adımları) görünür kılınır, kaynaklı son Markdown yanıtı (temizlenmiş olarak) gösterilir ve `.md` olarak dışa aktarılabilir; bütçe, iptal ve durdurma koşulları baştan sona gözetilir
- **Araştırma yanıtlarında formüller ve resimler** — LaTeX formülleri çizilir (MathJax, çevrimdışı); ajan görsel arayabilir, onlara bakabilir ve uygun olanları yanıtına yerleştirebilir; bunlar çalıştırmayla birlikte saklanır
- **Araştırma yanıtlarında diyagramlar ve grafikler** — Mermaid diyagramları ile çubuk, çizgi, alan, pasta, dağılım, radar ve ısı haritası grafikleri, uygulama temasında çevrimdışı çizilir; SVG veya PNG olarak kaydedilir, tam ekran açılır
- **Araştırma yanıtlarında videolar** — ajan her sitede video bulur (web video araması ya da doğrudan YouTube, Bilibili, Niconico ve SoundCloud), altyazılarını zaman damgalarıyla okur ve belirli anlara bağlantı verir; videolar saklanan küçük resimleriyle kart olarak görünür. İlk kullanımda indirilen yt-dlp ile çalışır (Android'de yerleşik Python ile)
- **Kullanım istatistikleri** — ay, yıl ve tüm zamanlar için para, token, model çağrıları, Brave ve Serper aramaları ve veri hacmi; modele ve moda göre dökümler, en pahalı çalıştırmalar, disk kullanımı, CSV dışa aktarma ve uyarılı aylık bütçe
- **13 arayüz dili** — İngilizce, Rusça, Almanca, Fransızca, İspanyolca, İtalyanca, Portekizce (Brezilya), Lehçe, Ukraynaca, Türkçe, Çince (basitleştirilmiş), Japonca ve Korece; varsayılan olarak sistem dili seçilir
- **Satır düzeyinde kaynaklar** — her satır, çıkarıldığı sayfalara bağlantı verir
- **Varlık yinelemelerini ayıklama** — bulanık eşleştirme + LLM destekli ayrıştırma
- **Ayarlanabilir durdurma koşulları** — hedef satır sayısı, en yüksek maliyet, en uzun süre
- **Çalıştırma geçmişi** — geçmiş araştırma çalıştırmalarına göz atın, görüntüleyin ve yeniden dışa aktarın
- **Dışa aktarma** — tam kaynak meta verileriyle CSV, JSON, XLSX
- **Koyu / Açık tema** — kenar çubuğundan değiştirilir
- **Sistem tepsisi** — tamamlanma bildirimleri (masaüstü)
- **Android uygulaması** — küçük ekranlara uygun düzeniyle telefonlarda aynı uygulama
- **Uygulama içi güncellemeler** — yeni sürümler değişiklikleriyle duyurulur ve imzaları denetlendikten sonra uygulamadan kurulur (masaüstü ve Android)
- **Yerel öncelikli** — tüm veriler SQLite’ta, bulut arka ucu gerekmez

## Teknoloji yığını

| Katman | Teknoloji |
|--------|-----------|
| Uygulama kabuğu | Tauri v2 (masaüstü ve Android) |
| Arka uç | Rust (Tokio asenkron çalışma zamanı) |
| Ön uç | Svelte 5 (SvelteKit SPA) |
| Arayüz stili | Tasarım belirteçleriyle düz CSS |
| Veritabanı | SQLite (sqlx, WAL modu) |
| LLM | OpenRouter / Ollama (yerel) / Ollama Cloud / OpenAI uyumlu API (ör. llama.cpp) |
| Arama | Brave Search API / Serper API |
| Simgeler | Lucide |

## Gereksinimler

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) (kararlı araç zinciri)
- İşletim sisteminiz için [Tauri v2 gereksinimleri](https://v2.tauri.app/start/prerequisites/)

## Başlarken

```bash
# Clone the repository
git clone https://github.com/hightemp/query2table.git
cd query2table

# Install frontend dependencies
npm install

# Run in development mode (starts both Vite dev server and Tauri)
npm run tauri dev

# Build for production
npm run tauri build
```

## Yapılandırma

İlk açılışta uygulama varsayılan ayarlarla yerel bir SQLite veritabanı oluşturur. Yapılandırmak için kenar çubuğundan **Ayarlar**’ı açın:

| Grup | Ayarlar |
|------|---------|
| **LLM sağlayıcısı** | Sağlayıcı (OpenRouter / Ollama / Ollama Cloud / OpenAI uyumlu), URL, model, API anahtarı, JSON modu, sıcaklık, en fazla token |
| **Arama sağlayıcısı** | Sağlayıcı (Brave / Serper), API anahtarı, yedek sağlayıcı anahtarı, sorgu başına sonuç |
| **Yürütme** | Paralel indirmeler (varsayılan 8), paralel çıkarmalar (varsayılan 3), indirme zaman aşımı, hız sınırlama, robots.txt |
| **Kalite** | Kesinlik/duyarlılık dengesi, kanıt katılığı, güven eşiği, yineleme benzerliği |

Durdurma koşulları (hedef satır, en yüksek maliyet, en uzun süre) Sorgu sayfasında her sorgu için ayrı ayarlanır.

**Yedek arama sağlayıcısını kullan**, hem web hem görsel aramalarında otomatik Brave/Serper yedeklemesini denetler. Her iki API anahtarı da kayıtlı olsa bile yalnızca seçili birincil sağlayıcıyı kullanmak için kapatın. Seçim, Kaydet’ten sonra yeni çalıştırmalara uygulanır; yeniden denemeler seçili sağlayıcıda kalır. Açıkken diğer sağlayıcı yalnızca bir hatadan sonra kullanılır ve kayıtlı anahtarını gerektirir.

Araştırma dahil dört modun hepsinde her arama sorgusunun kaç sonuç isteyeceğini seçmek için **Ayarlar → Arama → Sorgu başına sonuç** değerini ayarlayın, ardından yeni çalıştırmalar için kaydedin. Brave istekleri bu değeri belgelenmiş uç nokta aralığına sıkıştırır: [web araması için 1–20](https://api-dashboard.search.brave.com/api-reference/web/search/get) ve [görseller için en fazla 200](https://api-dashboard.search.brave.com/documentation/services/image-search). Bu, daha önce kaydedilmiş fazla büyük değerleri de kapsar. Daha büyük değerler otomatik olarak ek sayfalar istemez.

### Kullanım, fiyatlar ve harcama sınırları

**Kullanım ve maliyet**, sağlayıcının bildirdiği ücretleri, model ücretine dayalı tahminleri, kısmi toplamları ve bilinmeyen maliyetleri birbirinden ayırır. Yeniden denemeler, yedek aramalar, kullanım verisi olan başarısız tamamlamalar ve son ücreti alınamayan iptal edilmiş istekler dahil her sağlayıcı denemesi ayrı sayılır. Girdi/çıktı kullanımı sağlayıcı yanıtlarından gelir; düşünme tokenları çıktı tokenlarının üstüne ikinci kez ücretlendirilmez.

- **OpenRouter:** [bildirilen istek maliyeti](https://openrouter.ai/docs/cookbook/administration/usage-accounting) tercih edilir. Yoksa bildirilen tokenlardan ve gerçekte döndürülen modelin herkese açık katalog ücretlerinden tahmin edilir.
- **Ollama Cloud:** kullanım kredisi tüketimi, önbellekli girdi ve tanınan yoğun saat fiyatlandırma dönemleri dahil [yayımlanmış model ücretlerinden](https://ollama.com/pricing) tahmin edilir. Dahil krediler, abonelik ödemeleri ve nakit ücretler bu tahminden farklı olabilir. Tanınmayan veya bilinmeyen tarifeler bilinmiyor olarak kalır.
- **Özel model ücretleri:** milyon token başına girdi/çıktı USD değerlerini Ayarlar’da yapılandırın. Ücretler belirli bir sağlayıcıya, uç noktaya ve modele aittir. Açıkça girilen sıfır ücret ücretsiz kullanım demektir; eksik ücretler hiçbir zaman sessizce sıfır olmaz. Yayımlanmış ücretler, kaydedilmiş elle girilmiş ücretler değiştirilmeden alınır.
- **Arama:** Brave ve Serper için varsayılan fiyat 1.000 istek başına 0 USD’dir; bu, arama ücretlerini tahminlerin dışında tutar. Bunları dahil etmek için planınızın gerçek ücretini girin veya bilinmiyorsa fiyatı boş bırakın. Faturalama verisi olmayan başarısız denemeler fiyatsız kalır. Ayar değişiklikleri yeni çalıştırmalara uygulanır; kayıtlı geçmiş yeniden hesaplanmaz. Yükseltmede eski boş varsayılanlar bir kez sıfıra dönüşür; mevcut sayısal ücretler ve sonradan bilinçli olarak boş bırakılanlar korunur.

Çalıştırmanın **En yüksek maliyet** değeri, hesaplanan harcama sınıra ulaştığında yeni sağlayıcı isteklerini engeller. Bu, garantili bir fatura tavanı değildir: tahminler faturalamadan farklı olabilir, bilinmeyen ücretler dahil edilemez ve hâlihazırda süren istekler yine de ücretlendirilebilir. Kesin bir tavan gerektiğinde sağlayıcı tarafındaki sınırları kullanın. Hesaplama; tamamlanan, başarısız olan, iptal edilen ve boş sonuçlu çalıştırmalarda korunur. Önceki geçmiş kayıtları açıkça geçmiş tahmin olarak etiketlenir.

Yineleme ayıklama, ilk üyenin satır kimliğini korur, grubun değerlerini ve kaynak kayıtlarını tek bir işlemde birleştirir ve canlı tabloyu geçmiş ve dışa aktarmayla uyumlu hâle getirir. Önceden üzerine yazılmış geçmiş sonuçlar otomatik olarak yeniden oluşturulamaz; güncel sonuçlar için bu sorguları yeniden çalıştırın.

Ayarlar, kaydırırken **Kaydet** ve **Vazgeç** düğmelerini görünür tutar. Bölüm bağlantıları doğrudan ilgili denetimlere atlar; kaydedilmemiş değişikliklerle ayrılırken Kaydet / Vazgeç / Kal seçenekleri sunulur. Kaydetme sırasında yapılan düzenlemeler bir sonraki kaydetme için kullanılabilir kalır.

Sonuçlar, yerel arama ve klavyeyle sıralama sunan sürekli, sanallaştırılmış bir tablo kullanır. Tam değerleri okumak veya kopyalamak ve kayıtlı kaynaklarını incelemek için bir satırın **Ayrıntılar**’ını açın. Araştırma, yanıtı genişletilebilir etkinliğin önüne koyar. Görseller ızgara/liste görünümlerini ve klavyeyle önizlemeyi destekler; dışa aktarma ve silme ilerlemeyi ve sonucu açıkça gösterir. Tanılama amaçlı **Günlükler** panelini temizlemek çalıştırma etkinliğini sıfırlamaz.

Tarayıcı regresyon denetimleri yapay veriler ve taklit Tauri komutları kullanır: test tarayıcılarını `npx playwright install chromium webkit` ile kurun, ardından `npm run test:ui` komutunu çalıştırın. Mevcut ön uç testleri `npm test` ile çalışır.

### API anahtarları

En az bir arama API anahtarına ve yapılandırılmış bir LLM sağlayıcısına ihtiyacınız var. Yerel Ollama ve kimlik doğrulaması olmayan OpenAI uyumlu sunucular LLM API anahtarı gerektirmez:

| Hizmet | Anahtarı alacağınız yer |
|--------|-------------------------|
| Brave Search | https://brave.com/search/api/ |
| Serper | https://serper.dev/ |
| OpenRouter | https://openrouter.ai/ |
| Ollama (yerel) | https://ollama.com/ — anahtar gerekmez |
| Ollama Cloud | https://ollama.com/settings/keys |
| OpenAI uyumlu | Sunucunuza bağlı olarak isteğe bağlı anahtar |

Sağlayıcıyı **Ayarlar → LLM sağlayıcısı** bölümünde seçin, alanlarını doldurun ve **Kaydet**’e tıklayın. Sağlayıcı değiştirdiğinizde her sağlayıcı kendi modelini ve kimlik bilgilerini korur.

Ayar taşımaları başlangıçta otomatik olarak çalışır: yeni eklenen seçenekler varsayılan değerleri alır, mevcut değerler korunur. Yeniden adlandırılan ayarlar, varsayılanlar eklenmeden önce arka uçta açıkça taşınır; kayıtlı biçimlerdeki değişiklikler de yeni bir sürüm geliştirilirken taşıma kodu gerektirir.

**Ayarlar**’ın en altında **Uygulama dosyaları**, gerçek veri klasörünü, `data.db` veritabanını ve günlük klasörünü gösterir. Bir konumu sistem panosuna kopyalamak için **Yolu kopyala**’yı, dosya yöneticinizde açmak için **Klasörü aç**’ı kullanın. Veritabanı konumunu açmak, onu içeren klasörü açar. Ayarlar, geçmiş, sonuçlar ve kaynaklar `data.db` içinde bulunur; tanılama günlükleri ayrı saklanır.

- **OpenRouter:** API anahtarınızı girin ve aranabilir [OpenRouter kataloğundan](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties) bir model seçin. Model kimliklerini filtrelemek için yazın, fareyle ya da ok tuşları ve Enter ile seçin; listeyi yeniden yüklemek için **Yenile**’yi kullanın. Katalog, anahtar girmeden önce de incelenebilir; çıkarım için anahtar gerekir.
- **Ollama (yerel):** URL `http://localhost:11434` ve kurulu bir modelin adı.
- **Ollama Cloud:** URL `https://ollama.com`, bir Ollama API anahtarı ve aranabilir sunucu kataloğundan seçilmiş bir model. Model alanına tıklayıp filtrelemek için yazın, ardından fareyle ya da ok tuşları ve Enter ile seçin; **Yenile** kataloğu yeniden yükler. Filtreleme, bir sonuç seçene kadar seçili modeli değiştirmez. Doğrudan bulut erişimi [yerel Ollama API](https://docs.ollama.com/cloud)’sini kullanır. Bulut şu anda [yapılandırılmış çıktı desteği](https://docs.ollama.com/capabilities/structured-outputs) sunmadığından JSON istemler aracılığıyla istenir ve mevcut işlem hattı tarafından ayrıştırılır.
- **OpenAI uyumlu:** `/v1` dahil API temel URL’sini ve sunucunun model kimliğini girin. [llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server) için varsayılan URL `http://localhost:8080/v1`’dir; yüklü model adını veya yapılandırılmış takma adı kullanın. Sunucunuz gerektirmedikçe API anahtarını boş bırakın. Sunucu `response_format` seçeneğini reddederse **JSON modu**’nu kapatın.

**Düşünme / akıl yürütme düzeyi**, modelin yanıtından önceki akıl yürütmesini denetler. **Otomatik** güvenli varsayılanları korur: yapılandırılmış Ollama isteklerinde düşünme kapatılır (GPT-OSS Düşük kullanır), diğer sağlayıcılar varsayılanlarını korur. **Sağlayıcı varsayılanı** akıl yürütmeyi sunucuya bırakır. **Kapalı**, **Açık**, **Düşük**, **Orta**, **Yüksek** ve **En yüksek** belirli bir mod veya düzey ister; destek modele ve sağlayıcıya bağlıdır. Ollama’daki GPT-OSS Düşük/Orta/Yüksek kabul eder, Kapalı veya En yüksek kullanamaz. Değişiklikler Kaydet’ten sonra yeni çalıştırmalara uygulanır.

**En fazla çıktı tokenı**, istek başına bir paydır ve düşünme ile yanıt tokenları arasında paylaşılabilir. Çalıştırmanın maliyet/süre sınırlarından ve sağlayıcı hesap kotasından ayrıdır. Daha yüksek akıl yürütme düzeyi yanıta daha az token bırakabilir.

Model isteği sorunları; sağlayıcıyı, modeli, aşamayı, istenen çıktı üst sınırını, bildirilen girdi/çıktı/düşünme token sayılarını ve yeniden deneme planlanıp planlanmadığını gösterir. Çıktı sınırları, bağlam sınırları, sağlayıcı kotaları, hız sınırları, kimlik doğrulama ve ağ hatalarının her birinin ayrı açıklaması ve düzeltici eylemleri vardır. Tek tek sayfalar atlandığında sorunlar görünür kalır ve Çalıştırma Geçmişi’nde bulunur. Diğer hatalar kısa bir açıklama, yapılacak bir eylem ve daraltılabilir teknik ayrıntılar gösterir.

**Duraklat**, şema incelemesi dahil işlemeyi geçerli aşamada askıya alır; **Sürdür** isteği yeniden göndermeden devam eder. Uzak bir modele hâlihazırda gönderilmiş bir istek, duraklatılmışken orada yine de tamamlanabilir. **İptal**, yerel işi durdurur ve bekleyen istekleri kapatır. İşlem hattı hataları `failed` olarak gösterilir ve kaydedilir; asıl hata çalıştırma günlüğünde bulunur.

## Mimari

Arka uç, sırayla düzenlenen sabit rollere sahip bir **işlem hattı durum makinesi** kullanır:

```
Query → Interpret → Plan Schema → [User Confirms] → Plan Searches
      → Expand Queries → Execute Search → Fetch Pages → Parse Documents
      → Extract Entities → Validate → Deduplicate → [Stop Check] → Done
```

### İşlem hattı rolleri

| Rol | LLM | Amaç |
|-----|-----|------|
| QueryInterpreter | Evet | Sorguyu yapılandırılmış bir amaca dönüştürür |
| SchemaPlanner | Evet | Tablo sütunlarını ve türlerini önerir |
| SearchPlanner | Evet | Birden çok dil/bölge için arama sorguları üretir |
| QueryExpander | Evet | Sorguları hedef dillere çevirir |
| SearchExecutor | Hayır | Arama API’lerini çağırır, aday URL’leri toplar |
| Fetcher | Hayır | Hız sınırlama ve robots.txt ile HTTP indirme |
| DocumentParser | Hayır | HTML → temiz metin (kalıp içeriği kaldırma) |
| PdfParser | Hayır | PDF → temiz metin (pdf-extract ile) |
| Extractor | Evet | Metin + şema → yapılandırılmış satırlar |
| Validator | Kısmen | Şemaya uygunluk + anlamsal denetimler |
| Deduplicator | Kısmen | Bulanık eşleştirme (strsim) + uç durumlar için LLM |
| LinkRanker | Evet | Sayfa içeriğini okur ve ilgililiği puanlar (0.0–1.0) + açıklama üretir (bağlantı arama modu) |
| ResearchAgent | Evet | Ajan tabanlı arama/indirme/düşünme/yanıt döngüsünü yürütür ve Markdown yanıtı üretir (araştırma modu) |
| StoppingController | Hayır | Durdurma koşullarını değerlendirir (satırlar, bütçe, süre, doygunluk) |

### Durum makinesi

```
Pending → SchemaReview → Running ⇄ Paused → Completed / Failed / Cancelled
```

## Proje yapısı

```
query2table/
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── main.rs          # Tauri entry point
│   │   ├── commands/        # IPC command handlers
│   │   ├── orchestrator/    # Pipeline state machine, budget tracker
│   │   ├── roles/           # Pipeline & research-agent roles
│   │   ├── providers/       # External API clients (LLM, search, HTTP)
│   │   ├── storage/         # SQLite models & repository
│   │   └── export/          # CSV, JSON, XLSX export
│   └── migrations/          # SQLite schema migrations
├── src/                     # Svelte frontend
│   ├── lib/
│   │   ├── components/      # UI components
│   │   ├── stores/          # Svelte stores (run, settings, ui, logs)
│   │   ├── types/           # TypeScript type definitions
│   │   └── api/             # Tauri invoke/listen wrappers
│   └── routes/              # Pages (query, history, settings)
├── package.json
└── src-tauri/Cargo.toml
```

## Veritabanı

Aşağıdaki tablolarla WAL modunda SQLite:

| Tablo | Amaç |
|-------|------|
| `settings` | Kullanıcı yapılandırması (API anahtarları, modeller, tercihler) |
| `runs` | Durum ve istatistiklerle araştırma çalıştırması kayıtları |
| `run_schemas` | Her çalıştırma için onaylanmış tablo şeması |
| `search_queries` | Her çalıştırma için üretilen arama sorguları |
| `search_results` | Arama API’lerinden toplanan URL’ler |
| `fetched_pages` | İndirilen sayfa içeriği |
| `entity_rows` | Çıkarılan yapılandırılmış satırlar (JSON verisi) |
| `row_sources` | Satır düzeyinde kanıt (URL, başlık, alıntı) |
| `image_results` | Her çalıştırma için sıralanmış görseller (görsel arama modu) |
| `link_results` | Açıklama ve puanlarıyla sıralanmış ilgili bağlantılar (bağlantı arama modu) |
| `research_steps` | Her çalıştırma için ajan araç çağrısı adımları — search/fetch/think/error (araştırma modu) |
| `research_results` | Her çalıştırma için son Markdown yanıtı (araştırma modu) |
| `run_logs` | Her çalıştırma için yürütme günlükleri |

## Geliştirme

```bash
# Frontend dev server only
npm run dev

# Type checking
npm run check

# Run tests
npm test

# Run Rust tests
cd src-tauri && cargo test

# Lint
npm run lint
```

### Android

[Tauri Android gereksinimleri](https://v2.tauri.app/start/prerequisites/#android) gerekir: Android SDK ve NDK (`ANDROID_HOME`, `NDK_HOME`), Java 17 (`JAVA_HOME`) ve Rust hedefleri (`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`). USB hata ayıklaması açık bir telefon bağlayın veya bir emülatör başlatın.

```bash
make android-devices    # check the phone or emulator is connected
make android-emulator   # start an emulator (AVD=<name> to pick one)
make android-apk        # build dist/Query2Table_<version>_android.apk
make android-run        # install it on the device and open the app
make android-logs       # follow the app's log
make android-dev        # live reload on the device
```

`src-tauri/gen/android/keystore.properties` dosyası yoksa derlemeler hata ayıklama anahtarıyla imzalanır. `make android-keystore` bir yayın anahtarı ve bu dosyayı oluşturur, ayrıca yayın iş akışının her sürüme eklediği APK’yı imzalamak için kullandığı GitHub gizli değerlerini (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_PASSWORD`, `ANDROID_KEY_ALIAS`) yazdırır. Anahtarı saklayın: güncellemeler mevcut uygulamanın üzerine yalnızca aynı anahtarla imzalandığında kurulur.

## Dışa aktarma biçimleri

| Biçim | İçerik |
|-------|--------|
| **CSV** | Tüm sütunlar + JSON sütunu olarak kaynaklar |
| **JSON** | İç içe kaynak dizisi ve çalıştırma meta verileriyle tam satırlar |
| **XLSX** | Veri sayfası, kaynaklar sayfası ve meta veri sayfası içeren biçimlendirilmiş çalışma kitabı |

## Lisans

MIT
