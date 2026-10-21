<p align="center">
  <img src="../../images/query2table-logo.png" alt="Query2Table-Logo" width="160">
</p>

<h1 align="center">Query2Table</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="README.ru.md">Русский</a> · <b>Deutsch</b> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <a href="README.it.md">Italiano</a> · <a href="README.pt.md">Português (Brasil)</a> · <a href="README.pl.md">Polski</a> · <a href="README.uk.md">Українська</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.zh.md">简体中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a></p>

<p align="center">
  <a href="https://github.com/hightemp/query2table/releases/latest"><img src="https://img.shields.io/github/v/release/hightemp/query2table?style=flat-square" alt="GitHub-Release"></a>
  <a href="https://github.com/hightemp/query2table/releases"><img src="https://img.shields.io/github/downloads/hightemp/query2table/total?style=flat-square" alt="GitHub-Downloads"></a>
  <a href="#lizenz"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="Lizenz: MIT"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8D8?style=flat-square&amp;logo=tauri" alt="Erstellt mit Tauri v2"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust-orange?style=flat-square&amp;logo=rust" alt="Backend in Rust"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&amp;logo=svelte" alt="Frontend in Svelte 5"></a>
  <br>
  <img src="https://asdertasd.site/counter/query2table" alt="Aufrufe von Query2Table">
</p>

**Query2Table** ist ein lokal arbeitendes Recherchewerkzeug für den Desktop (Linux, Windows, macOS) und Android. Sie beschreiben, welche Daten Sie brauchen; die App durchsucht selbstständig das Internet über mehrere Quellen und Sprachen hinweg, lädt Seiten, extrahiert mit LLMs strukturierte Objekte, entfernt Duplikate und zeigt das Ergebnis als laufend aktualisierte Tabelle mit Quellenlinks für jede Zeile – alles auf Ihrem Rechner, ohne Cloud-Backend.

Neben Tabellen bietet sie drei weitere Modi: **Bildsuche**, die passende Bilder findet und per LLM nach visueller Relevanz ordnet; **Linksuche**, die den vollständigen Inhalt infrage kommender Seiten liest und nur die relevantesten Links mit vom LLM erstellten Beschreibungen und Relevanzwerten liefert; und **Recherche**, einen agentischen Modus, in dem das Modell selbstständig sucht, Seiten liest, nachdenkt und eine Markdown-Antwort mit Quellen schreibt.

Die App gibt es auch für **Android**-Telefone: dieselbe App mit denselben Modi, Einstellungen und demselben Verlauf, aus demselben Code gebaut und mit einer unteren Tab-Leiste an den Telefonbildschirm angepasst.

Fragen Sie etwa *„Erstelle eine Vergleichstabelle von Open-Source-Datenbanken und -Bibliotheken für Vektorsuche mit Sprache, Lizenz, Bereitstellungsart, Einsatzzweck und Website“* und sehen Sie zu, wie sich die Tabelle in Echtzeit füllt.

![Vergleichstabelle von Open-Source-Vektordatenbanken mit Belegen und Quellen](../../screenshots/test-en-2026-10-06/44-table-results.png)

## Rundgang

### Fragen in natürlicher Sprache

Wählen Sie ein Ergebnisformat, beschreiben Sie, was Sie brauchen, und legen Sie Abbruchbedingungen fest. Für Tabellen schlägt die App Spalten vor, die Sie vor Beginn der Suche bearbeiten können.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/01-query-table.png" alt="Neue Anfrage mit den vier Ergebnisformaten"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/05-schema-review.png" alt="Vorgeschlagenes Tabellenschema zur Prüfung"></td>
  </tr>
</table>

### Jede Zeile behält ihre Belege

Öffnen Sie eine Zeile, um vollständige Werte zu lesen, sie zu kopieren und die Seiten zu prüfen, aus denen sie extrahiert wurden.

![Zeilendetails mit Extraktionssicherheit und Quellen](../../screenshots/test-en-2026-10-06/45-row-details-and-sources.png)

### Recherche mit Folgefragen

