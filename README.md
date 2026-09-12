<p align="center">
  <img src="images/query2table-logo.png" alt="Query2Table logo" width="160">
</p>

<h1 align="center">Query2Table</h1>

<p align="center">
  <a href="https://github.com/hightemp/query2table/releases/latest"><img src="https://img.shields.io/github/v/release/hightemp/query2table?style=flat-square" alt="GitHub release"></a>
  <a href="https://github.com/hightemp/query2table/releases"><img src="https://img.shields.io/github/downloads/hightemp/query2table/total?style=flat-square" alt="GitHub downloads"></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License: MIT"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/built%20with-Tauri%20v2-24C8D8?style=flat-square&amp;logo=tauri" alt="Built with Tauri v2"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/backend-Rust-orange?style=flat-square&amp;logo=rust" alt="Rust backend"></a>
  <a href="https://svelte.dev/"><img src="https://img.shields.io/badge/frontend-Svelte%205-FF3E00?style=flat-square&amp;logo=svelte" alt="Svelte 5 frontend"></a>
  <br>
  <img src="https://asdertasd.site/counter/query2table" alt="Query2Table views">
</p>

**Query2Table** — local-first desktop research tool. You describe what data you need; the app autonomously searches the internet across multiple sources and languages, fetches pages, extracts structured entities with LLMs, deduplicates them, and presents the result as a live-updating table with row-level source links — all running on your machine without a cloud backend.

Beyond tables, it offers three more dedicated modes: **image search**, which finds and LLM-ranks relevant images by visual relevance; **link search**, which reads the full content of candidate pages and returns only the most relevant links with LLM-generated descriptions and relevance scores; and **research**, an agentic mode where the model autonomously searches, reads pages, reasons, and writes a sourced Markdown answer.

Ask something like *"Find all YC-backed AI startups from 2024 with their funding amount, CEO name, and website"* and watch the table fill up in real time.

![](screenshots/2026-03-28_09-50.png)
![](screenshots/2026-03-28_09-51.png)

## Features

- **Natural language queries** — describe what you want in plain English
- **Automatic schema inference** — the app proposes table columns; you confirm or edit before execution
- **Multi-provider search** — Brave Search + Serper with automatic fallback
- **Multilingual query expansion** — searches across languages and geographies
- **LLM-powered extraction** — OpenRouter (cloud) or Ollama (local) for entity extraction
- **Streaming results** — rows appear in a live table as they're found
- **PDF extraction** — automatically detects and extracts text from PDF documents found in search results
- **Image search mode** — a dedicated pipeline that searches images instead of text: generates 6–12 diverse query variations (LLM or static fallbacks), executes image searches via Brave / Serper, then ranks every result with an LLM using a strict relevance rubric (0.0–1.0); images scoring below 0.7 are dropped, the rest are stored sorted by score with real-time per-image UI updates; budget, cancellation, and stop conditions (count / cost / time) are respected throughout
- **Link search mode** — a dedicated pipeline that returns the most relevant pages instead of a table: generates search query variations from your request, executes web searches via Brave / Serper (deduplicated by URL), fetches and parses each page, then has an LLM read the full content and score its relevance (0.0–1.0) to your query while generating a short description; pages below the confidence threshold are dropped, the rest are stored sorted by score with real-time per-link UI updates; budget, cancellation, and stop conditions are respected throughout
- **Research mode** — an agentic mode that answers open-ended questions in Markdown instead of a table: the LLM drives a tool-calling loop (search the web, fetch a page as Markdown, think, answer) for up to a fixed number of steps, streaming each step to a live timeline; reasoning (`think`) steps and any failures (`error` steps, shown in red) are surfaced so the process stays transparent, and the final sourced Markdown answer is rendered (sanitized) and exportable as `.md`; budget, cancellation, and stop conditions are respected throughout
- **Row-level sources** — every row links back to the pages it was extracted from
- **Entity deduplication** — fuzzy matching + LLM-assisted disambiguation
- **Configurable stop conditions** — target row count, max cost, max duration
- **Run history** — browse, view, and re-export past research runs
- **Export** — CSV, JSON, XLSX with full source metadata
- **Dark / Light theme** — toggle in the sidebar
- **System tray** — completion notifications
- **Local-first** — all data in SQLite, no cloud backend required

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri v2 |
| Backend | Rust (Tokio async runtime) |
| Frontend | Svelte 5 (SvelteKit SPA) |
| UI framework | Skeleton UI + Tailwind CSS v4 |
| Database | SQLite (sqlx, WAL mode) |
| LLM | OpenRouter / Ollama (local) / Ollama Cloud / OpenAI-compatible API (e.g. llama.cpp) |
| Search | Brave Search API / Serper API |
| Icons | Lucide |

## Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

## Getting Started

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

## Configuration

On first launch the app creates a local SQLite database with default settings. Open **Settings** from the sidebar to configure:

