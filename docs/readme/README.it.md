<p align="center">
  <img src="../../images/query2table-logo.png" alt="Logo di Query2Table" width="160">
</p>

<h1 align="center">Query2Table</h1>

<p align="center"><a href="../../README.md">English</a> · <a href="README.ru.md">Русский</a> · <a href="README.de.md">Deutsch</a> · <a href="README.fr.md">Français</a> · <a href="README.es.md">Español</a> · <b>Italiano</b> · <a href="README.pt.md">Português (Brasil)</a> · <a href="README.pl.md">Polski</a> · <a href="README.uk.md">Українська</a> · <a href="README.tr.md">Türkçe</a> · <a href="README.zh.md">简体中文</a> · <a href="README.ja.md">日本語</a> · <a href="README.ko.md">한국어</a></p>

<p align="center">
  <a href="https://github.com/hightemp/query2table/releases/latest"><img src="https://img.shields.io/github/v/release/hightemp/query2table?style=flat-square" alt="Release su GitHub"></a>
  <a href="https://github.com/hightemp/query2table/releases"><img src="https://img.shields.io/github/downloads/hightemp/query2table/total?style=flat-square" alt="Download da GitHub"></a>
  <a href="#licenza"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="Licenza: MIT"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8D8?style=flat-square&amp;logo=tauri" alt="Realizzato con Tauri v2"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust-orange?style=flat-square&amp;logo=rust" alt="Backend in Rust"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&amp;logo=svelte" alt="Frontend in Svelte 5"></a>
  <br>
  <img src="https://asdertasd.site/counter/query2table" alt="Visualizzazioni di Query2Table">
</p>

**Query2Table** è uno strumento di ricerca local-first per desktop (Linux, Windows, macOS) e Android. Descrivi i dati che ti servono: l’app cerca in modo autonomo su internet, in più fonti e in più lingue, scarica le pagine, estrae entità strutturate con gli LLM, elimina i duplicati e presenta il risultato in una tabella che si aggiorna in tempo reale, con i link alle fonti per ogni riga. Tutto gira sul tuo computer, senza un backend nel cloud.

Oltre alle tabelle offre altre tre modalità dedicate: **ricerca di immagini**, che trova immagini pertinenti e le ordina con un LLM in base alla pertinenza visiva; **ricerca di link**, che legge l’intero contenuto delle pagine candidate e restituisce solo i link più pertinenti, con descrizioni e punteggi di pertinenza generati da un LLM; e **ricerca**, una modalità ad agente in cui il modello cerca, legge le pagine, ragiona e scrive in autonomia una risposta in Markdown con le fonti.

L’app è disponibile anche per smartphone **Android**: la stessa app con le stesse modalità, impostazioni e cronologia, compilata dallo stesso codice e adattata allo schermo del telefono con una barra di schede in basso.

Chiedi qualcosa come *«Crea una tabella comparativa dei database e delle librerie open source per la ricerca vettoriale, con linguaggio, licenza, tipo di deployment, caso d’uso e sito web»* e guarda la tabella riempirsi in tempo reale.

![Tabella comparativa di database vettoriali open source con riscontri e fonti](../../screenshots/test-en-2026-10-06/44-table-results.png)

## Panoramica

### Chiedi in linguaggio naturale

Scegli un formato del risultato, descrivi cosa ti serve e imposta le condizioni di arresto. Per le tabelle, l’app propone delle colonne che puoi modificare prima che la ricerca inizi.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/01-query-table.png" alt="Nuova query con i quattro formati del risultato"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/05-schema-review.png" alt="Schema della tabella proposto, pronto per la revisione"></td>
  </tr>
</table>

### Ogni riga conserva i suoi riscontri

Apri una riga per leggere i valori completi, copiarli e controllare le pagine da cui sono stati estratti.

![Dettagli della riga con affidabilità dell’estrazione e fonti](../../screenshots/test-en-2026-10-06/45-row-details-and-sources.png)

### Ricerca con domande successive

