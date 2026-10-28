<p align="center">
  <img src="../../images/query2table-logo.png" alt="Logo Query2Table" width="160">
</p>

<h1 align="center">Query2Table</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="README.ru.md">Русский</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português (Brasil)</a> · <b>Polski</b> · <a href="README.uk.md">Українська</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.zh.md">简体中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a></p>

<p align="center">
  <a href="https://github.com/hightemp/query2table/releases/latest"><img src="https://img.shields.io/github/v/release/hightemp/query2table?style=flat-square" alt="Wydanie na GitHub"></a>
  <a href="https://github.com/hightemp/query2table/releases"><img src="https://img.shields.io/github/downloads/hightemp/query2table/total?style=flat-square" alt="Pobrania z GitHub"></a>
  <a href="#licencja"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="Licencja: MIT"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8D8?style=flat-square&amp;logo=tauri" alt="Zbudowane w Tauri v2"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust-orange?style=flat-square&amp;logo=rust" alt="Backend w Rust"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&amp;logo=svelte" alt="Frontend w Svelte 5"></a>
  <br>
  <img src="https://asdertasd.site/counter/query2table" alt="Wyświetlenia Query2Table">
</p>

**Query2Table** to lokalne narzędzie badawcze na komputer (Linux, Windows, macOS) i Androida. Opisujesz, jakich danych potrzebujesz, a aplikacja samodzielnie przeszukuje internet w wielu źródłach i językach, pobiera strony, wyodrębnia z nich ustrukturyzowane dane za pomocą LLM, usuwa duplikaty i przedstawia wynik w tabeli aktualizowanej na żywo, z linkami do źródeł dla każdego wiersza. Wszystko działa na Twoim komputerze, bez backendu w chmurze.

Oprócz tabel aplikacja ma trzy kolejne tryby: **wyszukiwanie obrazów**, które znajduje obrazy i porządkuje je za pomocą LLM według trafności wizualnej; **wyszukiwanie linków**, które czyta pełną treść stron-kandydatów i zwraca tylko najtrafniejsze linki z opisami i ocenami trafności wygenerowanymi przez LLM; oraz **badanie**, tryb agentowy, w którym model samodzielnie wyszukuje, czyta strony, rozumuje i pisze odpowiedź w Markdown ze źródłami.

Aplikacja jest dostępna także na telefony z **Androidem**: ta sama aplikacja z tymi samymi trybami, ustawieniami i historią, zbudowana z tego samego kodu i dostosowana do ekranu telefonu z dolnym paskiem kart.

Zadaj pytanie w rodzaju *„Utwórz tabelę porównawczą otwartoźródłowych baz danych i bibliotek do wyszukiwania wektorowego z ich językiem, licencją, typem wdrożenia, zastosowaniem i stroną WWW”* i obserwuj, jak tabela wypełnia się na bieżąco.

![Tabela porównawcza otwartoźródłowych wektorowych baz danych z dowodami i źródłami](../../screenshots/test-en-2026-10-06/44-table-results.png)

## Przegląd

### Pytaj zwykłym językiem

Wybierz format wyników, opisz, czego potrzebujesz, i ustaw warunki zatrzymania. W przypadku tabel aplikacja proponuje kolumny, które możesz edytować przed rozpoczęciem wyszukiwania.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/01-query-table.png" alt="Nowe zapytanie z czterema formatami wyników"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/05-schema-review.png" alt="Proponowany schemat tabeli gotowy do przeglądu"></td>
  </tr>
</table>

### Każdy wiersz zachowuje swoje dowody

Otwórz wiersz, aby przeczytać pełne wartości, skopiować je i sprawdzić strony, z których zostały wyodrębnione.

![Szczegóły wiersza z pewnością wyodrębnienia i źródłami](../../screenshots/test-en-2026-10-06/45-row-details-and-sources.png)

### Badanie z pytaniami uzupełniającymi