Der Agent sucht, liest Seiten und schreibt eine Antwort mit Quellen und Inhaltsverzeichnis. Jeder Schritt ist unter „Aktivität“ sichtbar, und Folgefragen setzen die Unterhaltung mit ihrem Verlauf fort.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/21-research-answer.png" alt="Rechercheantwort mit Inhaltsverzeichnis und vorgeschlagenen Folgefragen"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/22-research-activity.png" alt="Rechercheaktivität mit nummerierten Such- und Leseschritten"></td>
  </tr>
</table>

### Bilder und Links

Die Bildsuche ordnet Ergebnisse nach Relevanz und filtert nach Website und Größe; die Linksuche liest jede Seite und erklärt, warum sie passt.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/28-images-grid.png" alt="Ergebnisse der Bildsuche in einem Blocksatz-Raster"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/32-links-results.png" alt="Relevante Links mit Beschreibungen und Übereinstimmungsstufen"></td>
  </tr>
</table>

### Verlauf und Einstellungen

Durchsuchen, filtern und öffnen Sie frühere Durchläufe, führen Sie sie erneut aus oder exportieren Sie mehrere auf einmal. Die Einstellungen prüfen Ihre LLM- und Suchverbindungen, bevor Sie beginnen.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/18-history.png" alt="Verlauf der Durchläufe, nach Tagen gruppiert, mit Ergebnissen, Dauer, Kosten und Modell"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/43-settings-search-connection.png" alt="Sucheinstellungen mit erfolgreichem Verbindungstest"></td>
  </tr>
</table>

### Helles und dunkles Design

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/53-research-dark.png" alt="Rechercheantwort im dunklen Design"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/50-table-dark.png" alt="Ergebnistabelle im dunklen Design"></td>
  </tr>
</table>

## Download