L’agente cerca, legge le pagine e scrive una risposta con le fonti e un indice. Ogni passaggio è visibile in Attività, e le domande successive proseguono la conversazione con la sua cronologia.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/21-research-answer.png" alt="Risposta della ricerca con indice e domande successive suggerite"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/22-research-activity.png" alt="Attività della ricerca con passaggi numerati di ricerca e lettura"></td>
  </tr>
</table>

### Immagini e link

La ricerca di immagini ordina i risultati per pertinenza, con filtri per sito e dimensione; la ricerca di link legge ogni pagina e spiega perché è pertinente.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/28-images-grid.png" alt="Risultati della ricerca di immagini in una griglia giustificata"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/32-links-results.png" alt="Link pertinenti con descrizioni e livelli di pertinenza"></td>
  </tr>
</table>

### Cronologia e impostazioni

Cerca, filtra e riapri le esecuzioni passate, eseguile di nuovo o esportane diverse in una volta. Le impostazioni verificano le connessioni all’LLM e alla ricerca prima di iniziare.

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/18-history.png" alt="Cronologia delle esecuzioni raggruppata per giorno, con risultati, durata, costo e modello"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/43-settings-search-connection.png" alt="Impostazioni di ricerca con un test di connessione riuscito"></td>
  </tr>
</table>

### Temi chiari e scuri

<table>
  <tr>
    <td><img src="../../screenshots/test-en-2026-10-06/53-research-dark.png" alt="Risposta della ricerca nel tema scuro"></td>
    <td><img src="../../screenshots/test-en-2026-10-06/50-table-dark.png" alt="Tabella dei risultati nel tema scuro"></td>
  </tr>
</table>

## Download