Agent wyszukuje, czyta strony i pisze odpowiedź ze źródłami oraz spisem treści. Każdy krok jest widoczny w zakładce Aktywność, a pytania uzupełniające kontynuują rozmowę z jej historią.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/21-research-answer.png" alt="Odpowiedź badania ze spisem treści i sugerowanymi pytaniami uzupełniającymi"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/22-research-activity.png" alt="Aktywność badania z ponumerowanymi krokami wyszukiwania i czytania"></td>
  </tr>
</table>

### Obrazy i linki

Wyszukiwanie obrazów porządkuje wyniki według trafności i pozwala filtrować je według witryny i rozmiaru; wyszukiwanie linków czyta każdą stronę i wyjaśnia, dlaczego pasuje.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/28-images-grid.png" alt="Wyniki wyszukiwania obrazów w wyrównanej siatce"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/32-links-results.png" alt="Trafne linki z opisami i poziomami dopasowania"></td>
  </tr>
</table>

### Historia i ustawienia

Wyszukuj, filtruj i ponownie otwieraj wcześniejsze przebiegi, uruchamiaj je ponownie lub eksportuj kilka naraz. Ustawienia sprawdzają połączenia z LLM i wyszukiwarką, zanim zaczniesz.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/18-history.png" alt="Historia przebiegów pogrupowana według dni z wynikami, czasem trwania, kosztem i modelem"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/43-settings-search-connection.png" alt="Ustawienia wyszukiwania z udanym testem połączenia"></td>
  </tr>
</table>

### Motyw jasny i ciemny

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/53-research-dark.png" alt="Odpowiedź badania w motywie ciemnym"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/50-table-dark.png" alt="Tabela wyników w motywie ciemnym"></td>
  </tr>
</table>

## Pobieranie