Laden Sie den neuesten Build unter [Releases](https://github.com/hightemp/query2table/releases/latest) herunter:

| Plattform | Datei |
|-----------|-------|
| Linux | `.AppImage`, `.deb`, `.rpm` |
| Windows | `.msi`, `.exe` |
| macOS (Apple Silicon) | `.dmg` |
| Android 7.0+ | `Query2Table_<version>_android.apk` |

Um Ihre Einrichtung vom Computer auf das Telefon zu übertragen, verwenden Sie **Einstellungen → Einstellungen exportieren…**, aktivieren **API-Schlüssel und Proxys einschließen** und importieren die Datei auf dem Telefon. Eine solche Datei enthält Ihre Schlüssel: Halten Sie sie privat und löschen Sie sie nach dem Import.

Öffnen Sie unter Android die heruntergeladene APK und erlauben Sie auf Nachfrage die Installation von Apps aus Ihrem Browser oder Dateimanager. Die Android-App speichert ihre Daten auf dem Telefon; sie hat kein Tray-Symbol, Benachrichtigungen erscheinen als normale Android-Benachrichtigungen. Lokale Modellserver (Ollama, llama.cpp) lassen sich über Ihr Netzwerk unter ihrer Adresse nutzen, z. B. `http://192.168.1.10:11434`.

## Funktionen

- **Anfragen in natürlicher Sprache** – beschreiben Sie in Alltagssprache, was Sie möchten
- **Automatische Schema-Erkennung** – die App schlägt Tabellenspalten vor; Sie bestätigen oder bearbeiten sie vor der Ausführung
- **Suche über mehrere Anbieter** – Brave Search + Serper mit automatischer Ausweichlösung
- **Mehrsprachige Erweiterung der Suchanfragen** – Suche über Sprachen und Regionen hinweg
- **Extraktion mit LLMs** – OpenRouter, Anthropic Claude (Prompt-Caching, optional Lesen ganzer PDFs), Ollama (lokal oder Cloud), DeepSeek, Groq, Mistral, Gemini, xAI, OpenAI, LM Studio, llama.cpp oder jede OpenAI-kompatible Adresse
- **Ergebnisse im Live-Stream** – Zeilen erscheinen in einer Live-Tabelle, sobald sie gefunden werden
- **PDF-Extraktion** – erkennt PDF-Dokumente in Suchergebnissen automatisch und extrahiert ihren Text
- **Modus „Bilder“** – eine eigene Pipeline, die statt Text nach Bildern sucht: Sie erzeugt 6–12 unterschiedliche Varianten der Anfrage (per LLM oder mit statischen Ersatzvarianten), führt Bildsuchen über Brave / Serper aus und bewertet dann jedes Ergebnis mit einem LLM nach einem strengen Relevanzschema (0,0–1,0); Bilder unter 0,7 werden verworfen, die übrigen nach Wert sortiert gespeichert und einzeln in Echtzeit in der Oberfläche aktualisiert; Budget, Abbruch und Abbruchbedingungen (Anzahl / Kosten / Zeit) werden durchgehend beachtet
- **Modus „Links“** – eine eigene Pipeline, die statt einer Tabelle die relevantesten Seiten liefert: Sie erzeugt Varianten der Suchanfrage aus Ihrer Anfrage, führt Websuchen über Brave / Serper aus (nach URL dedupliziert), lädt und parst jede Seite und lässt dann ein LLM den vollständigen Inhalt lesen und seine Relevanz für Ihre Anfrage bewerten (0,0–1,0), wobei es eine kurze Beschreibung erstellt; Seiten unter der Konfidenzschwelle werden verworfen, die übrigen nach Wert sortiert gespeichert und einzeln in Echtzeit in der Oberfläche aktualisiert; Budget, Abbruch und Abbruchbedingungen werden durchgehend beachtet
- **Modus „Recherche“** – ein agentischer Modus, der offene Fragen statt mit einer Tabelle in Markdown beantwortet: Das LLM steuert eine Tool-Calling-Schleife (im Web suchen, eine Seite als Markdown laden, nachdenken, antworten) bis zu einer festen Zahl von Schritten und überträgt jeden Schritt live in eine Zeitleiste; Denkschritte (`think`) und Fehler (`error`-Schritte, rot dargestellt) werden angezeigt, damit der Ablauf nachvollziehbar bleibt, und die endgültige Markdown-Antwort mit Quellen wird (bereinigt) dargestellt und lässt sich als `.md` exportieren; Budget, Abbruch und Abbruchbedingungen werden durchgehend beachtet
- **Formeln und Bilder in Rechercheantworten** – LaTeX-Formeln werden gezeichnet (MathJax, offline); der Agent kann nach Bildern suchen, sie ansehen und passende in seine Antwort aufnehmen, die mit dem Durchlauf gespeichert werden
- **Diagramme in Rechercheantworten** – Mermaid-Diagramme sowie Balken-, Linien-, Flächen-, Kreis-, Streu-, Netzdiagramme und Heatmaps, offline im Design der App gezeichnet; als SVG oder PNG speicherbar, im Vollbild zu öffnen
- **Nutzungsstatistik** – Geld, Token, Modellaufrufe, Brave- und Serper-Suchen und Datenvolumen nach Monat, Jahr und gesamtem Zeitraum, mit Aufschlüsselung nach Modell und Modus, den teuersten Durchläufen, Speicherbelegung, CSV-Export und einem Monatsbudget mit Warnungen
- **Quellen pro Zeile** – jede Zeile verlinkt auf die Seiten, aus denen sie extrahiert wurde
- **Deduplizierung von Objekten** – unscharfer Abgleich + LLM-gestützte Unterscheidung
- **Einstellbare Abbruchbedingungen** – Zielzahl der Zeilen, maximale Kosten, maximale Dauer
- **Verlauf der Durchläufe** – frühere Recherchedurchläufe durchsuchen, ansehen und erneut exportieren
- **Export** – CSV, JSON, XLSX mit vollständigen Quellmetadaten
- **Dunkles / helles Design** – Umschalter in der Seitenleiste
- **Infobereich (Tray)** – Benachrichtigungen bei Abschluss (Desktop)
- **Android-App** – dieselbe App auf Telefonen, mit einem Layout für kleine Bildschirme
- **Updates in der App** – neue Versionen werden mit ihren Änderungen angekündigt und nach Prüfung ihrer Signatur aus der App installiert (Desktop und Android)
- **Lokal zuerst** – alle Daten in SQLite, kein Cloud-Backend nötig

## Technologie-Stack

| Ebene | Technologie |
|-------|-------------|
| App-Hülle | Tauri v2 (Desktop und Android) |
| Backend | Rust (Tokio-Async-Laufzeit) |
| Frontend | Svelte 5 (SvelteKit-SPA) |
| UI-Gestaltung | Reines CSS mit Design-Tokens |
| Datenbank | SQLite (sqlx, WAL-Modus) |
| LLM | OpenRouter / Ollama (lokal) / Ollama Cloud / OpenAI-kompatible API (z. B. llama.cpp) |
| Suche | Brave Search API / Serper API |
| Symbole | Lucide |

## Voraussetzungen

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) (stabile Toolchain)
- [Voraussetzungen für Tauri v2](https://v2.tauri.app/start/prerequisites/) für Ihr Betriebssystem

## Erste Schritte

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

## Konfiguration

Beim ersten Start legt die App eine lokale SQLite-Datenbank mit Standardeinstellungen an. Öffnen Sie **Einstellungen** in der Seitenleiste, um Folgendes zu konfigurieren:

| Gruppe | Einstellungen |
|--------|---------------|
| **LLM-Anbieter** | Anbieter (OpenRouter / Ollama / Ollama Cloud / OpenAI-kompatibel), URL, Modell, API-Schlüssel, JSON-Modus, Temperatur, max. Token |
| **Suchanbieter** | Anbieter (Brave / Serper), API-Schlüssel, Ausweichschalter, Ergebnisse pro Suchanfrage |
| **Ausführung** | Parallele Seitenabrufe (Standard 8), parallele Extraktionen (Standard 3), Zeitlimit für Seiten, Ratenbegrenzung, robots.txt |
| **Qualität** | Gleichgewicht zwischen Präzision und Vollständigkeit, Strenge bei Belegen, Konfidenzschwelle, Ähnlichkeit für Duplikate |

Abbruchbedingungen (Zielzeilen, max. Kosten, max. Dauer) werden pro Anfrage auf der Seite „Anfrage“ festgelegt.

**Anderen Anbieter als Reserve nutzen** steuert das automatische Ausweichen zwischen Brave und Serper für Web- und Bildsuchen. Schalten Sie es aus, um nur den gewählten Hauptanbieter zu verwenden, auch wenn beide API-Schlüssel gespeichert sind. Die Auswahl gilt nach dem Speichern für neue Durchläufe; Wiederholungsversuche bleiben beim gewählten Anbieter. Ist die Option aktiv, wird der andere Anbieter nur nach einem Fehler verwendet und benötigt seinen gespeicherten Schlüssel.

Legen Sie unter **Einstellungen → Suche → Ergebnisse pro Suchanfrage** fest, wie viele Ergebnisse jede Suchanfrage in allen vier Modi, einschließlich Recherche, anfordert, und speichern Sie dann für neue Durchläufe. Anfragen an Brave begrenzen diesen Wert auf den dokumentierten Bereich des Endpunkts: [1–20 für die Websuche](https://api-dashboard.search.brave.com/api-reference/web/search/get) und [bis zu 200 für Bilder](https://api-dashboard.search.brave.com/documentation/services/image-search). Das gilt auch für früher gespeicherte zu große Werte. Größere Werte lösen keine automatischen zusätzlichen Seiten aus.

### Nutzung, Preise und Kostenlimits

**Nutzung & Kosten** unterscheidet vom Anbieter gemeldete Kosten, Schätzungen nach Modelltarifen, Teilsummen und unbekannte Kosten. Jeder Versuch beim Anbieter wird einzeln gezählt, einschließlich Wiederholungen, Ausweichsuchen, fehlgeschlagener Antworten mit Nutzungsdaten und abgebrochener Anfragen, deren endgültige Kosten nicht ermittelt werden konnten. Die Eingabe-/Ausgabenutzung stammt aus den Antworten der Anbieter; Denk-Token werden nicht zusätzlich zu den Ausgabe-Token ein zweites Mal berechnet.

- **OpenRouter:** bevorzugt die [gemeldeten Kosten der Anfrage](https://openrouter.ai/docs/cookbook/administration/usage-accounting). Sind sie nicht verfügbar, wird anhand der gemeldeten Token und der öffentlichen Katalogtarife des tatsächlich gelieferten Modells geschätzt.
- **Ollama Cloud:** schätzt den Verbrauch von Nutzungsguthaben anhand der [veröffentlichten Modelltarife](https://ollama.com/pricing), einschließlich gecachter Eingabe und erkannter Spitzenzeiten. Enthaltene Guthaben, Abozahlungen und Barzahlungen können von dieser Schätzung abweichen. Nicht erkannte oder nicht verfügbare Tarife bleiben unbekannt.
- **Eigene Modelltarife:** Legen Sie in den Einstellungen Eingabe-/Ausgabepreise in USD pro Million Token fest. Tarife gehören zu einem bestimmten Anbieter, Endpunkt und Modell. Ausdrückliche Nulltarife bedeuten kostenlose Nutzung; fehlende Tarife werden nie stillschweigend zu null. Veröffentlichte Tarife werden abgerufen, ohne gespeicherte manuelle Tarife zu ändern.
- **Suche:** Brave und Serper sind standardmäßig mit 0 USD pro 1.000 Anfragen angesetzt, sodass Suchkosten nicht in Schätzungen eingehen. Tragen Sie den tatsächlichen Tarif Ihres Plans ein, um sie einzubeziehen, oder lassen Sie den Preis leer, wenn er unbekannt ist. Fehlgeschlagene Versuche ohne Abrechnungsdaten bleiben ohne Preis. Änderungen an den Einstellungen gelten für neue Durchläufe; der gespeicherte Verlauf wird nicht neu berechnet. Beim Upgrade werden frühere leere Standardwerte einmalig zu null, wobei vorhandene Zahlenwerte und spätere bewusst leer gelassene Felder erhalten bleiben.

**Max. Kosten** eines Durchlaufs verhindert neue Anfragen an Anbieter, sobald die erfassten Kosten das Limit erreichen. Es ist keine garantierte Obergrenze der Rechnung: Schätzungen können von der Abrechnung abweichen, unbekannte Kosten können nicht einbezogen werden, und bereits laufende Anfragen können noch Kosten verursachen. Nutzen Sie Limits beim Anbieter, wenn eine strikte Obergrenze nötig ist. Die Abrechnung bleibt bei abgeschlossenen, fehlgeschlagenen, abgebrochenen und ergebnislosen Durchläufen erhalten. Frühere Einträge im Verlauf sind ausdrücklich als historische Schätzungen gekennzeichnet.

Die Deduplizierung behält die Zeilen-ID des ersten Mitglieds, führt die Werte und Quelldatensätze der Gruppe in einer Transaktion zusammen und gleicht die Live-Tabelle mit Verlauf und Export ab. Früher überschriebene historische Ergebnisse lassen sich nicht automatisch wiederherstellen; führen Sie diese Anfragen erneut aus, um aktuelle Ergebnisse zu erhalten.

In den Einstellungen bleiben **Speichern** und **Verwerfen** beim Scrollen sichtbar. Bereichslinks springen direkt zu den passenden Steuerelementen, und beim Verlassen mit ungespeicherten Änderungen stehen Speichern / Verwerfen / Bleiben zur Wahl. Änderungen, die während eines Speichervorgangs vorgenommen werden, bleiben für das nächste Speichern erhalten.

Ergebnisse erscheinen in einer durchgehenden, virtualisierten Tabelle mit lokaler Suche und Sortierung per Tastatur. Öffnen Sie die **Details** einer Zeile, um vollständige Werte zu lesen oder zu kopieren und ihre gespeicherten Quellen zu prüfen. In der Recherche steht die Antwort vor der aufklappbaren Aktivität. Bilder unterstützen Raster- und Listenansicht sowie Vorschauen per Tastatur; Export und Löschen zeigen Fortschritt und Ergebnis deutlich an. Das Leeren des Diagnosebereichs **Logs** setzt die Aktivität eines Durchlaufs nicht zurück.

Browser-Regressionstests verwenden synthetische Daten und simulierte Tauri-Befehle: Installieren Sie die Testbrowser mit `npx playwright install chromium webkit` und führen Sie dann `npm run test:ui` aus. Die bestehenden Frontend-Tests laufen mit `npm test`.

### API-Schlüssel

Sie benötigen mindestens einen API-Schlüssel für die Suche und einen konfigurierten LLM-Anbieter. Lokales Ollama und OpenAI-kompatible Server ohne Authentifizierung benötigen keinen LLM-API-Schlüssel:

| Dienst | Schlüssel erhältlich unter |
|--------|----------------------------|
| Brave Search | https://brave.com/search/api/ |
| Serper | https://serper.dev/ |
| OpenRouter | https://openrouter.ai/ |
| Ollama (lokal) | https://ollama.com/ – kein Schlüssel nötig |
| Ollama Cloud | https://ollama.com/settings/keys |
| OpenAI-kompatibel | Optionaler Schlüssel, je nach Server |

Wählen Sie den Anbieter unter **Einstellungen → LLM → Anbieter**, füllen Sie seine Felder aus und klicken Sie auf **Speichern**. Jeder Anbieter behält beim Wechsel sein eigenes Modell und seine Zugangsdaten.

Migrationen der Einstellungen laufen beim Start automatisch: Neu hinzugekommene Optionen erhalten Standardwerte, bestehende Werte bleiben erhalten. Umbenannte Einstellungen werden im Backend ausdrücklich migriert, bevor Standardwerte eingefügt werden; Änderungen an gespeicherten Formaten erfordern bei einer neuen Version ebenfalls Migrationscode.

Unten in den **Einstellungen** zeigt **Anwendungsdateien** den tatsächlichen Datenordner, die Datenbank `data.db` und den Log-Ordner. Mit **Pfad kopieren** kopieren Sie einen Speicherort in die Zwischenablage, mit **Ordner öffnen** öffnen Sie ihn im Dateimanager. Beim Speicherort der Datenbank wird der übergeordnete Ordner geöffnet. Einstellungen, Verlauf, Ergebnisse und Quellen liegen in `data.db`; Diagnoselogs werden getrennt gespeichert.

- **OpenRouter:** Geben Sie Ihren API-Schlüssel ein und wählen Sie ein Modell aus dem durchsuchbaren [OpenRouter-Katalog](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties). Tippen Sie, um Modell-IDs zu filtern, wählen Sie mit der Maus oder den Pfeiltasten und Enter, und laden Sie die Liste mit **Aktualisieren** neu. Der Katalog lässt sich vor Eingabe eines Schlüssels durchsuchen; für Anfragen an das Modell ist ein Schlüssel nötig.
- **Ollama (lokal):** URL `http://localhost:11434` und der Name eines installierten Modells.
- **Ollama Cloud:** URL `https://ollama.com`, ein Ollama-API-Schlüssel und ein Modell aus dem durchsuchbaren Serverkatalog. Klicken Sie in das Modellfeld und tippen Sie zum Filtern, wählen Sie dann mit der Maus oder den Pfeiltasten und Enter; **Aktualisieren** lädt den Katalog neu. Das Filtern ändert das gewählte Modell erst, wenn Sie ein Ergebnis auswählen. Der direkte Cloud-Zugriff nutzt die [native Ollama-API](https://docs.ollama.com/cloud). Die Cloud unterstützt derzeit keine [strukturierte Ausgabe](https://docs.ollama.com/capabilities/structured-outputs), daher wird JSON über Prompts angefordert und von der bestehenden Pipeline geparst.
- **OpenAI-kompatibel:** Geben Sie die API-Basis-URL einschließlich `/v1` und die Modell-ID des Servers ein. Für [llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server) lautet die Standard-URL `http://localhost:8080/v1`; verwenden Sie den Namen des geladenen Modells oder den konfigurierten Alias. Lassen Sie den API-Schlüssel leer, sofern Ihr Server keinen verlangt. Schalten Sie **JSON-Modus** aus, wenn der Server `response_format` ablehnt.

**Denken** (Reasoning-Aufwand) steuert, wie viel das Modell vor seiner Antwort nachdenkt. **Automatisch** behält die sicheren Standardwerte bei: Strukturierte Ollama-Anfragen schalten das Denken aus (GPT-OSS verwendet „Niedrig“), andere Anbieter behalten ihre Standardwerte. **Anbieterstandard** überlässt das Denken dem Server. **Aus**, **Ein**, **Niedrig**, **Mittel**, **Hoch** und **Maximal** fordern einen bestimmten Modus oder eine bestimmte Stufe an; die Unterstützung hängt von Modell und Anbieter ab. GPT-OSS unter Ollama akzeptiert Niedrig/Mittel/Hoch und kann „Aus“ oder „Maximal“ nicht verwenden. Änderungen gelten nach dem Speichern für neue Durchläufe.

**Max. Ausgabe-Token** ist ein Kontingent pro Anfrage, das sich Denk- und Antwort-Token teilen können. Es ist unabhängig von den Kosten- und Zeitlimits des Durchlaufs und vom Kontingent Ihres Anbieterkontos. Ein höherer Denkaufwand kann weniger Token für die Antwort übrig lassen.

Probleme bei Modellanfragen zeigen Anbieter, Modell, Phase, das angeforderte Ausgabelimit, eventuell gemeldete Token-Zahlen für Eingabe/Ausgabe/Denken und ob ein erneuter Versuch geplant wurde. Ausgabelimits, Kontextlimits, Anbieterkontingente, Ratenlimits, Authentifizierungs- und Netzwerkfehler haben jeweils eigene Erklärungen und Abhilfemaßnahmen. Probleme bleiben sichtbar, wenn einzelne Seiten übersprungen werden, und sind im Verlauf abrufbar. Andere Fehler zeigen eine kurze Erklärung, eine empfohlene Aktion und aufklappbare technische Details.

**Pausieren** hält die Verarbeitung in der aktuellen Phase an, auch während der Schema-Prüfung, und **Fortsetzen** macht weiter, ohne die Anfrage erneut zu senden. Eine bereits an ein entferntes Modell gesendete Anfrage kann dort auch während der Pause noch abgeschlossen werden. **Abbrechen** beendet die lokale Arbeit und schließt ausstehende Anfragen. Fehler der Pipeline werden angezeigt und als `failed` gespeichert; der zugrunde liegende Fehler steht im Log des Durchlaufs.

## Architektur

Das Backend verwendet eine **Pipeline-Zustandsmaschine** mit festen Rollen, die nacheinander orchestriert werden:

```
Query → Interpret → Plan Schema → [User Confirms] → Plan Searches
      → Expand Queries → Execute Search → Fetch Pages → Parse Documents
      → Extract Entities → Validate → Deduplicate → [Stop Check] → Done
```

### Rollen der Pipeline

| Rolle | LLM | Zweck |
|-------|-----|-------|
| QueryInterpreter | Ja | Anfrage in eine strukturierte Absicht zerlegen |
| SchemaPlanner | Ja | Tabellenspalten und ihre Typen vorschlagen |
| SearchPlanner | Ja | Suchanfragen für mehrere Sprachen/Regionen erzeugen |
| QueryExpander | Ja | Suchanfragen in die Zielsprachen übersetzen |
| SearchExecutor | Nein | Such-APIs aufrufen, infrage kommende URLs sammeln |
| Fetcher | Nein | HTTP-Abruf mit Ratenbegrenzung und robots.txt |
| DocumentParser | Nein | HTML → bereinigter Text (Entfernen von Boilerplate) |
| PdfParser | Nein | PDF → bereinigter Text (über pdf-extract) |
| Extractor | Ja | Text + Schema → strukturierte Zeilen |
| Validator | Teilweise | Schema-Konformität + semantische Prüfungen |
| Deduplicator | Teilweise | Unscharfer Abgleich (strsim) + LLM für Grenzfälle |
| LinkRanker | Ja | Seiteninhalt lesen und Relevanz bewerten (0,0–1,0) + Beschreibung erzeugen (Modus „Links“) |
| ResearchAgent | Ja | Die agentische Schleife aus Suchen/Laden/Denken/Antworten steuern und eine Markdown-Antwort erstellen (Modus „Recherche“) |
| StoppingController | Nein | Abbruchbedingungen auswerten (Zeilen, Budget, Zeit, Sättigung) |

### Zustandsmaschine

```
Pending → SchemaReview → Running ⇄ Paused → Completed / Failed / Cancelled
```

## Projektstruktur

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

## Datenbank

SQLite im WAL-Modus mit folgenden Tabellen:

| Tabelle | Zweck |
|---------|-------|
| `settings` | Benutzerkonfiguration (API-Schlüssel, Modelle, Vorgaben) |
| `runs` | Datensätze der Recherchedurchläufe mit Status und Statistik |
| `run_schemas` | Bestätigtes Tabellenschema pro Durchlauf |
| `search_queries` | Erzeugte Suchanfragen pro Durchlauf |
| `search_results` | Von Such-APIs gesammelte URLs |
| `fetched_pages` | Heruntergeladener Seiteninhalt |
| `entity_rows` | Extrahierte strukturierte Zeilen (JSON-Daten) |
| `row_sources` | Belege pro Zeile (URL, Titel, Ausschnitt) |
| `image_results` | Bewertete Bilder pro Durchlauf (Modus „Bilder“) |
| `link_results` | Bewertete relevante Links mit Beschreibungen und Werten (Modus „Links“) |
| `research_steps` | Tool-Aufrufe des Agenten pro Durchlauf – search/fetch/think/error (Modus „Recherche“) |
| `research_results` | Endgültige Markdown-Antwort pro Durchlauf (Modus „Recherche“) |
| `run_logs` | Ausführungslogs pro Durchlauf |

## Entwicklung

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

Benötigt die [Android-Voraussetzungen für Tauri](https://v2.tauri.app/start/prerequisites/#android): Android SDK und NDK (`ANDROID_HOME`, `NDK_HOME`), Java 17 (`JAVA_HOME`) und die Rust-Ziele (`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`). Schließen Sie ein Telefon mit aktiviertem USB-Debugging an oder starten Sie einen Emulator.

```bash
make android-devices    # check the phone or emulator is connected
make android-emulator   # start an emulator (AVD=<name> to pick one)
make android-apk        # build dist/Query2Table_<version>_android.apk
make android-run        # install it on the device and open the app
make android-logs       # follow the app's log
make android-dev        # live reload on the device
```

Builds werden mit dem Debug-Schlüssel signiert, sofern `src-tauri/gen/android/keystore.properties` nicht existiert. `make android-keystore` erstellt einen Release-Schlüssel und diese Datei und gibt die GitHub-Secrets aus (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_PASSWORD`, `ANDROID_KEY_ALIAS`), mit denen der Release-Workflow die APK signiert, die er jedem Release hinzufügt. Bewahren Sie den Schlüssel auf: Updates lassen sich nur über eine bestehende App installieren, wenn sie mit demselben Schlüssel signiert sind.

## Exportformate

| Format | Inhalt |
|--------|--------|
| **CSV** | Alle Spalten + Quellen als JSON-Spalte |
| **JSON** | Vollständige Zeilen mit verschachteltem Quellen-Array und Metadaten des Durchlaufs |
| **XLSX** | Formatierte Arbeitsmappe mit Datenblatt, Quellenblatt und Metadatenblatt |

## Lizenz

MIT