Scarica la build più recente da [Releases](https://github.com/hightemp/query2table/releases/latest):

| Piattaforma | File |
|-------------|------|
| Linux | `.AppImage`, `.deb`, `.rpm` |
| Windows | `.msi`, `.exe` |
| macOS (Apple Silicon) | `.dmg` |
| Android 7.0+ | `Query2Table_<version>_android.apk` |

Per trasferire la configurazione da un computer al telefono, usa **Impostazioni → Esporta impostazioni…**, spunta **Includi chiavi API e proxy** e importa il file sul telefono. Un file così contiene le tue chiavi: tienilo privato ed eliminalo dopo l’importazione.

Su Android, apri l’APK scaricato e, quando richiesto, consenti l’installazione di app dal browser o dal gestore file. L’app Android conserva i dati sul telefono; non ha l’icona nell’area di notifica e le notifiche arrivano come normali notifiche Android. I server di modelli locali (Ollama, llama.cpp) si possono usare attraverso la rete tramite il loro indirizzo, ad es. `http://192.168.1.10:11434`.

## Funzionalità

- **Query in linguaggio naturale**: descrivi ciò che vuoi con parole tue
- **Deduzione automatica dello schema**: l’app propone le colonne della tabella; tu le confermi o le modifichi prima dell’esecuzione
- **Ricerca con più provider**: Brave Search + Serper con passaggio automatico all’altro in caso di errore
- **Ampliamento multilingue delle query**: ricerche in più lingue e aree geografiche
- **Estrazione con LLM**: OpenRouter, Anthropic Claude (prompt caching, lettura facoltativa dei PDF interi), Ollama (locale o Cloud), DeepSeek, Groq, Mistral, Gemini, xAI, OpenAI, LM Studio, llama.cpp o qualsiasi indirizzo compatibile con OpenAI
- **Risultati in streaming**: le righe compaiono in una tabella dal vivo man mano che vengono trovate
- **Estrazione da PDF**: rileva automaticamente i documenti PDF trovati nei risultati di ricerca e ne estrae il testo
- **Modalità ricerca di immagini**: una pipeline dedicata che cerca immagini invece di testo: genera 6–12 varianti diverse della query (con un LLM o con alternative statiche), esegue ricerche di immagini tramite Brave / Serper, poi valuta ogni risultato con un LLM secondo un criterio di pertinenza rigoroso (0.0–1.0); le immagini sotto 0.7 vengono scartate, le altre salvate in ordine di punteggio con aggiornamenti dell’interfaccia in tempo reale per ogni immagine; budget, annullamento e condizioni di arresto (numero / costo / tempo) sono rispettati in ogni fase
- **Modalità ricerca di link**: una pipeline dedicata che restituisce le pagine più pertinenti invece di una tabella: genera varianti di query di ricerca dalla tua richiesta, esegue ricerche web tramite Brave / Serper (senza duplicati per URL), scarica e analizza ogni pagina, poi fa leggere a un LLM l’intero contenuto per valutarne la pertinenza (0.0–1.0) rispetto alla query e scriverne una breve descrizione; le pagine sotto la soglia di affidabilità vengono scartate, le altre salvate in ordine di punteggio con aggiornamenti dell’interfaccia in tempo reale per ogni link; budget, annullamento e condizioni di arresto sono rispettati in ogni fase
- **Modalità ricerca**: una modalità ad agente che risponde a domande aperte in Markdown invece che con una tabella: l’LLM guida un ciclo di chiamate a strumenti (cercare sul web, scaricare una pagina come Markdown, ragionare, rispondere) fino a un numero fisso di passaggi, mostrando ogni passaggio in una cronologia dal vivo; i passaggi di ragionamento (`think`) e gli eventuali errori (passaggi `error`, mostrati in rosso) restano visibili perché il processo sia trasparente, e la risposta finale in Markdown con le fonti viene visualizzata (sanificata) ed è esportabile come `.md`; budget, annullamento e condizioni di arresto sono rispettati in ogni fase
- **Formule e immagini nelle risposte della ricerca**: le formule LaTeX vengono disegnate (MathJax, offline); l’agente può cercare immagini, osservarle e inserire quelle adatte nella risposta, conservate con l’esecuzione
- **Diagrammi e grafici nelle risposte della ricerca**: diagrammi Mermaid e grafici a barre, a linee, ad area, a torta, a dispersione, radar e mappe di calore, disegnati offline nel tema dell’app; si salvano come SVG o PNG e si aprono a schermo intero
- **Statistiche di utilizzo**: denaro, token, chiamate ai modelli, ricerche Brave e Serper e volume di dati per mese, anno e da sempre, con ripartizioni per modello e modalità, le esecuzioni più costose, lo spazio su disco, l’esportazione CSV e un budget mensile con avvisi
- **13 lingue dell’interfaccia**: inglese, russo, tedesco, francese, spagnolo, italiano, portoghese (Brasile), polacco, ucraino, turco, cinese (semplificato), giapponese e coreano; per impostazione predefinita si usa la lingua del sistema
- **Fonti per ogni riga**: ogni riga rimanda alle pagine da cui è stata estratta
- **Rimozione dei duplicati delle entità**: confronto approssimato + disambiguazione assistita dall’LLM
- **Condizioni di arresto configurabili**: numero di righe obiettivo, costo massimo, durata massima
- **Cronologia delle esecuzioni**: sfoglia, visualizza ed esporta di nuovo le ricerche passate
- **Esportazione**: CSV, JSON, XLSX con tutti i metadati delle fonti
- **Tema scuro / chiaro**: si cambia dalla barra laterale
- **Area di notifica**: notifiche di completamento (desktop)
- **App Android**: la stessa app sui telefoni, con un layout per schermi piccoli
- **Aggiornamenti dall’app**: le nuove versioni vengono annunciate con le loro novità e installate dall’app dopo la verifica della firma (desktop e Android)
- **Local-first**: tutti i dati in SQLite, nessun backend nel cloud necessario

## Tecnologie

| Livello | Tecnologia |
|---------|------------|
| Shell dell’app | Tauri v2 (desktop e Android) |
| Backend | Rust (runtime asincrono Tokio) |
| Frontend | Svelte 5 (SPA SvelteKit) |
| Stile dell’interfaccia | CSS semplice con design token |
| Database | SQLite (sqlx, modalità WAL) |
| LLM | OpenRouter / Ollama (locale) / Ollama Cloud / API compatibile con OpenAI (ad es. llama.cpp) |
| Ricerca | Brave Search API / Serper API |
| Icone | Lucide |

## Requisiti

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) (toolchain stable)
- [Requisiti di Tauri v2](https://v2.tauri.app/start/prerequisites/) per il tuo sistema operativo

## Per iniziare

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

## Configurazione

Al primo avvio l’app crea un database SQLite locale con le impostazioni predefinite. Apri **Impostazioni** dalla barra laterale per configurare:

| Gruppo | Impostazioni |
|--------|--------------|
| **Provider LLM** | Provider (OpenRouter / Ollama / Ollama Cloud / compatibile con OpenAI), URL, modello, chiave API, modalità JSON, temperatura, token massimi |
| **Provider di ricerca** | Provider (Brave / Serper), chiave API, provider di riserva, risultati per query |
| **Esecuzione** | Download in parallelo (predefinito 8), estrazioni in parallelo (predefinito 3), timeout di download, limitazione della frequenza, robots.txt |
| **Qualità** | Equilibrio tra precisione e richiamo, rigore dei riscontri, soglia di affidabilità, somiglianza dei duplicati |

Le condizioni di arresto (righe obiettivo, costo massimo, durata massima) si impostano per ogni query nella pagina Query.

**Usa l’altro provider come riserva** controlla il passaggio automatico tra Brave e Serper sia per le ricerche web sia per quelle di immagini. Disattivalo per usare solo il provider principale selezionato, anche se sono salvate entrambe le chiavi API. La scelta vale per le nuove esecuzioni dopo il salvataggio; i nuovi tentativi restano sul provider selezionato. Se è attivo, l’altro provider viene usato solo dopo un errore e richiede la sua chiave salvata.

Imposta **Impostazioni → Ricerca web → Risultati per query** per scegliere quanti risultati richiede ogni query di ricerca in tutte e quattro le modalità, Ricerca compresa, poi salva per le nuove esecuzioni. Le richieste a Brave limitano questo valore all’intervallo documentato dell’endpoint: [da 1 a 20 per la ricerca web](https://api-dashboard.search.brave.com/api-reference/web/search/get) e [fino a 200 per le immagini](https://api-dashboard.search.brave.com/documentation/services/image-search). Questo vale anche per valori troppo grandi salvati in precedenza. Valori più grandi non richiedono automaticamente pagine aggiuntive.

### Utilizzo, prezzi e limiti di spesa

**Utilizzo e costi** distingue gli addebiti segnalati dal provider, le stime basate sulle tariffe dei modelli, i totali parziali e i costi sconosciuti. Ogni tentativo verso un provider è conteggiato a parte, compresi i nuovi tentativi, le ricerche di riserva, i completamenti non riusciti con dati di utilizzo e le richieste annullate di cui non è stato possibile ottenere l’addebito finale. L’utilizzo di input e output proviene dalle risposte dei provider; i token di ragionamento non vengono addebitati una seconda volta in aggiunta ai token di output.

- **OpenRouter:** si preferisce il [costo della richiesta segnalato](https://openrouter.ai/docs/cookbook/administration/usage-accounting). Se non è disponibile, la stima si basa sui token segnalati e sulle tariffe pubbliche del catalogo per il modello effettivamente restituito.
- **Ollama Cloud:** il consumo di crediti d’uso è stimato dalle [tariffe pubblicate dei modelli](https://ollama.com/pricing), compresi l’input in cache e le fasce di prezzo di punta riconosciute. Crediti inclusi, abbonamenti e addebiti in denaro possono differire da questa stima. Le tariffe non riconosciute o non disponibili restano sconosciute.
- **Tariffe personalizzate dei modelli:** imposta nelle Impostazioni i costi di input/output in USD per milione di token. Le tariffe appartengono a un provider, un endpoint e un modello specifici. Le tariffe a zero esplicite indicano un uso gratuito; le tariffe mancanti non diventano mai zero di nascosto. Le tariffe pubblicate vengono scaricate senza modificare quelle manuali salvate.
- **Ricerca:** Brave e Serper hanno come valore predefinito 0 USD ogni 1.000 richieste, quindi gli addebiti della ricerca sono esclusi dalle stime. Imposta la tariffa effettiva del tuo piano per includerli, oppure lascia vuoto il prezzo se non lo conosci. I tentativi non riusciti senza dati di fatturazione restano senza prezzo. Le modifiche alle impostazioni valgono per le nuove esecuzioni; la cronologia salvata non viene ricalcolata. Con l’aggiornamento, i vecchi valori predefiniti vuoti diventano zero una sola volta, mantenendo le tariffe numeriche esistenti e i campi lasciati vuoti di proposito in seguito.

Il **Costo massimo** di un’esecuzione impedisce nuove richieste ai provider quando la spesa conteggiata raggiunge il limite. Non è un tetto garantito della fattura: le stime possono differire dalla fatturazione, gli addebiti sconosciuti non possono essere inclusi e le richieste già in corso possono comunque generare costi. Usa i limiti lato provider quando serve un tetto rigoroso. La contabilità viene conservata per esecuzioni completate, non riuscite, annullate e senza risultati. Le voci precedenti della cronologia sono indicate esplicitamente come stime storiche.

La rimozione dei duplicati conserva l’ID di riga del primo membro, unisce in modo transazionale i valori e le fonti del gruppo e allinea la tabella dal vivo con la cronologia e l’esportazione. I risultati storici sovrascritti in passato non possono essere ricostruiti automaticamente; esegui di nuovo quelle query per ottenere risultati aggiornati.

Nelle impostazioni **Salva** e **Scarta** restano visibili durante lo scorrimento. I link delle sezioni portano direttamente ai controlli pertinenti, e uscire con modifiche non salvate propone Salva / Scarta / Resta. Le modifiche fatte durante un salvataggio restano disponibili per il salvataggio successivo.

I risultati usano una tabella continua e virtualizzata con ricerca locale e ordinamento da tastiera. Apri i **Dettagli** di una riga per leggere o copiare i valori completi ed esaminarne le fonti salvate. La Ricerca mostra la risposta prima dell’attività espandibile. Le immagini supportano la vista a griglia o a elenco e le anteprime da tastiera; esportazione ed eliminazione mostrano avanzamento ed esiti in modo esplicito. Svuotare il pannello diagnostico dei **Log** non azzera l’attività dell’esecuzione.

I test di regressione nel browser usano dati sintetici e comandi Tauri simulati: installa i browser di test con `npx playwright install chromium webkit`, poi esegui `npm run test:ui`. I test frontend esistenti si eseguono con `npm test`.

### Chiavi API

Servono almeno una chiave API di ricerca e un provider LLM configurato. Ollama in locale e i server compatibili con OpenAI senza autenticazione non richiedono una chiave API per l’LLM:

| Servizio | Dove ottenere una chiave |
|----------|--------------------------|
| Brave Search | https://brave.com/search/api/ |
| Serper | https://serper.dev/ |
| OpenRouter | https://openrouter.ai/ |
| Ollama (locale) | https://ollama.com/ (nessuna chiave necessaria) |
| Ollama Cloud | https://ollama.com/settings/keys |
| Compatibile con OpenAI | Chiave facoltativa, a seconda del server |

Seleziona il provider in **Impostazioni → LLM → Provider**, compila i suoi campi e fai clic su **Salva**. Ogni provider mantiene il proprio modello e le proprie credenziali quando cambi.

Le migrazioni delle impostazioni vengono eseguite automaticamente all’avvio: le nuove opzioni ricevono i valori predefiniti e i valori esistenti vengono mantenuti. Le impostazioni rinominate vengono migrate esplicitamente nel backend prima dell’inserimento dei valori predefiniti; anche le modifiche ai formati salvati richiedono codice di migrazione quando si implementa una nuova versione.

In fondo alle **Impostazioni**, **File dell’applicazione** mostra la cartella dei dati effettiva, il database `data.db` e la cartella dei log. Usa **Copia percorso** per copiare una posizione negli appunti di sistema oppure **Apri cartella** per aprirla nel gestore file. Aprire la posizione del database apre la cartella che lo contiene. Impostazioni, cronologia, risultati e fonti si trovano in `data.db`; i log diagnostici sono salvati a parte.

- **OpenRouter:** inserisci la chiave API e seleziona un modello dal [catalogo di OpenRouter](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties), in cui si può cercare. Digita per filtrare gli ID dei modelli, scegli con il mouse o con i tasti freccia e Enter, e usa **Aggiorna** per ricaricare l’elenco. Il catalogo si può sfogliare prima di inserire una chiave; la chiave serve per l’inferenza.
- **Ollama (locale):** URL `http://localhost:11434` e il nome di un modello installato.
- **Ollama Cloud:** URL `https://ollama.com`, una chiave API di Ollama e un modello selezionato dal catalogo del server, in cui si può cercare. Fai clic sul campo del modello e digita per filtrare, poi scegli con il mouse o con i tasti freccia e Enter; **Aggiorna** ricarica il catalogo. Il filtro non cambia il modello selezionato finché non scegli un risultato. L’accesso diretto al cloud usa l’[API nativa di Ollama](https://docs.ollama.com/cloud). Il cloud al momento non supporta l’[output strutturato](https://docs.ollama.com/capabilities/structured-outputs), quindi il JSON viene richiesto tramite i prompt e analizzato dalla pipeline esistente.
- **Compatibile con OpenAI:** inserisci l’URL base dell’API incluso `/v1` e l’ID del modello sul server. Per [llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server), l’URL predefinito è `http://localhost:8080/v1`; usa il nome del modello caricato o l’alias configurato. Lascia vuota la chiave API a meno che il server non la richieda. Disattiva **Modalità JSON** se il server rifiuta `response_format`.

**Ragionamento** controlla quanto il modello ragiona prima di rispondere. **Auto** mantiene valori predefiniti sicuri: le richieste strutturate a Ollama disattivano il ragionamento (GPT-OSS usa Basso), mentre gli altri provider mantengono i loro valori predefiniti. **Predefinito del provider** lascia il ragionamento al server. **Disattivato**, **Attivato**, **Basso**, **Medio**, **Alto** e **Massimo** richiedono una modalità o un livello espliciti; il supporto dipende dal modello e dal provider. GPT-OSS su Ollama accetta Basso/Medio/Alto e non può usare Disattivato o Massimo. Le modifiche valgono per le nuove esecuzioni dopo il salvataggio.

**Token di output massimi** è una quota per richiesta, che può essere condivisa tra token di ragionamento e di risposta. È separata dai limiti di costo e di tempo dell’esecuzione e dalla quota dell’account presso il provider. Un ragionamento più intenso può lasciare meno token per la risposta.

I problemi nelle richieste ai modelli mostrano il provider, il modello, la fase, il limite di output richiesto, gli eventuali conteggi di token di input, output e ragionamento segnalati, e se è stato pianificato un nuovo tentativo. Limiti di output, limiti di contesto, quote del provider, limiti di frequenza, autenticazione ed errori di rete hanno spiegazioni e azioni correttive distinte. I problemi restano visibili quando singole pagine vengono saltate e sono disponibili nella Cronologia esecuzioni. Gli altri errori mostrano una breve spiegazione, un’azione da compiere e dettagli tecnici comprimibili.

**Pausa** sospende l’elaborazione alla fase corrente, revisione dello schema compresa, e **Riprendi** continua senza inviare di nuovo la richiesta. Una richiesta già inviata a un modello remoto può comunque completarsi lì durante la pausa. **Annulla** ferma il lavoro locale e chiude le richieste in sospeso. Gli errori della pipeline vengono mostrati e salvati come `failed`, con l’errore originale disponibile nel log dell’esecuzione.

## Architettura

Il backend usa una **macchina a stati della pipeline** con ruoli fissi orchestrati in sequenza:

```
Query → Interpret → Plan Schema → [User Confirms] → Plan Searches
      → Expand Queries → Execute Search → Fetch Pages → Parse Documents
      → Extract Entities → Validate → Deduplicate → [Stop Check] → Done
```

### Ruoli della pipeline

| Ruolo | LLM | Scopo |
|-------|-----|-------|
| QueryInterpreter | Sì | Analizza la query e ne ricava un intento strutturato |
| SchemaPlanner | Sì | Propone le colonne della tabella e i loro tipi |
| SearchPlanner | Sì | Genera query di ricerca per più lingue e aree geografiche |
| QueryExpander | Sì | Traduce le query nelle lingue di destinazione |
| SearchExecutor | No | Chiama le API di ricerca e raccoglie gli URL candidati |
| Fetcher | No | Download HTTP con limitazione della frequenza e robots.txt |
| DocumentParser | No | HTML → testo pulito (rimozione degli elementi di contorno) |
| PdfParser | No | PDF → testo pulito (tramite pdf-extract) |
| Extractor | Sì | Testo + schema → righe strutturate |
| Validator | In parte | Conformità allo schema + controlli semantici |
| Deduplicator | In parte | Confronto approssimato (strsim) + LLM per i casi limite |
| LinkRanker | Sì | Legge il contenuto della pagina, ne valuta la pertinenza (0.0–1.0) e genera una descrizione (modalità ricerca di link) |
| ResearchAgent | Sì | Guida il ciclo ad agente cerca/scarica/ragiona/rispondi e produce una risposta in Markdown (modalità ricerca) |
| StoppingController | No | Valuta le condizioni di arresto (righe, budget, tempo, saturazione) |

### Macchina a stati

```
Pending → SchemaReview → Running ⇄ Paused → Completed / Failed / Cancelled
```

## Struttura del progetto

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

## Database

SQLite in modalità WAL con le seguenti tabelle:

| Tabella | Scopo |
|---------|-------|
| `settings` | Configurazione dell’utente (chiavi API, modelli, preferenze) |
| `runs` | Record delle esecuzioni di ricerca con stato e statistiche |
| `run_schemas` | Schema della tabella confermato per ogni esecuzione |
| `search_queries` | Query di ricerca generate per ogni esecuzione |
| `search_results` | URL raccolti dalle API di ricerca |
| `fetched_pages` | Contenuto delle pagine scaricate |
| `entity_rows` | Righe strutturate estratte (dati JSON) |
| `row_sources` | Riscontri per ogni riga (URL, titolo, estratto) |
| `image_results` | Immagini ordinate per ogni esecuzione (modalità ricerca di immagini) |
| `link_results` | Link pertinenti ordinati con descrizioni e punteggi (modalità ricerca di link) |
| `research_steps` | Passaggi di chiamata agli strumenti dell’agente per ogni esecuzione: search/fetch/think/error (modalità ricerca) |
| `research_results` | Risposta finale in Markdown per ogni esecuzione (modalità ricerca) |
| `run_logs` | Log di esecuzione per ogni esecuzione |

## Sviluppo

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

Servono i [requisiti di Tauri per Android](https://v2.tauri.app/start/prerequisites/#android): Android SDK e NDK (`ANDROID_HOME`, `NDK_HOME`), Java 17 (`JAVA_HOME`) e i target Rust (`rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`). Collega un telefono con il debug USB attivo, oppure avvia un emulatore.

```bash
make android-devices    # check the phone or emulator is connected
make android-emulator   # start an emulator (AVD=<name> to pick one)
make android-apk        # build dist/Query2Table_<version>_android.apk
make android-run        # install it on the device and open the app
make android-logs       # follow the app's log
make android-dev        # live reload on the device
```

Le build sono firmate con la chiave di debug, a meno che non esista `src-tauri/gen/android/keystore.properties`. `make android-keystore` crea una chiave di release e quel file, e stampa i secret di GitHub (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_PASSWORD`, `ANDROID_KEY_ALIAS`) che il workflow di release usa per firmare l’APK allegato a ogni release. Conserva la chiave: gli aggiornamenti si installano sopra un’app esistente solo se firmati con la stessa chiave.

## Formati di esportazione

| Formato | Contenuto |
|---------|-----------|
| **CSV** | Tutte le colonne + le fonti come colonna JSON |
| **JSON** | Righe complete con array annidato delle fonti e metadati dell’esecuzione |
| **XLSX** | Cartella di lavoro formattata con un foglio dei dati, uno delle fonti e uno dei metadati |

## Licenza

MIT