Najnowszą wersję pobierzesz z sekcji [Releases](https://github.com/hightemp/query2table/releases/latest):

| Platforma | Plik |
|----------|------|
| Linux | `.AppImage`, `.deb`, `.rpm` |
| Windows | `.msi`, `.exe` |
| macOS (Apple Silicon) | `.dmg` |
| Android 7.0+ | `Query2Table_<version>_android.apk` |

Aby przenieść konfigurację z komputera na telefon, użyj opcji **Ustawienia → Eksportuj ustawienia…**, zaznacz **Dołącz klucze API i proxy** i zaimportuj plik na telefonie. Taki plik zawiera Twoje klucze: zachowaj go w tajemnicy i usuń po zaimportowaniu.

Na Androidzie otwórz pobrany plik APK i, gdy pojawi się prośba, zezwól na instalowanie aplikacji z przeglądarki lub menedżera plików. Aplikacja na Androida przechowuje dane na telefonie; nie ma ikony w zasobniku, a powiadomienia pojawiają się jako zwykłe powiadomienia Androida. Z lokalnych serwerów modeli (Ollama, llama.cpp) można korzystać przez sieć, podając ich adres, np. `http://192.168.1.10:11434`.

## Funkcje

- **Zapytania w języku naturalnym** — opisz zwykłymi słowami, czego szukasz
- **Automatyczne wyznaczanie schematu** — aplikacja proponuje kolumny tabeli; zatwierdzasz je lub edytujesz przed uruchomieniem
- **Wielu dostawców wyszukiwania** — Brave Search + Serper z automatycznym przełączaniem na zapasowego
- **Wielojęzyczne rozszerzanie zapytań** — wyszukiwanie w różnych językach i regionach
- **Wyodrębnianie za pomocą LLM** — OpenRouter, Anthropic Claude (buforowanie promptów, opcjonalne czytanie całych plików PDF), Ollama (lokalnie lub w chmurze), DeepSeek, Groq, Mistral, Gemini, xAI, OpenAI, LM Studio, llama.cpp lub dowolny adres zgodny z OpenAI
- **Wyniki strumieniowe** — wiersze pojawiają się w tabeli na żywo, gdy tylko zostaną znalezione
- **Wyodrębnianie z PDF** — automatyczne wykrywanie dokumentów PDF w wynikach wyszukiwania i wyodrębnianie z nich tekstu
- **Tryb wyszukiwania obrazów** — osobny potok, który zamiast tekstu wyszukuje obrazy: generuje 6–12 zróżnicowanych wariantów zapytania (przez LLM lub statyczne warianty zapasowe), wykonuje wyszukiwania obrazów przez Brave / Serper, a następnie ocenia każdy wynik za pomocą LLM według ścisłej skali trafności (0,0–1,0); obrazy z oceną poniżej 0,7 są odrzucane, pozostałe są zapisywane w kolejności ocen, a interfejs aktualizuje się na bieżąco dla każdego obrazu; budżet, anulowanie i warunki zatrzymania (liczba / koszt / czas) są przestrzegane przez cały czas
- **Tryb wyszukiwania linków** — osobny potok, który zamiast tabeli zwraca najtrafniejsze strony: generuje warianty zapytań wyszukiwania na podstawie Twojej prośby, wykonuje wyszukiwania w internecie przez Brave / Serper (z usuwaniem duplikatów według URL), pobiera i analizuje każdą stronę, a następnie LLM czyta pełną treść, ocenia jej trafność (0,0–1,0) względem zapytania i tworzy krótki opis; strony poniżej progu pewności są odrzucane, pozostałe są zapisywane w kolejności ocen, a interfejs aktualizuje się na bieżąco dla każdego linku; budżet, anulowanie i warunki zatrzymania są przestrzegane przez cały czas
- **Tryb badania** — tryb agentowy, który odpowiada na otwarte pytania w Markdown zamiast tabelą: LLM prowadzi pętlę wywołań narzędzi (wyszukaj w internecie, pobierz stronę jako Markdown, pomyśl, odpowiedz) przez ustaloną maksymalną liczbę kroków, przesyłając każdy krok na żywo do osi czasu; kroki rozumowania (`think`) i wszelkie błędy (kroki `error`, wyróżnione na czerwono) są widoczne, dzięki czemu proces pozostaje przejrzysty, a końcowa odpowiedź Markdown ze źródłami jest wyświetlana (po oczyszczeniu) i może zostać wyeksportowana jako `.md`; budżet, anulowanie i warunki zatrzymania są przestrzegane przez cały czas
- **Wzory i obrazy w odpowiedziach badania** — wzory LaTeX są rysowane (MathJax, offline); agent może wyszukiwać obrazy, oglądać je i umieszczać pasujące w odpowiedzi, a są one przechowywane razem z przebiegiem
- **Diagramy i wykresy w odpowiedziach badania** — diagramy Mermaid oraz wykresy słupkowe, liniowe, warstwowe, kołowe, punktowe, radarowe i mapy ciepła, rysowane offline w motywie aplikacji; można je zapisać jako SVG lub PNG i otworzyć na pełnym ekranie
- **Wideo w odpowiedziach badania** — agent znajduje filmy na dowolnych stronach (wyszukiwanie wideo w sieci albo bezpośrednio YouTube, Bilibili, Niconico i SoundCloud), czyta ich napisy ze znacznikami czasu i cytuje konkretne momenty; filmy pojawiają się jako karty z zapisanymi miniaturami. Korzysta z yt-dlp pobieranego przy pierwszym użyciu (na Androidzie z wbudowanym Pythonem)
- **Statystyki użycia** — pieniądze, tokeny, wywołania modeli, wyszukiwania Brave i Serper oraz ilość danych według miesięcy, lat i za cały okres, z podziałem na modele i tryby, najdroższymi przebiegami, zajętością dysku, eksportem CSV i budżetem miesięcznym z ostrzeżeniami
- **13 języków interfejsu** — angielski, rosyjski, niemiecki, francuski, hiszpański, włoski, portugalski (Brazylia), polski, ukraiński, turecki, chiński (uproszczony), japoński i koreański; domyślnie używany jest język systemu
- **Źródła dla każdego wiersza** — każdy wiersz zawiera linki do stron, z których został wyodrębniony
- **Usuwanie duplikatów obiektów** — dopasowanie przybliżone + rozstrzyganie niejednoznaczności z pomocą LLM
- **Konfigurowalne warunki zatrzymania** — docelowa liczba wierszy, maksymalny koszt, maksymalny czas
- **Historia przebiegów** — przeglądanie, wyświetlanie i ponowny eksport wcześniejszych przebiegów
- **Eksport** — CSV, JSON, XLSX z pełnymi metadanymi źródeł
- **Motyw ciemny / jasny** — przełącznik w panelu bocznym
- **Zasobnik systemowy** — powiadomienia o zakończeniu (na komputerze)
- **Aplikacja na Androida** — ta sama aplikacja na telefonach, z układem dla małych ekranów
- **Aktualizacje w aplikacji** — nowe wersje są ogłaszane wraz z listą zmian i instalowane z poziomu aplikacji po sprawdzeniu ich podpisu (komputer i Android)
- **Najpierw lokalnie** — wszystkie dane w SQLite, bez potrzeby backendu w chmurze

## Stos technologiczny

| Warstwa | Technologia |
|-------|------------|
| Powłoka aplikacji | Tauri v2 (komputer i Android) |
| Backend | Rust (asynchroniczne środowisko Tokio) |
| Frontend | Svelte 5 (SvelteKit SPA) |
| Style interfejsu | Czysty CSS z tokenami projektowymi |
| Baza danych | SQLite (sqlx, tryb WAL) |
| LLM | OpenRouter / Ollama (lokalnie) / Ollama Cloud / API zgodne z OpenAI (np. llama.cpp) |
| Wyszukiwanie | Brave Search API / Serper API |
| Ikony | Lucide |

## Wymagania wstępne

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) (stabilny zestaw narzędzi)
- [Wymagania wstępne Tauri v2](https://v2.tauri.app/start/prerequisites/) dla Twojego systemu operacyjnego

## Pierwsze kroki

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

## Konfiguracja

Przy pierwszym uruchomieniu aplikacja tworzy lokalną bazę danych SQLite z ustawieniami domyślnymi. Otwórz **Ustawienia** z panelu bocznego, aby skonfigurować:

| Grupa | Ustawienia |
|-------|----------|
| **Dostawca LLM** | Dostawca (OpenRouter / Ollama / Ollama Cloud / zgodny z OpenAI), URL, model, klucz API, tryb JSON, temperatura, maks. tokenów |
| **Dostawca wyszukiwania** | Dostawca (Brave / Serper), klucz API, przełącznik dostawcy zapasowego, wyniki na zapytanie |
| **Wykonywanie** | Równoległe pobieranie (domyślnie 8), równoległe wyodrębnianie (domyślnie 3), limit czasu pobierania, ograniczanie liczby żądań, robots.txt |
| **Jakość** | Równowaga precyzji i kompletności, rygor dowodów, próg pewności, podobieństwo duplikatów |

Warunki zatrzymania (docelowa liczba wierszy, maks. koszt, maks. czas) ustawia się dla każdego zapytania na stronie Zapytanie.

Opcja **Używaj drugiego dostawcy jako zapasowego** steruje automatycznym przełączaniem między Brave a Serper zarówno w wyszukiwaniu w internecie, jak i w wyszukiwaniu obrazów. Wyłącz ją, aby używać tylko wybranego głównego dostawcy, nawet jeśli zapisano oba klucze API. Wybór obowiązuje dla nowych przebiegów po zapisaniu; ponowne próby pozostają przy wybranym dostawcy. Gdy opcja jest włączona, drugi dostawca jest używany tylko po niepowodzeniu i wymaga zapisanego klucza.

W **Ustawienia → Wyszukiwanie → Wyniki na zapytanie** określisz, ile wyników żąda każde zapytanie wyszukiwania we wszystkich czterech trybach, łącznie z Badaniem; następnie zapisz zmiany dla nowych przebiegów. Żądania do Brave ograniczają tę wartość do udokumentowanego zakresu punktu końcowego: [1–20 dla wyszukiwania w internecie](https://api-dashboard.search.brave.com/api-reference/web/search/get) i [do 200 dla obrazów](https://api-dashboard.search.brave.com/documentation/services/image-search). Dotyczy to także wcześniej zapisanych zbyt dużych wartości. Większe wartości nie powodują automatycznego pobierania dodatkowych stron wyników.

### Użycie, ceny i limity wydatków

**Użycie i koszt** rozróżnia opłaty zgłoszone przez dostawcę, szacunki ze stawek modeli, sumy częściowe i koszty nieznane. Każda próba u dostawcy jest liczona osobno, w tym ponowne próby, wyszukiwania zapasowe, nieudane odpowiedzi z danymi o użyciu oraz anulowane żądania, których końcowej opłaty nie udało się uzyskać. Użycie wejścia i wyjścia pochodzi z odpowiedzi dostawcy; tokeny myślenia nie są naliczane po raz drugi ponad tokeny wyjściowe.

- **OpenRouter:** pierwszeństwo ma [zgłoszony koszt żądania](https://openrouter.ai/docs/cookbook/administration/usage-accounting). Jeśli jest niedostępny, szacunek powstaje na podstawie zgłoszonych tokenów i publicznych stawek z katalogu dla faktycznie zwróconego modelu.
- **Ollama Cloud:** szacunek zużycia kredytów na podstawie [opublikowanych stawek modeli](https://ollama.com/pricing), z uwzględnieniem buforowanego wejścia i rozpoznanych okresów cen szczytowych. Kredyty w pakiecie, opłaty abonamentowe i płatności gotówkowe mogą różnić się od tego szacunku. Nierozpoznane lub niedostępne taryfy pozostają nieznane.
- **Własne stawki modeli:** w Ustawieniach można ustawić stawki za wejście i wyjście w USD za milion tokenów. Stawki dotyczą konkretnego dostawcy, punktu końcowego i modelu. Jawne zero oznacza darmowe użycie; brakujące stawki nigdy nie stają się po cichu zerem. Opublikowane stawki są pobierane bez zmieniania zapisanych stawek ręcznych.
- **Wyszukiwanie:** Brave i Serper domyślnie mają 0 USD za 1000 żądań, co wyłącza opłaty za wyszukiwanie z szacunków. Ustaw rzeczywistą stawkę swojego planu, aby je uwzględnić, lub pozostaw cenę pustą, jeśli jest nieznana. Nieudane próby bez danych rozliczeniowych pozostają bez ceny. Zmiany ustawień dotyczą nowych przebiegów; zapisana historia nie jest przeliczana. Przy aktualizacji dawne puste wartości domyślne jednorazowo stają się zerem, a istniejące stawki liczbowe i późniejsze celowo puste pola zostają zachowane.

**Maks. koszt** przebiegu blokuje nowe żądania do dostawców, gdy rozliczone wydatki osiągną limit. Nie jest to gwarantowany pułap faktury: szacunki mogą różnić się od rozliczeń, nieznanych opłat nie da się uwzględnić, a żądania już w toku mogą nadal generować koszty. Gdy wymagany jest ścisły limit, użyj limitów po stronie dostawcy. Rozliczenie jest zachowywane po zakończeniu, niepowodzeniu, anulowaniu i przebiegach bez wyników. Wcześniejsze wpisy historii są wyraźnie oznaczone jako szacunki historyczne.

Usuwanie duplikatów zachowuje identyfikator wiersza pierwszego elementu grupy, transakcyjnie scala wartości i rekordy źródeł grupy oraz uzgadnia tabelę na żywo z historią i eksportem. Wcześniej nadpisanych wyników historycznych nie da się automatycznie odtworzyć; aby uzyskać świeże wyniki, uruchom te zapytania ponownie.

Przyciski **Zapisz** i **Odrzuć** w Ustawieniach pozostają widoczne podczas przewijania. Linki do sekcji prowadzą bezpośrednio do odpowiednich opcji, a opuszczenie strony z niezapisanymi zmianami oferuje wybór: Zapisz / Odrzuć / Zostań. Zmiany wprowadzone w trakcie zapisywania pozostają dostępne do następnego zapisu.

Wyniki są wyświetlane w ciągłej, zwirtualizowanej tabeli z lokalnym wyszukiwaniem i sortowaniem z klawiatury. Otwórz **Szczegóły** wiersza, aby przeczytać lub skopiować pełne wartości i sprawdzić zapisane źródła. W Badaniu odpowiedź znajduje się przed rozwijaną aktywnością. Obrazy obsługują widok siatki i listy oraz podgląd z klawiatury; eksport i usuwanie pokazują wyraźny postęp i wynik. Wyczyszczenie diagnostycznego panelu **Logi** nie resetuje aktywności przebiegu.

Testy regresji w przeglądarce używają danych syntetycznych i symulowanych poleceń Tauri: zainstaluj przeglądarki testowe poleceniem `npx playwright install chromium webkit`, a następnie uruchom `npm run test:ui`. Istniejące testy frontendu uruchamia się poleceniem `npm test`.

### Klucze API

Potrzebny jest co najmniej jeden klucz API wyszukiwarki i skonfigurowany dostawca LLM. Lokalna Ollama i serwery zgodne z OpenAI bez uwierzytelniania nie wymagają klucza API LLM:

| Usługa | Gdzie uzyskać klucz |
|---------|-------------|
| Brave Search | https://brave.com/search/api/ |
| Serper | https://serper.dev/ |
| OpenRouter | https://openrouter.ai/ |
| Ollama (lokalnie) | https://ollama.com/ — klucz niepotrzebny |
| Ollama Cloud | https://ollama.com/settings/keys |
| Zgodny z OpenAI | Klucz opcjonalny, zależnie od serwera |

Wybierz dostawcę w **Ustawienia → LLM → Dostawca**, wypełnij jego pola i kliknij **Zapisz**. Każdy dostawca zachowuje własny model i dane logowania po przełączeniu.

Migracje ustawień uruchamiają się automatycznie przy starcie: nowe opcje otrzymują wartości domyślne, a istniejące wartości są zachowywane. Ustawienia o zmienionych nazwach są jawnie migrowane w backendzie przed wstawieniem wartości domyślnych; zmiany formatu przechowywanych danych również wymagają kodu migracji przy wdrażaniu nowej wersji.

Na dole **Ustawień** sekcja **Pliki aplikacji** pokazuje faktyczny folder danych, bazę danych `data.db` i folder logów. Użyj **Kopiuj ścieżkę**, aby skopiować lokalizację do schowka systemowego, lub **Otwórz folder**, aby otworzyć ją w menedżerze plików. Otwarcie lokalizacji bazy danych otwiera folder, w którym się znajduje. Ustawienia, historia, wyniki i źródła są przechowywane w `data.db`; logi diagnostyczne są zapisywane osobno.

- **OpenRouter:** wpisz klucz API i wybierz model z przeszukiwalnego [katalogu OpenRouter](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties). Wpisz tekst, aby filtrować identyfikatory modeli, wybierz myszą lub strzałkami i klawiszem Enter, a **Odśwież** wczytuje listę ponownie. Katalog można przeglądać przed wpisaniem klucza; klucz jest wymagany do korzystania z modelu.
- **Ollama (lokalnie):** URL `http://localhost:11434` i nazwa zainstalowanego modelu.
- **Ollama Cloud:** URL `https://ollama.com`, klucz API Ollama i model wybrany z przeszukiwalnego katalogu serwera. Kliknij pole modelu i wpisz tekst, aby filtrować, a następnie wybierz myszą lub strzałkami i klawiszem Enter; **Odśwież** wczytuje katalog ponownie. Filtrowanie nie zmienia wybranego modelu, dopóki nie wybierzesz wyniku. Bezpośredni dostęp do chmury korzysta z [natywnego API Ollama](https://docs.ollama.com/cloud). Chmura nie obsługuje obecnie [ustrukturyzowanego wyjścia](https://docs.ollama.com/capabilities/structured-outputs), więc JSON jest żądany przez prompty i analizowany przez istniejący potok.
- **Zgodny z OpenAI:** wpisz bazowy URL API razem z `/v1` i identyfikator modelu na serwerze. Dla [llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server) domyślny URL to `http://localhost:8080/v1`; użyj nazwy wczytanego modelu lub skonfigurowanego aliasu. Pozostaw klucz API pusty, chyba że serwer go wymaga. Wyłącz **Tryb JSON**, jeśli serwer odrzuca `response_format`.

**Myślenie / poziom rozumowania** określa, jak model rozumuje przed udzieleniem odpowiedzi. **Auto** zachowuje bezpieczne ustawienia domyślne: ustrukturyzowane żądania do Ollama wyłączają myślenie (GPT-OSS używa poziomu Niskie), a pozostali dostawcy zachowują swoje ustawienia domyślne. **Domyślne dostawcy** pozostawia rozumowanie serwerowi. **Wył.**, **Wł.**, **Niskie**, **Średnie**, **Wysokie** i **Maks.** żądają konkretnego trybu lub poziomu; obsługa zależy od modelu i dostawcy. GPT-OSS w Ollama przyjmuje Niskie/Średnie/Wysokie i nie może używać Wył. ani Maks. Zmiany obowiązują dla nowych przebiegów po zapisaniu.

**Maks. tokenów wyjściowych** to limit na jedno żądanie, potencjalnie wspólny dla tokenów myślenia i odpowiedzi. Jest niezależny od limitów kosztu i czasu przebiegu oraz od limitu konta u dostawcy. Wyższy poziom rozumowania może zostawić mniej tokenów na odpowiedź.

Problemy z żądaniami do modelu pokazują dostawcę, model, etap, żądany limit wyjścia, zgłoszone liczby tokenów wejścia, wyjścia i myślenia oraz informację, czy zaplanowano ponowną próbę. Limity wyjścia, limity kontekstu, limity dostawcy, ograniczenia liczby żądań, błędy uwierzytelniania i sieci mają osobne wyjaśnienia i działania naprawcze. Problemy pozostają widoczne, gdy pojedyncze strony zostają pominięte, i są dostępne w Historii przebiegów. Inne błędy pokazują krótkie wyjaśnienie, zalecane działanie i zwijane szczegóły techniczne.

**Wstrzymaj** zawiesza przetwarzanie na bieżącym etapie, łącznie z przeglądem schematu, a **Wznów** kontynuuje bez ponownego wysyłania żądania. Żądanie już wysłane do zdalnego modelu może się tam zakończyć mimo wstrzymania. **Anuluj** zatrzymuje pracę lokalną i zamyka oczekujące żądania. Błędy potoku są wyświetlane i zapisywane jako `failed`, a pierwotny błąd jest dostępny w logu przebiegu.

## Architektura

Backend używa **maszyny stanów potoku** ze stałymi rolami wykonywanymi kolejno:

```
Query → Interpret → Plan Schema → [User Confirms] → Plan Searches
      → Expand Queries → Execute Search → Fetch Pages → Parse Documents
      → Extract Entities → Validate → Deduplicate → [Stop Check] → Done
```

### Role potoku

| Rola | LLM | Przeznaczenie |
|------|-----|---------|
| QueryInterpreter | Tak | Przekształca zapytanie w ustrukturyzowaną intencję |
| SchemaPlanner | Tak | Proponuje kolumny tabeli i ich typy |
| SearchPlanner | Tak | Generuje zapytania wyszukiwania dla wielu języków i regionów |
| QueryExpander | Tak | Tłumaczy zapytania na języki docelowe |
| SearchExecutor | Nie | Wywołuje API wyszukiwania, zbiera kandydackie adresy URL |
| Fetcher | Nie | Pobieranie HTTP z ograniczaniem liczby żądań i obsługą robots.txt |
| DocumentParser | Nie | HTML → czysty tekst (usuwanie zbędnych elementów strony) |
| PdfParser | Nie | PDF → czysty tekst (przez pdf-extract) |
| Extractor | Tak | Tekst + schemat → ustrukturyzowane wiersze |
| Validator | Częściowo | Zgodność ze schematem + kontrole semantyczne |
| Deduplicator | Częściowo | Dopasowanie przybliżone (strsim) + LLM w przypadkach granicznych |
| LinkRanker | Tak | Czyta treść strony, ocenia trafność (0,0–1,0) i tworzy opis (tryb wyszukiwania linków) |
| ResearchAgent | Tak | Prowadzi agentową pętlę wyszukaj/pobierz/pomyśl/odpowiedz i tworzy odpowiedź w Markdown (tryb badania) |
| StoppingController | Nie | Sprawdza warunki zatrzymania (wiersze, budżet, czas, nasycenie) |

### Maszyna stanów

```
Pending → SchemaReview → Running ⇄ Paused → Completed / Failed / Cancelled
```

## Struktura projektu

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

## Baza danych

SQLite w trybie WAL z następującymi tabelami:

| Tabela | Przeznaczenie |
|-------|---------|
| `settings` | Konfiguracja użytkownika (klucze API, modele, preferencje) |
| `runs` | Rekordy przebiegów ze stanem i statystykami |
| `run_schemas` | Zatwierdzony schemat tabeli dla każdego przebiegu |
| `search_queries` | Wygenerowane zapytania wyszukiwania dla każdego przebiegu |
| `search_results` | Adresy URL zebrane z API wyszukiwania |
| `fetched_pages` | Pobrana treść stron |
| `entity_rows` | Wyodrębnione ustrukturyzowane wiersze (dane JSON) |
| `row_sources` | Dowody dla wierszy (URL, tytuł, fragment) |
| `image_results` | Ocenione obrazy dla każdego przebiegu (tryb wyszukiwania obrazów) |
| `link_results` | Ocenione trafne linki z opisami i ocenami (tryb wyszukiwania linków) |
| `research_steps` | Kroki wywołań narzędzi przez agenta dla każdego przebiegu — search/fetch/think/error (tryb badania) |
| `research_results` | Końcowa odpowiedź w Markdown dla każdego przebiegu (tryb badania) |
| `run_logs` | Logi wykonania dla każdego przebiegu |

## Rozwój

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

Wymagane są [wymagania wstępne Tauri dla Androida](https://v2.tauri.app/start/prerequisites/#android): Android SDK i NDK (`ANDROID_HOME`, `NDK_HOME`), Java 17 (`JAVA_HOME`) oraz cele Rust (`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`). Podłącz telefon z włączonym debugowaniem USB lub uruchom emulator.

```bash
make android-devices    # check the phone or emulator is connected
make android-emulator   # start an emulator (AVD=<name> to pick one)
make android-apk        # build dist/Query2Table_<version>_android.apk
make android-run        # install it on the device and open the app
make android-logs       # follow the app's log
make android-dev        # live reload on the device
```

Kompilacje są podpisywane kluczem debugowania, chyba że istnieje plik `src-tauri/gen/android/keystore.properties`. `make android-keystore` tworzy klucz wydania i ten plik oraz wypisuje sekrety GitHub (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_PASSWORD`, `ANDROID_KEY_ALIAS`), których workflow wydań używa do podpisania pliku APK dołączanego do każdego wydania. Zachowaj klucz: aktualizacje instalują się na istniejącej aplikacji tylko wtedy, gdy są podpisane tym samym kluczem.

## Formaty eksportu

| Format | Zawartość |
|--------|----------|
| **CSV** | Wszystkie kolumny + źródła jako kolumna JSON |
| **JSON** | Pełne wiersze z zagnieżdżoną tablicą źródeł i metadanymi przebiegu |
| **XLSX** | Sformatowany skoroszyt z arkuszem danych, arkuszem źródeł i arkuszem metadanych |

## Licencja

MIT