| Group | Settings |
|-------|----------|
| **LLM Provider** | Provider (OpenRouter / Ollama / Ollama Cloud / OpenAI-compatible), URL, model, API key, JSON mode, temperature, max tokens |
| **Search Provider** | Provider (Brave / Serper), API key, fallback toggle, results per query |
| **Execution** | Parallel fetches (default 8), parallel extractions (default 3), fetch timeout, rate limiting, robots.txt |
| **Quality** | Precision/recall balance, evidence strictness, confidence threshold, dedup similarity |

Stop conditions (target rows, max cost, max duration) are set per-query on the Query page.

### API Keys

You need at least one search API key and a configured LLM provider. Local Ollama and unauthenticated OpenAI-compatible servers do not require an LLM API key:

| Service | Get a key at |
|---------|-------------|
| Brave Search | https://brave.com/search/api/ |
| Serper | https://serper.dev/ |
| OpenRouter | https://openrouter.ai/ |
| Ollama (local) | https://ollama.com/ — no key needed |
| Ollama Cloud | https://ollama.com/settings/keys |
| OpenAI-compatible | Optional key, depending on your server |

Select the provider in **Settings → LLM Provider**, fill in its fields, and click **Save**. Each provider keeps its own model and credentials when you switch.

- **OpenRouter:** enter your API key and select a model from the searchable [OpenRouter catalog](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties). Type to filter model IDs, choose with the mouse or arrow keys and Enter, and use **Refresh** to reload the list. The catalog can be browsed before entering a key; a key is required for inference.
- **Ollama (Local):** URL `http://localhost:11434` and the name of an installed model.
- **Ollama Cloud:** URL `https://ollama.com`, an Ollama API key, and a model selected from the searchable server catalog. Click the model field and type to filter, then choose with the mouse or arrow keys and Enter; **Refresh** reloads the catalog. Filtering does not change the selected model until you choose a result. Direct cloud access uses the [native Ollama API](https://docs.ollama.com/cloud). Cloud currently lacks [structured output support](https://docs.ollama.com/capabilities/structured-outputs), so JSON is requested through prompts and parsed by the existing pipeline.
- **OpenAI-compatible:** enter the API base URL including `/v1` and the server's model ID. For [llama.cpp](https://github.com/ggml-org/llama.cpp/tree/master/tools/server), the default URL is `http://localhost:8080/v1`; use the loaded model name or configured alias. Leave the API key empty unless your server requires one. Disable **JSON Mode** if the server rejects `response_format`.

## Architecture

The backend uses a **pipeline state machine** with fixed roles orchestrated in sequence:

```
Query → Interpret → Plan Schema → [User Confirms] → Plan Searches
      → Expand Queries → Execute Search → Fetch Pages → Parse Documents
      → Extract Entities → Validate → Deduplicate → [Stop Check] → Done
```

### Pipeline Roles

| Role | LLM | Purpose |
|------|-----|---------|
| QueryInterpreter | Yes | Parse query into structured intent |
| SchemaPlanner | Yes | Propose table columns and types |
| SearchPlanner | Yes | Generate search queries for multiple languages/geos |
| QueryExpander | Yes | Translate queries into target languages |
| SearchExecutor | No | Call search APIs, collect candidate URLs |
| Fetcher | No | HTTP fetch with rate limiting and robots.txt |
| DocumentParser | No | HTML → clean text (boilerplate removal) |
| PdfParser | No | PDF → clean text (via pdf-extract) |
| Extractor | Yes | Text + schema → structured rows |
| Validator | Partial | Schema conformance + semantic checks |
| Deduplicator | Partial | Fuzzy matching (strsim) + LLM for edge cases |
| LinkRanker | Yes | Read page content and score relevance (0.0–1.0) + generate description (link search mode) |
| ResearchAgent | Yes | Drive the agentic search/fetch/think/answer loop and produce a Markdown answer (research mode) |
| StoppingController | No | Evaluate stop conditions (rows, budget, time, saturation) |

### State Machine

```
Pending → SchemaReview → Running ⇄ Paused → Completed / Failed / Cancelled
```

## Project Structure

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

SQLite in WAL mode with the following tables:

| Table | Purpose |
|-------|---------|
| `settings` | User configuration (API keys, models, preferences) |
| `runs` | Research run records with status and stats |
| `run_schemas` | Confirmed table schema per run |
| `search_queries` | Generated search queries per run |
| `search_results` | URLs collected from search APIs |
| `fetched_pages` | Downloaded page content |
| `entity_rows` | Extracted structured rows (JSON data) |
| `row_sources` | Row-level evidence (URL, title, snippet) |
| `image_results` | Ranked images per run (image search mode) |
| `link_results` | Ranked relevant links with descriptions and scores (link search mode) |
| `research_steps` | Agent tool-call steps per run — search/fetch/think/error (research mode) |
| `research_results` | Final Markdown answer per run (research mode) |
| `run_logs` | Execution logs per run |

## Development

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

## Export Formats

| Format | Contents |
|--------|----------|
| **CSV** | All columns + sources as JSON column |
| **JSON** | Full rows with nested sources array and run metadata |
| **XLSX** | Formatted workbook with data sheet, sources sheet, and metadata sheet |

## License

MIT
