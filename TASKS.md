# Query2Table — Technical Implementation Plan

## 1. Executive Summary

Query2Table is a local-first desktop application that converts natural-language research queries into structured tables of entities. The system uses a controlled orchestrator with fixed roles — not free-form autonomous agents — to search the internet, fetch pages, extract structured data via LLMs, deduplicate results, and stream them into a live table with row-level evidence.

The stack is Tauri v2 (desktop shell) + Rust (backend orchestration) + Svelte/SvelteKit (frontend) + SQLite (local persistence). LLM access is via OpenRouter and/or Ollama. Search is via Brave Search API and Serper with user-configurable primary/fallback.

The MVP delivers a fully functional agent search pipeline: query understanding → schema proposal → user confirmation → search planning → execution → extraction → validation → deduplication → streaming table with sources. Resume, export, history, multilingual expansion, and full settings are MVP. JS-rendered pages, PDF parsing, and templates are Phase 2.

---

## 2. Assumptions

| # | Assumption | Rationale |
|---|-----------|-----------|
| A1 | User has at least one search API key (Brave or Serper) | App cannot search without one |
| A2 | User has OpenRouter API key OR local Ollama instance | LLM is required for schema planning and extraction |
| A3 | Average web page yields 2-4KB of cleaned text | Informs token budget calculations |
| A4 | LLM structured JSON output is reliable at >90% with gpt-5.4-mini | Basis for extraction pipeline; fallback handles failures |
| A5 | 80% of target pages are plain HTML (no JS rendering needed for MVP) | Justifies deferring headless browser to Phase 2 |
| A6 | Typical research query produces 20-200 result rows | Informs default stop condition and UI virtualization |
| A7 | User tolerates 30-120 seconds for initial results to appear | Sets latency expectations for pipeline |
| A8 | Brave Search returns max 20 results per query, Serper max 100 | Informs search planning batch sizes |
| A9 | OpenRouter pricing is per-token, similar to OpenAI | Basis for budget calculation |
| A10 | SQLite handles concurrent writes from async pipeline safely with WAL mode | Standard pattern for local apps |

---

## 3. Product Goal Restatement

Build a cross-platform desktop application that:
1. Accepts any natural-language research query (companies, people, events, jobs, laws, etc.)
2. Infers a candidate table schema and lets the user confirm/edit it
3. Executes adaptive multi-step internet search with multilingual expansion
4. Fetches and parses web pages, extracting structured entity rows via LLMs
5. Validates, deduplicates, and attaches sources to each row
6. Streams results into a live table as they are found
7. Stores everything locally with resume capability
8. Exports to CSV, JSON, XLSX

The system must be local-first, privacy-respecting, and configurable in precision/recall/cost tradeoffs.

---

## 4. Core Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    DESKTOP SHELL (Tauri v2)                  │
│  ┌───────────────────────┐  ┌────────────────────────────┐  │
│  │   FRONTEND (Svelte)   │  │    BACKEND (Rust/Tokio)    │  │
│  │                       │  │                            │  │
│  │  QueryInput           │◄─┤►  IPC Commands Layer       │  │
│  │  SchemaEditor     IPC │  │                            │  │
│  │  ResultsTable   Events│  │   Orchestrator             │  │
│  │  RunProgress          │  │    ├── QueryInterpreter    │  │
│  │  SettingsPanel        │  │    ├── SchemaPlanner       │  │
│  │  HistoryList          │  │    ├── SearchPlanner       │  │
│  │  LogViewer            │  │    ├── QueryExpander       │  │
│  │  ExportDialog         │  │    ├── SearchExecutor      │  │
│  │                       │  │    ├── Fetcher             │  │
│  │  Skeleton UI          │  │    ├── DocumentParser      │  │
│  │  TanStack Table       │  │    ├── Extractor           │  │
│  │                       │  │    ├── Validator            │  │
│  │                       │  │    ├── Deduplicator         │  │
│  │                       │  │    ├── StoppingController   │  │
│  │                       │  │    └── PersistenceManager   │  │
│  │                       │  │                            │  │
│  │                       │  │   Providers                │  │
│  │                       │  │    ├── OpenRouter Client   │  │
│  │                       │  │    ├── Ollama Client       │  │
│  │                       │  │    ├── Brave Search        │  │
│  │                       │  │    ├── Serper Search       │  │
│  │                       │  │    └── HTTP Fetcher        │  │
│  │                       │  │                            │  │
│  │                       │  │   Storage (SQLite/sqlx)    │  │
│  └───────────────────────┘  └────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Layers:**
1. **UI Layer** — Svelte + Skeleton UI + TanStack Table. Communicates with backend via Tauri IPC (commands + events).
2. **IPC Layer** — Tauri `#[command]` handlers. Thin translation between frontend requests and orchestrator actions.
3. **Orchestrator Layer** — State machine driving the pipeline. Controls execution order, parallelism, stop conditions.
4. **Roles Layer** — Fixed-function components. Each role has a single responsibility with defined input/output contracts.
5. **Providers Layer** — External API clients (search, LLM, HTTP). Behind traits for swappability.
6. **Storage Layer** — SQLite via sqlx. WAL mode for concurrent access. Migrations for schema versioning.

---

## 5. System Components

| Component | Technology | Purpose |
|-----------|-----------|---------|
| Desktop shell | Tauri v2 | Window, tray, IPC, auto-update, filesystem |
| Frontend | SvelteKit (SPA) | UI, user interaction, data display |
| Design system | Skeleton UI | Consistent UI components, dark/light theme |
| Table component | TanStack Table (Svelte) | Virtualized, streaming-compatible data table |
| Backend runtime | Rust + Tokio | Async orchestration, all business logic |
| LLM client | reqwest (OpenAI-compatible) | OpenRouter and Ollama API calls |
| Search client | reqwest | Brave Search API, Serper API |
| Page fetcher | reqwest + rate limiter | HTTP fetching with per-domain throttling |
| HTML parser | scraper + ammonia | DOM parsing + HTML sanitization |
| Text cleaner | custom (Rust) | Boilerplate removal, main content extraction |
| Database | SQLite via sqlx | Local persistence, WAL mode |
| Export | csv, serde_json, rust_xlsxwriter | CSV, JSON, XLSX export |
| Logging | tracing + tracing-subscriber | Structured logging to file + events |
| Fuzzy matching | strsim | String similarity for deduplication |

---

## 6. Agent / Role Model

The system uses a **controlled orchestrator with fixed roles**. No role improvises or calls other roles directly. The orchestrator dispatches work and manages state transitions.

| Role | Type | LLM? | Purpose |
|------|------|------|---------|
| **QueryInterpreter** | LLM-first | Yes | Parse NL query → structured intent (entity type, attributes, constraints, geo, language) |
| **SchemaPlanner** | LLM-first | Yes | Intent → proposed table schema (columns with names, types, descriptions) |
| **SearchPlanner** | LLM-first | Yes | Intent + schema → search plan (list of search queries × languages × geo targets) |
| **QueryExpander** | LLM-first | Yes | Expand/translate queries into target languages for multilingual coverage |
| **SearchExecutor** | Deterministic | No | Execute search queries via Brave/Serper APIs, collect candidate URLs |
| **Fetcher** | Deterministic | No | HTTP GET pages with rate limiting, timeout, robots.txt check |
| **DocumentParser** | Deterministic | No | Clean HTML → plain text (remove boilerplate, scripts, ads) |
| **Extractor** | LLM-first | Yes | Cleaned text + schema → structured entity rows (JSON) |
| **Validator** | Hybrid | Partial | Validate extracted rows (schema check deterministic, semantic check LLM) |
| **Deduplicator** | Hybrid | Partial | Fuzzy string matching (deterministic) + LLM for ambiguous cases |
| **StoppingController** | Deterministic | No | Evaluate stop conditions (row count, budget, time, saturation) |
| **PersistenceManager** | Deterministic | No | SQLite CRUD operations for all entities |
| **UIEventPublisher** | Deterministic | No | Emit Tauri events to stream updates to frontend |

**Orchestrator controls:**
- Which roles run and in what order
- Parallelism limits (how many fetchers, extractors run concurrently)
- State transitions (pending → running → completed/failed)
- Budget tracking across all LLM calls
- Resume logic on restart

---

## 7. End-to-End Pipeline

```
User enters query
       │
       ▼
  ┌─────────────┐
  │ Query       │  LLM: parse NL → structured intent
  │ Interpreter │  Output: {entity_type, attributes, constraints, geo, languages}
  └──────┬──────┘
         │
         ▼
  ┌─────────────┐
  │ Schema      │  LLM: intent → proposed columns
  │ Planner     │  Output: [{name, type, description, required}]
  └──────┬──────┘
         │
         ▼
  ┌─────────────┐
  │ User Schema │  UI: show proposed schema, user confirms/edits
  │ Confirmation│  Blocks until user confirms
  └──────┬──────┘
         │
         ▼
  ┌─────────────┐
  │ Search      │  LLM: intent + schema → search queries
  │ Planner     │  Output: [{query_text, language, geo_target}]
  └──────┬──────┘
         │
         ▼
  ┌─────────────┐
  │ Query       │  LLM: translate/expand queries across languages
  │ Expander    │  Output: expanded query list
  └──────┬──────┘
         │
         ▼
    ┌────┴────┐
    │  SEARCH  │◄──────────────────────────────────┐
    │  LOOP    │  Iterates until stop condition met │
    └────┬────┘                                     │
         │                                          │
         ▼                                          │
  ┌─────────────┐                                   │
  │ Search      │  Deterministic: call search API   │
  │ Executor    │  Output: candidate URLs            │
  └──────┬──────┘                                   │
         │                                          │
         ▼                                          │
  ┌─────────────┐                                   │
  │ Fetcher     │  Deterministic: HTTP GET (parallel,│
  │ (×8 workers)│  rate limited, robots.txt check)   │
  └──────┬──────┘                                   │
         │                                          │
         ▼                                          │
  ┌─────────────┐                                   │
  │ Document    │  Deterministic: clean HTML → text  │
  │ Parser      │                                    │
  └──────┬──────┘                                   │
         │                                          │
         ▼                                          │
  ┌─────────────┐                                   │
  │ Extractor   │  LLM: text + schema → entity rows │
  │ (×3 workers)│  Output: [{col1: val, ...}]        │
  └──────┬──────┘                                   │
         │                                          │
         ▼                                          │
  ┌─────────────┐                                   │
  │ Validator   │  Hybrid: schema check + optional   │
  │             │  LLM semantic validation           │
  └──────┬──────┘                                   │
         │                                          │
         ▼                                          │
  ┌─────────────┐                                   │
  │ Deduplicator│  Hybrid: fuzzy match + optional LLM│
  │             │                                    │
  └──────┬──────┘                                   │
         │                                          │
         ▼                                          │
  ┌─────────────┐     ┌──────────────┐              │
  │ Persistence │────►│ UI Event     │              │
  │ Manager     │     │ Publisher    │              │
  └──────┬──────┘     └──────────────┘              │
         │                                          │
         ▼                                          │
  ┌─────────────┐                                   │
  │ Stopping    │── continue? ─────────────────────►│
  │ Controller  │── stop? → finalize run             │
  └─────────────┘
```

---

## 8. Subtask Breakdown

### Phase 1: Foundation (MVP)

| ID | Subtask | Purpose | Inputs | Outputs | Owner Role | Complexity | Priority | Dependencies |
|----|---------|---------|--------|---------|------------|-----------|----------|-------------|
| T001 | Initialize Tauri v2 + SvelteKit project | Project scaffolding | None | Working Tauri+Svelte app shell | DevOps | Low | P0 | — |
| T002 | Configure Skeleton UI + theming | Design system setup | T001 | Themed app with dark/light switch | Frontend | Low | P0 | T001 |
| T003 | Set up Rust module structure | Backend organization | T001 | Module tree: commands, orchestrator, roles, providers, storage, export, utils | Backend | Low | P0 | T001 |
| T004 | Set up SQLite + sqlx migrations | Database foundation | T003 | DB connection pool, migration system, initial schema | Backend | Medium | P0 | T003 |
| T005 | Implement settings storage (Rust) | Persist user config | T004 | CRUD for settings table, typed settings struct | Backend | Low | P0 | T004 |
| T006 | Implement settings UI (Svelte) | User configures API keys, models, providers | T002, T005 | Settings page with all controls | Frontend | Medium | P0 | T002, T005 |
| T007 | Set up tracing + structured logging | Logging infrastructure | T003 | File logger + event emitter for GUI logs | Backend | Medium | P0 | T003 |
| T008 | Implement log viewer UI | Debug visibility | T002, T007 | Log panel with level filtering | Frontend | Low | P1 | T002, T007 |

### Phase 2: Providers (MVP)

| ID | Subtask | Purpose | Inputs | Outputs | Owner Role | Complexity | Priority | Dependencies |
|----|---------|---------|--------|---------|------------|-----------|----------|-------------|
| T009 | Define LlmProvider trait | Abstraction for LLM backends | T003 | Trait: `async fn chat_completion(messages, model, json_mode) → Result<String>` | Backend | Low | P0 | T003 |
| T010 | Implement OpenRouter client | Cloud LLM access | T009 | OpenRouter provider implementing LlmProvider trait | Backend | Medium | P0 | T009 |
| T011 | Implement Ollama client | Local LLM access | T009 | Ollama provider implementing LlmProvider trait | Backend | Medium | P0 | T009 |
| T012 | Implement LLM provider manager | Switch/fallback between LLM providers | T010, T011 | Manager that routes to configured provider per stage | Backend | Medium | P0 | T010, T011 |
| T013 | Define SearchProvider trait | Abstraction for search APIs | T003 | Trait: `async fn search(query, count, offset, geo, lang) → Result<Vec<SearchResult>>` | Backend | Low | P0 | T003 |
| T014 | Implement Brave Search client | Brave web search | T013 | Brave provider implementing SearchProvider | Backend | Medium | P0 | T013 |
| T015 | Implement Serper client | Google search via Serper | T013 | Serper provider implementing SearchProvider | Backend | Medium | P0 | T013 |
| T016 | Implement search provider manager | Primary/fallback switching | T014, T015 | Manager that routes to primary, falls back on error | Backend | Medium | P0 | T014, T015 |
| T017 | Implement HTTP fetcher with rate limiter | Page fetching | T003 | Fetcher: async fetch with per-domain rate limit (1 req/2s), timeout, User-Agent rotation | Backend | High | P0 | T003 |
| T018 | Implement robots.txt checker | Ethical scraping | T017 | Check + cache robots.txt per domain before fetching | Backend | Medium | P1 | T017 |

### Phase 3: Core Pipeline Roles (MVP)

| ID | Subtask | Purpose | Inputs | Outputs | Owner Role | Complexity | Priority | Dependencies |
|----|---------|---------|--------|---------|------------|-----------|----------|-------------|
| T019 | Implement HTML cleaner / DocumentParser | Content extraction | T017 | Clean text from HTML (remove scripts, styles, nav, ads, extract main content) | Backend | High | P0 | T017 |
| T020 | Implement QueryInterpreter role | Parse NL query | T012 | Structured intent: {entity_type, attributes[], constraints[], geo, languages[]} | Backend | High | P0 | T012 |
| T021 | Implement SchemaPlanner role | Propose table schema | T012, T020 | Schema: {columns: [{name, type, description, required}]} | Backend | High | P0 | T012 |
| T022 | Schema confirmation UI | User reviews/edits schema | T002, T021 | Schema editor component with add/remove/edit columns | Frontend | High | P0 | T002, T021 |
| T023 | Implement SearchPlanner role | Generate search queries | T012, T021 | Search plan: [{query_text, language, geo_target, priority}] | Backend | High | P0 | T012 |
| T024 | Implement QueryExpander role | Multilingual expansion | T012 | Expanded queries in multiple languages | Backend | Medium | P0 | T012 |
| T025 | Implement SearchExecutor role | Execute searches | T016 | Call search APIs, collect + normalize results | Backend | Medium | P0 | T016 |
| T026 | Implement Extractor role | LLM entity extraction | T012, T019 | Extract structured rows from cleaned page text using schema | Backend | High | P0 | T012, T019 |
| T027 | Implement Validator role | Validate extracted rows | T026 | Schema validation (deterministic) + optional LLM validation; confidence score | Backend | High | P0 | T026 |
| T028 | Implement Deduplicator role | Entity dedup | T027 | Fuzzy string matching (strsim), merge duplicate entities, group by dedup_group_id | Backend | High | P0 | T027 |
| T029 | Implement StoppingController | Stop condition evaluation | T004 | Check: row_count >= target OR budget_exceeded OR time_exceeded OR search_saturated | Backend | Medium | P0 | T004 |

### Phase 4: Orchestrator (MVP)

| ID | Subtask | Purpose | Inputs | Outputs | Owner Role | Complexity | Priority | Dependencies |
|----|---------|---------|--------|---------|------------|-----------|----------|-------------|
| T030 | Implement pipeline state machine | Orchestration core | T020-T029 | State machine: Pending → SchemaReview → Running → Completed/Failed/Cancelled | Backend | High | P0 | T020-T029 |
| T031 | Implement run state persistence | Resume capability | T004, T030 | Save pipeline state to SQLite; resume from last checkpoint on restart | Backend | High | P0 | T004, T030 |
| T032 | Implement UIEventPublisher | Stream updates to frontend | T030 | Emit Tauri events: row_added, progress_update, run_status_changed, log_entry | Backend | Medium | P0 | T030 |
| T033 | Implement parallel fetch worker pool | Concurrent fetching | T017, T030 | Tokio task pool (8 workers) pulling from URL queue | Backend | High | P0 | T017, T030 |
| T034 | Implement parallel extraction pool | Concurrent LLM extraction | T026, T030 | Tokio task pool (3 workers) for LLM calls | Backend | Medium | P0 | T026, T030 |
| T035 | Implement budget tracker | Cost control | T030 | Track API calls, estimated token usage, enforce budget limits | Backend | Medium | P0 | T030 |

### Phase 5: Frontend (MVP)

| ID | Subtask | Purpose | Inputs | Outputs | Owner Role | Complexity | Priority | Dependencies |
|----|---------|---------|--------|---------|------------|-----------|----------|-------------|
| T036 | Implement query input UI | Main interaction point | T002 | Query text area with "Run" button, query history autocomplete | Frontend | Medium | P0 | T002 |
| T037 | Implement results table with TanStack Table | Display streaming results | T002, T032 | Virtualized table, streaming row injection, column sorting/filtering | Frontend | High | P0 | T002 |
| T038 | Implement row detail panel | Show sources per row | T037 | Click row → show sources (URL, title, snippet) | Frontend | Medium | P0 | T037 |
| T039 | Implement run progress indicator | Execution feedback | T032 | Progress bar, stats (rows found, pages fetched, queries run), elapsed time | Frontend | Medium | P0 | T032 |
| T040 | Implement run controls | User control | T030 | Pause/Resume/Cancel buttons during run | Frontend | Low | P0 | T030 |
| T041 | Implement history page | Browse past runs | T004 | List of previous runs with query, date, row count, status; click to view results | Frontend | Medium | P1 | T004 |
| T042 | Implement export dialog | Data export | T002 | Modal with format selection (CSV/JSON/XLSX), file save dialog | Frontend | Low | P1 | T002 |
| T043 | Implement system tray + notifications | Background awareness | T001 | Tray icon, notification on run completion | Frontend | Medium | P1 | T001 |
| T044 | Implement app layout + navigation | Overall UX | T002 | Sidebar nav: Query, History, Settings; main content area | Frontend | Medium | P0 | T002 |

### Phase 6: Export & Polish (MVP)

| ID | Subtask | Purpose | Inputs | Outputs | Owner Role | Complexity | Priority | Dependencies |
|----|---------|---------|--------|---------|------------|-----------|----------|-------------|
| T045 | Implement CSV export | Export format | T004 | Write entity_rows + sources to CSV file | Backend | Low | P1 | T004 |
| T046 | Implement JSON export | Export format | T004 | Write entity_rows + sources to JSON file | Backend | Low | P1 | T004 |
| T047 | Implement XLSX export | Export format | T004 | Write entity_rows + sources to XLSX via rust_xlsxwriter | Backend | Medium | P1 | T004 |
| T048 | Error handling + retry logic | Robustness | T030 | Exponential backoff for API calls, graceful degradation, user-visible error messages | Backend | Medium | P0 | T030 |
| T049 | Integration testing | Quality | T030 | Test full pipeline with mock providers | Testing | High | P1 | T030 |
| T050 | CI/CD setup (GitHub Actions) | Automation | T001 | Multi-platform builds (Win/Mac/Linux), release artifacts | DevOps | Medium | P1 | T001 |
| T051 | Tauri auto-updater configuration | Updates | T001, T050 | Auto-update from GitHub Releases | DevOps | Medium | P2 | T050 |
| T052 | End-to-end testing | Quality | T049 | Full pipeline tests with real APIs (optional, gated by API keys) | Testing | High | P2 | T049 |

### Phase 2: Post-MVP Extensions

| ID | Subtask | Purpose | Inputs | Outputs | Owner Role | Complexity | Priority | Dependencies |
|----|---------|---------|--------|---------|------------|-----------|----------|-------------|
| T053 | JS-rendered page support | Handle SPAs | T017 | chromiumoxide integration: detect JS-heavy pages, fallback to headless render | Backend | High | P3 | T017 |
| T054 | PDF document parsing | Handle PDFs | T017 | pdf-extract integration: detect PDF URLs, extract text | Backend | Medium | P3 | T017 |
| T055 | Templates/presets system | Quick start | T004, T006 | Predefined templates: "Find companies", "Find conferences", "Find jobs", etc. | Full-stack | Medium | P3 | T004 |
| T056 | Advanced LLM dedup | Better entity resolution | T028 | LLM-assisted dedup for ambiguous cases (score 0.7-0.9) | Backend | Medium | P3 | T028 |
| T057 | Plugin system | Extensibility | T030 | User-defined extractors/validators as WASM or script plugins | Backend | High | P4 | T030 |

---

## 9. Dependency Graph

### Dependency List

```
T001 (Tauri+Svelte scaffold)
├── T002 (Skeleton UI) ── T006 (Settings UI) ── T022 (Schema UI)
│                      ── T008 (Log viewer)
│                      ── T036 (Query input)
│                      ── T037 (Results table) ── T038 (Row detail)
│                      ── T039 (Progress)
│                      ── T041 (History)
│                      ── T042 (Export dialog)
│                      ── T044 (Layout+nav)
├── T003 (Rust modules)
│   ├── T004 (SQLite) ── T005 (Settings storage) ── T006
│   │                 ── T029 (StoppingController)
│   │                 ── T031 (Run state persistence)
│   │                 ── T041, T045, T046, T047
│   ├── T007 (Logging) ── T008
│   ├── T009 (LLM trait) ── T010 (OpenRouter) ── T012 (LLM manager)
│   │                    ── T011 (Ollama)      ── T012
│   │                    T012 ── T020 (QueryInterpreter)
│   │                         ── T021 (SchemaPlanner)
│   │                         ── T023 (SearchPlanner)
│   │                         ── T024 (QueryExpander)
│   │                         ── T026 (Extractor)
│   │                         ── T027 (Validator)
│   ├── T013 (Search trait) ── T014 (Brave) ── T016 (Search manager)
│   │                       ── T015 (Serper) ── T016
│   │                       T016 ── T025 (SearchExecutor)
│   ├── T017 (HTTP fetcher) ── T018 (robots.txt)
│   │                       ── T019 (HTML cleaner)
│   │                       ── T033 (Fetch worker pool)
│   T028 (Deduplicator) ←── T027
│   T030 (Pipeline state machine) ←── T020-T029
│   T031 (Resume) ←── T004, T030
│   T032 (UIEventPublisher) ←── T030
│   T033 (Fetch pool) ←── T017, T030
│   T034 (Extract pool) ←── T026, T030
│   T035 (Budget tracker) ←── T030
├── T043 (System tray)
├── T050 (CI/CD) ── T051 (Auto-update)
```

### Recommended Implementation Order

**Sprint 1 — Scaffold + Foundation (Week 1-2)**
1. ~~T001 Initialize Tauri + SvelteKit~~ ✅
2. ~~T003 Rust module structure~~ ✅
3. ~~T002 Skeleton UI + theming~~ ✅
4. ~~T004 SQLite + migrations~~ ✅
5. ~~T005 Settings storage~~ ✅
6. ~~T007 Logging infrastructure~~ ✅
7. ~~T044 App layout + navigation~~ ✅
8. ~~T006 Settings UI~~ ✅
9. ~~T008 Log viewer panel~~ ✅

**Sprint 2 — Providers (Week 2-3)**
9. ~~T009 LLM trait~~ ✅
10. ~~T010 OpenRouter client~~ ✅
11. ~~T011 Ollama client~~ ✅
12. ~~T012 LLM provider manager~~ ✅
13. ~~T013 Search trait~~ ✅
14. ~~T014 Brave Search client~~ ✅
15. ~~T015 Serper client~~ ✅
16. ~~T016 Search provider manager~~ ✅
17. ~~T017 HTTP fetcher + rate limiter~~ ✅
18. ~~T018 robots.txt checker~~ ✅

**Sprint 3 — Pipeline Roles (Week 3-5)**
19. ~~T019 HTML cleaner / DocumentParser~~ ✅
20. ~~T020 QueryInterpreter~~ ✅
21. ~~T021 SchemaPlanner~~ ✅
22. ~~T023 SearchPlanner~~ ✅
23. ~~T024 QueryExpander~~ ✅
24. ~~T025 SearchExecutor~~ ✅
25. ~~T026 Extractor~~ ✅
26. ~~T027 Validator~~ ✅
27. ~~T028 Deduplicator~~ ✅
28. ~~T029 StoppingController~~ ✅

**Sprint 4 — Orchestrator + Streaming (Week 5-6)**
29. ~~T030 Pipeline state machine~~ ✅
30. ~~T031 Run state persistence (resume)~~ ✅
31. ~~T032 UIEventPublisher~~ ✅
32. ~~T033 Parallel fetch worker pool~~ ✅
33. ~~T034 Parallel extraction pool~~ ✅
34. ~~T035 Budget tracker~~ ✅

**Sprint 5 — Frontend Integration (Week 6-7)**
35. ~~T036 Query input UI~~ ✅
36. ~~T022 Schema confirmation UI~~ ✅
37. ~~T037 Results table (TanStack)~~ ✅
38. ~~T038 Row detail panel~~ ✅
39. ~~T039 Run progress indicator~~ ✅
40. ~~T040 Run controls (pause/resume/cancel)~~ ✅
41. ~~T008 Log viewer~~ ✅

**Sprint 6 — Export, History, Polish (Week 7-8)**
42. ~~T045 CSV export~~ ✅
43. ~~T046 JSON export~~ ✅
44. ~~T047 XLSX export~~ ✅
45. ~~T042 Export dialog~~ ✅
46. ~~T041 History page~~ ✅ (done in Sprint 5)
47. ~~T043 System tray + notifications~~ ✅
48. ~~T048 Error handling + retries~~ ✅

**Sprint 7 — Testing + CI/CD (Week 8-9)**
49. T049 Integration tests
50. T050 CI/CD setup
51. T051 Auto-updater
52. T052 E2E tests

---

## 10. Data Model

### Entity-Relationship

```
settings (key-value)

runs ─────────┬── run_schemas
              ├── search_queries ── search_results ── fetched_pages
              ├── entity_rows ──── row_sources
              └── run_logs
```

### Table Definitions

```sql
-- User settings (API keys, preferences, model config)
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Research runs
CREATE TABLE runs (
    id TEXT PRIMARY KEY,                -- UUID v4
    query TEXT NOT NULL,                -- Original NL query
    status TEXT NOT NULL DEFAULT 'pending',
        -- pending | schema_review | running | paused | completed | failed | cancelled
    config TEXT NOT NULL,               -- JSON: snapshot of execution config at run start
    stats TEXT,                         -- JSON: {total_rows, total_searches, total_fetches, 
                                        --        total_llm_calls, estimated_cost, total_pages_fetched}
    error TEXT,                         -- Error message if failed
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    completed_at INTEGER
);

-- Schema for each run (confirmed by user)
CREATE TABLE run_schemas (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    columns TEXT NOT NULL,              -- JSON: [{name, type, description, required}]
                                        -- type: text | number | url | date | boolean | list
    confirmed INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Search queries generated by SearchPlanner + QueryExpander
CREATE TABLE search_queries (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    query_text TEXT NOT NULL,
    language TEXT DEFAULT 'en',
    geo_target TEXT,                    -- Country code: US, DE, IL, etc.
    provider TEXT,                      -- brave | serper
    status TEXT NOT NULL DEFAULT 'pending',
        -- pending | executing | completed | failed
    result_count INTEGER DEFAULT 0,
    batch_number INTEGER DEFAULT 0,    -- Which iteration of adaptive search
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    executed_at INTEGER
);

-- Individual search results (URLs discovered)
CREATE TABLE search_results (
    id TEXT PRIMARY KEY,
    search_query_id TEXT NOT NULL REFERENCES search_queries(id) ON DELETE CASCADE,
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    title TEXT,
    snippet TEXT,
    rank INTEGER,                      -- Position in search results
    status TEXT NOT NULL DEFAULT 'pending',
        -- pending | fetching | fetched | failed | skipped | robots_blocked
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Fetched page content (cleaned text stored, not raw HTML)
CREATE TABLE fetched_pages (
    id TEXT PRIMARY KEY,
    search_result_id TEXT NOT NULL REFERENCES search_results(id) ON DELETE CASCADE,
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    status TEXT NOT NULL,              -- success | failed | timeout | blocked | too_large
    content_text TEXT,                 -- Cleaned main content text
    content_length INTEGER,            -- Length in chars
    fetch_duration_ms INTEGER,
    http_status INTEGER,
    fetched_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Extracted entity rows (the main results!)
CREATE TABLE entity_rows (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    data TEXT NOT NULL,                 -- JSON: {col1: value1, col2: value2, ...}
    confidence REAL NOT NULL DEFAULT 0.0,  -- 0.0 to 1.0
    status TEXT NOT NULL DEFAULT 'raw',
        -- raw | validated | deduplicated | final
    dedup_group_id TEXT,               -- Groups duplicate entities together
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Sources / evidence for each entity row
CREATE TABLE row_sources (
    id TEXT PRIMARY KEY,
    entity_row_id TEXT NOT NULL REFERENCES entity_rows(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    title TEXT,
    snippet TEXT,                      -- Relevant text excerpt from page
    fetched_page_id TEXT REFERENCES fetched_pages(id),
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Structured run logs (for debugging + log viewer)
CREATE TABLE run_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT REFERENCES runs(id) ON DELETE CASCADE,
    level TEXT NOT NULL,               -- DEBUG | INFO | WARN | ERROR
    role TEXT,                         -- Which role produced this: query_interpreter, fetcher, etc.
    message TEXT NOT NULL,
    details TEXT,                      -- JSON: additional structured context
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Indexes for performance
CREATE INDEX idx_runs_status ON runs(status);
CREATE INDEX idx_search_queries_run_id ON search_queries(run_id);
CREATE INDEX idx_search_results_run_id ON search_results(run_id);
CREATE INDEX idx_search_results_status ON search_results(status);
CREATE INDEX idx_fetched_pages_run_id ON fetched_pages(run_id);
CREATE INDEX idx_entity_rows_run_id ON entity_rows(run_id);
CREATE INDEX idx_entity_rows_status ON entity_rows(status);
CREATE INDEX idx_row_sources_entity_row_id ON row_sources(entity_row_id);
CREATE INDEX idx_run_logs_run_id ON run_logs(run_id);
CREATE INDEX idx_run_logs_level ON run_logs(level);
```

---

## 11. Storage Design

**Database:** SQLite 3 with WAL mode (Write-Ahead Logging) for concurrent reads during writes.

**Connection pool:** sqlx `SqlitePool` with 5 connections max.

**Migration strategy:** sqlx embedded migrations (`sqlx::migrate!("./migrations")`).

**Data sizing estimates:**
- Per run: ~1KB (run record) + ~500B × N search queries + ~200B × M search results + ~3KB × P fetched pages (cleaned text) + ~500B × R entity rows + ~300B × S row sources
- For a typical run with 50 rows: ~500KB total
- History of 1000 runs: ~500MB (no limit, user manages)

**JSON fields:** Used for flexible schema (config, columns, entity data, stats). Queried via SQLite JSON functions when needed.

**Backup strategy:** SQLite file is self-contained. User can copy the `.db` file. Future: export history to archive.

**File locations:**
- Database: `{app_data_dir}/query2table/data.db`
- Logs: `{app_data_dir}/query2table/logs/`
- Exports: User-chosen directory via save dialog

---

## 12. Async Execution Model

```
                    Orchestrator (main task)
                           │
            ┌──────────────┼──────────────┐
            │              │              │
     Search Queue    Fetch Queue    Extract Queue
     (sequential)    (8 workers)    (3 workers)
            │              │              │
            ▼              ▼              ▼
    SearchExecutor    Fetcher×8     Extractor×3
    (calls APIs)     (HTTP GET)    (LLM calls)
```

**Tokio task structure:**
- **Orchestrator task:** Main `tokio::spawn` that drives the pipeline state machine. Runs for the duration of a run.
- **Fetch worker pool:** 8 `tokio::spawn` tasks consuming from a bounded `tokio::sync::mpsc` channel. Each worker fetches one URL at a time with rate limiting.
- **Extract worker pool:** 3 `tokio::spawn` tasks consuming from another bounded channel. Each worker sends cleaned text to LLM and parses the response.
- **Event emitter:** Dedicated `tokio::spawn` that batches events and emits them to the frontend every 200ms (debounced).

**Channels:**
- `url_tx/url_rx` — `mpsc::channel(100)` — SearchExecutor → Fetch workers
- `page_tx/page_rx` — `mpsc::channel(50)` — Fetch workers → Extract workers
- `row_tx/row_rx` — `mpsc::channel(200)` — Extract workers → Orchestrator (for validation/dedup/persist)
- `event_tx/event_rx` — `mpsc::channel(500)` — All roles → UIEventPublisher

**Backpressure:** Bounded channels provide natural backpressure. If extraction is slow, fetching pauses automatically.

**Cancellation:** `tokio_util::sync::CancellationToken` shared across all tasks. User clicking "Cancel" triggers the token, all tasks wind down gracefully.

**Pause/Resume:** `tokio::sync::Notify` — orchestrator waits on notify when paused, resumes when user clicks Resume.

---

## 13. LLM Usage by Stage

| Stage | Why LLM is needed | Input contract | Output contract | Structured output format | Failure fallback | User chooses model? |
|-------|-------------------|----------------|-----------------|-------------------------|------------------|-------------------|
| **QueryInterpreter** | NL query is ambiguous; need to extract entity type, attributes, constraints, geographic/language scope | `{query: string}` | `{entity_type: string, attributes: string[], constraints: {key: string, value: string}[], geo_targets: string[], languages: string[], description: string}` | JSON (response_format: json_object) | Treat entire query as keyword search; entity_type="generic", attributes=["name","description","url"] | Yes |
| **SchemaPlanner** | Need to infer appropriate table columns from the intent; different entity types need different schemas | `{entity_type: string, attributes: string[], constraints: object, description: string}` | `{columns: [{name: string, type: "text"\|"number"\|"url"\|"date"\|"boolean"\|"list", description: string, required: boolean}]}` | JSON | Fall back to default schema: [name, description, url, source] | Yes |
| **SearchPlanner** | Need intelligent search query generation; simple keyword concatenation misses results | `{entity_type: string, attributes: string[], constraints: object, geo_targets: string[], schema_columns: Column[]}` | `{queries: [{text: string, language: string, geo_target: string, priority: number}], rationale: string}` | JSON | Generate simple queries: `"{entity_type} {constraint1} {constraint2}"` | Yes |
| **QueryExpander** | Translate queries across languages for broader coverage; multilingual entity names | `{queries: [{text: string, source_language: string}], target_languages: string[]}` | `{expanded: [{original: string, translated: string, language: string}]}` | JSON | Skip expansion; use only original language queries | Yes |
| **Extractor** | Unstructured text → structured rows; pattern variety too high for regex | `{text: string (max ~3000 tokens), schema: Column[], entity_type: string, extraction_hint: string}` | `{entities: [{col1: value, col2: value, ...}], extraction_quality: "high"\|"medium"\|"low"}` | JSON | Skip page; log warning | Yes |
| **Validator** | Semantic validation — are extracted values plausible? (e.g., is "www.google.com" a plausible website for company X?) | `{entity: object, schema: Column[], source_url: string, source_snippet: string}` | `{valid: boolean, confidence: number, issues: string[]}` | JSON | Schema-only validation (type checks, required field checks) — skip semantic validation | Yes |
| **Deduplicator** (ambiguous cases only) | When fuzzy score is 0.7-0.9 and deterministic matching is uncertain | `{entity_a: object, entity_b: object, schema: Column[], similarity_score: number}` | `{same_entity: boolean, confidence: number, reason: string}` | JSON | Use deterministic threshold (>0.85 = same, <0.85 = different) | No |

**Token budget estimates per run (50 target rows):**
- QueryInterpreter: ~500 tokens (1 call)
- SchemaPlanner: ~800 tokens (1 call)
- SearchPlanner: ~1,000 tokens (1-2 calls)
- QueryExpander: ~1,500 tokens (1 call per language)
- Extractor: ~2,000 tokens × 80 pages = ~160,000 tokens (dominant cost)
- Validator: ~500 tokens × 60 entities = ~30,000 tokens
- Deduplicator: ~400 tokens × 10 ambiguous pairs = ~4,000 tokens
- **Total estimate: ~200,000 tokens per run**

---

## 14. Search Strategy Design

### Multi-Step Adaptive Search

```
Step 1: Initial Search Plan
  SearchPlanner generates 5-15 search queries from the intent
  Queries vary by: phrasing, specificity, geo target

Step 2: Query Expansion
  QueryExpander translates queries into target languages
  Typical: EN, + languages relevant to geo targets

Step 3: Batch Execution (Batch 0)
  SearchExecutor runs queries through primary provider
  Collects candidate URLs, deduplicates by URL

Step 4: Fetch + Extract (parallel)
  Fetcher gets pages → DocumentParser cleans → Extractor extracts
  New rows stream to table

Step 5: Evaluate Stop Conditions
  StoppingController checks:
    - row_count >= target? → STOP
    - budget_exceeded? → STOP
    - time_exceeded? → STOP
    - All conditions passed → continue

Step 6: Saturation Check
  If < 5% new unique rows from last batch of pages → search is saturated
  If saturated for 3 consecutive batches → STOP

Step 7: Adaptive Refinement (Batch N+1)
  If not stopped and more queries available:
    - Use remaining queries from plan
    - OR: generate new queries based on gaps in results
    - Go to Step 3
```

### Provider Strategy

- User selects primary provider in settings (Brave or Serper)
- On API error (429, 500, timeout), automatically try fallback provider
- Per-provider rate limiting honored
- Results from both providers normalized to common `SearchResult` struct

### Search Parameters

| Parameter | Brave | Serper |
|-----------|-------|--------|
| Query | `q` (GET param) | `q` (POST body) |
| Count | `count` (max 20) | `num` (max 100) |
| Offset/Page | `offset` | `page` |
| Country | `country` | `gl` |
| Language | `search_lang` | `hl` |
| Freshness | `freshness` (pd/pw/pm) | `tbs` (qdr:d/w/m) |

---

## 15. Extraction and Validation Design

### Extraction Pipeline

```
Cleaned text (max 6000 chars)
       │
       ▼
  ┌─────────────────────┐
  │ Pre-check: is text  │
  │ relevant to query?  │  ← Simple keyword check (deterministic)
  │ Skip if irrelevant  │
  └──────┬──────────────┘
         │
         ▼
  ┌─────────────────────┐
  │ Chunking (if needed)│  ← Split text >4000 tokens into chunks
  │ Overlap: 200 chars  │    with 200 char overlap
  └──────┬──────────────┘
         │
         ▼
  ┌─────────────────────┐
  │ LLM Extraction      │  ← Send: system prompt + schema + text chunk
  │ (Extractor role)    │  ← Receive: JSON array of entities
  └──────┬──────────────┘
         │
         ▼
  ┌─────────────────────┐
  │ Schema Validation   │  ← Deterministic: check types, required fields
  │ (Validator role)    │     Drop invalid rows, log warnings
  └──────┬──────────────┘
         │
         ▼
  ┌─────────────────────┐
  │ Confidence Scoring  │  ← Based on: field completeness, source quality,
  │                     │     extraction_quality flag from LLM
  └──────┬──────────────┘
         │
         ▼
  ┌─────────────────────┐
  │ Source Attachment    │  ← Attach URL + title + relevant snippet
  └─────────────────────┘
```

### Extraction Prompt Template

```
System: You are a structured data extractor. Extract entities matching the schema 
from the provided text. Return ONLY valid JSON. If no entities found, return 
{"entities": [], "extraction_quality": "low"}.

Schema:
{columns_json}

Entity type: {entity_type}
Extraction hint: {description}

Rules:
- Extract ONLY entities that match the schema
- Every entity MUST have all required fields filled
- Use null for unknown optional fields
- Do NOT invent data — only extract what is explicitly stated in the text
- For URLs: use full absolute URLs
- For dates: use ISO 8601 format

Text:
{cleaned_text}

Respond with JSON:
{"entities": [...], "extraction_quality": "high|medium|low"}
```

### Validation Rules (Deterministic)

| Check | Logic | Action on fail |
|-------|-------|---------------|
| Required fields present | All `required: true` columns have non-null values | Drop row |
| Type match | `number` is numeric, `url` looks like URL, `date` parseable | Set confidence -= 0.2 |
| Field length | Text fields < 1000 chars, no HTML fragments | Truncate / clean |
| Duplicate within page | Same entity extracted twice from one page | Keep highest confidence |

---

## 16. Deduplication Strategy

### Three-Tier Dedup

```
Tier 1: Exact URL dedup (before fetching)
  ├── Normalize URLs (strip tracking params, www prefix, trailing slash)
  ├── Hash and check against seen set
  └── Skip already-fetched URLs

Tier 2: Fuzzy entity dedup (after extraction)
  ├── Normalize entity key fields (lowercase, strip whitespace, remove articles)
  ├── Compute Jaro-Winkler similarity on key fields
  ├── Score > 0.95 → auto-merge (keep higher confidence)
  ├── Score 0.70-0.95 → ambiguous (potential LLM check in Phase 2, 
  │                      for MVP: merge if > 0.85, keep separate if < 0.85)
  └── Score < 0.70 → different entities

Tier 3: Cross-language dedup
  ├── Detect script/language of entity name
  ├── Transliterate to Latin (unicode_normalization)
  ├── Apply Tier 2 on transliterated forms
  └── Also check URL field — same URL = same entity regardless of name
```

### Key Fields for Dedup

The deduplicator uses the first 2-3 columns marked as `required` in the schema as "key fields" for similarity comparison. If the schema has a `url` or `website` column, that is always included as a key field (exact URL match = same entity).

### Merge Strategy

When merging duplicate entities:
1. Keep the row with higher confidence score
2. Merge sources — the merged entity gets sources from both originals
3. Fill null fields from the other entity (higher coverage)
4. Assign shared `dedup_group_id`

---

## 17. Stopping Logic

### Stop Conditions (evaluated after each batch)

| Condition | Parameter | Default | Logic |
|-----------|-----------|---------|-------|
| **Target rows reached** | `target_row_count` | 50 | `final_rows >= target` |
| **Budget exceeded** | `max_budget_usd` | 1.00 | `estimated_cost >= max_budget` |
| **Time exceeded** | `max_duration_seconds` | 600 (10 min) | `elapsed >= max_duration` |
| **Search saturated** | `saturation_threshold` | 0.05 | `new_rows / fetched_pages < threshold` for 3 consecutive batches |
| **All queries exhausted** | — | — | No more search queries to execute |
| **User cancelled** | — | — | CancellationToken triggered |

### Evaluation Order

```rust
fn should_stop(&self, state: &RunState) -> StopReason {
    if state.cancelled { return StopReason::UserCancelled; }
    if state.final_rows >= state.config.target_row_count { return StopReason::TargetReached; }
    if state.estimated_cost >= state.config.max_budget_usd { return StopReason::BudgetExceeded; }
    if state.elapsed() >= state.config.max_duration { return StopReason::TimeExceeded; }
    if state.consecutive_saturated_batches >= 3 { return StopReason::SearchSaturated; }
    if state.pending_queries == 0 && state.pending_urls == 0 { return StopReason::AllExhausted; }
    StopReason::Continue
}
```

---

## 18. Settings and User Controls

### Settings Categories

**API Keys:**
| Key | Type | Description |
|-----|------|-------------|
| `brave_api_key` | string (secret) | Brave Search API key |
| `serper_api_key` | string (secret) | Serper API key |
| `openrouter_api_key` | string (secret) | OpenRouter API key |
| `ollama_base_url` | string | Ollama server URL (default: http://localhost:11434) |

**LLM Configuration:**
| Key | Type | Description |
|-----|------|-------------|
| `llm_provider` | enum | `openrouter` \| `ollama` |
| `default_model` | string | Default model for all stages (default: openai/gpt-5.4-mini) |
| `model_query_interpreter` | string? | Override model for QueryInterpreter |
| `model_schema_planner` | string? | Override model for SchemaPlanner |
| `model_search_planner` | string? | Override model for SearchPlanner |
| `model_query_expander` | string? | Override model for QueryExpander |
| `model_extractor` | string? | Override model for Extractor |
| `model_validator` | string? | Override model for Validator |

**Search Configuration:**
| Key | Type | Description |
|-----|------|-------------|
| `primary_search_provider` | enum | `brave` \| `serper` |
| `search_fallback_enabled` | bool | Enable fallback to other provider on error |
| `max_results_per_query` | integer | Max results per search query (default: 20) |

**Execution Configuration:**
| Key | Type | Description |
|-----|------|-------------|
| `max_parallel_fetches` | integer | Concurrent fetch workers (default: 8) |
| `max_parallel_extractions` | integer | Concurrent LLM extraction workers (default: 3) |
| `fetch_timeout_seconds` | integer | Timeout per page fetch (default: 15) |
| `rate_limit_per_domain_ms` | integer | Min delay between requests to same domain (default: 2000) |
| `respect_robots_txt` | bool | Check robots.txt before fetching (default: true) |
| `max_page_size_kb` | integer | Skip pages larger than this (default: 5000) |

**Quality Configuration:**
| Key | Type | Description |
|-----|------|-------------|
| `precision_recall` | enum | `precision` \| `balanced` \| `recall` |
| `evidence_strictness` | enum | `strict` \| `moderate` \| `lenient` |
| `min_confidence_threshold` | float | Min confidence to include row (default: 0.5) |
| `enable_semantic_validation` | bool | Use LLM for semantic validation (default: true) |
| `dedup_similarity_threshold` | float | Auto-merge threshold (default: 0.85) |

**Stop Conditions:**
| Key | Type | Description |
|-----|------|-------------|
| `target_row_count` | integer | Target number of result rows (default: 50) |
| `max_budget_usd` | float | Max estimated LLM cost (default: 1.00) |
| `max_duration_seconds` | integer | Max run duration (default: 600) |
| `saturation_threshold` | float | New row ratio below which search is saturated (default: 0.05) |

**UI Configuration:**
| Key | Type | Description |
|-----|------|-------------|
| `theme` | enum | `dark` \| `light` \| `system` |
| `default_export_format` | enum | `csv` \| `json` \| `xlsx` |

---

## 19. Desktop UX Flow

### Main Screens

```
┌─────────────────────────────────────────────────────┐
│ ┌─── Sidebar ───┐  ┌─── Main Content ────────────┐ │
│ │               │  │                              │ │
│ │  🔍 Query     │  │  [Query Input Screen]        │ │
│ │  📋 History   │  │  OR                          │ │
│ │  ⚙️ Settings  │  │  [Results Screen]            │ │
│ │               │  │  OR                          │ │
│ │               │  │  [History Screen]             │ │
│ │               │  │  OR                          │ │
│ │               │  │  [Settings Screen]            │ │
│ │               │  │                              │ │
│ └───────────────┘  └──────────────────────────────┘ │
│ ┌─── Log Panel (collapsible) ─────────────────────┐ │
│ │ [INFO] QueryInterpreter: parsed entity_type=... │ │
│ │ [INFO] SearchExecutor: 20 results from Brave    │ │
│ │ [WARN] Fetcher: timeout on https://...          │ │
│ └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

### User Flow

```
1. QUERY SCREEN
   ┌──────────────────────────────────────┐
   │ Enter your research query:           │
   │ ┌──────────────────────────────────┐ │
   │ │ Find all AI security conferences │ │
   │ │ in Europe 2026                   │ │
   │ └──────────────────────────────────┘ │
   │                                      │
   │  [Advanced Options ▼]               │
   │   Target rows: [50]                 │
   │   Max budget: [$1.00]               │
   │   Time limit: [10 min]              │
   │   Languages: [en, de, fr]           │
   │                                      │
   │  [ 🚀 Run Query ]                   │
   └──────────────────────────────────────┘
                    │
                    ▼
2. SCHEMA REVIEW (modal/overlay)
   ┌──────────────────────────────────────┐
   │ Proposed Schema:                     │
   │                                      │
   │ ☑ name       (text)    *required     │
   │ ☑ date       (date)    *required     │
   │ ☑ location   (text)                  │
   │ ☑ website    (url)                   │
   │ ☑ topics     (list)                  │
   │ ☑ organizer  (text)                  │
   │                                      │
   │ [+ Add Column]  [✓ Confirm Schema]  │
   └──────────────────────────────────────┘
                    │
                    ▼
3. RESULTS SCREEN (streaming)
   ┌──────────────────────────────────────────────────┐
   │ "AI security conferences in Europe 2026"         │
   │ Status: Running ⏸️ ⏹️  |  32/50 rows  |  2:15  │
   │ Searches: 12 | Pages: 45 | Cost: $0.23          │
   │                                                   │
   │ ┌─────────┬────────┬──────────┬─────────┬──────┐│
   │ │ Name    │ Date   │ Location │ Website │ ...  ││
   │ ├─────────┼────────┼──────────┼─────────┼──────┤│
   │ │ BlackHat│ Jul 26 │ London   │ bh.com  │      ││
   │ │ SecConf │ Sep 26 │ Berlin   │ sec.io  │      ││
   │ │ ...streaming...                              ││
   │ └─────────────────────────────────────────────┘│
   │                                                   │
   │ [📥 Export ▼]  (CSV | JSON | XLSX)              │
   └──────────────────────────────────────────────────┘
                    │
                    ▼ (click row)
4. ROW DETAIL PANEL
   ┌──────────────────────────────────────┐
   │ BlackHat Europe 2026                 │
   │                                      │
   │ Sources (3):                         │
   │ 🔗 blackhat.com/eu-26 — "BlackHat   │
   │    Europe returns to London..."      │
   │ 🔗 infosec.org/events — "Upcoming   │
   │    security conferences..."          │
   │ 🔗 techcrunch.com/... — "AI security│
   │    events to watch in 2026..."       │
   │                                      │
   │ Confidence: 0.92                     │
   └──────────────────────────────────────┘
```

---

## 20. Error Handling and Recovery

### Error Categories

| Category | Examples | Strategy |
|----------|----------|----------|
| **API errors** | 401 (bad key), 429 (rate limit), 500 (server) | Retry with backoff; 401 → stop + notify user; 429 → wait + retry; 500 → retry 3x then skip |
| **Network errors** | Timeout, DNS failure, connection refused | Retry 2x with backoff; then skip URL; log warning |
| **LLM errors** | Invalid JSON response, empty response, gibberish | Retry 1x; if still invalid → use fallback (skip/default) |
| **Parse errors** | Malformed HTML, encoding issues | Best-effort clean; if unparseable → skip page |
| **Storage errors** | SQLite write failure, disk full | Pause run; notify user; retry on resume |
| **Budget errors** | Estimated cost exceeds limit | Stop run gracefully; keep results collected so far |

### Retry Policy

```rust
struct RetryPolicy {
    max_retries: u32,      // Default: 3 for API, 2 for fetch, 1 for LLM
    base_delay_ms: u64,    // Default: 1000
    max_delay_ms: u64,     // Default: 30000
    backoff_factor: f64,   // Default: 2.0
}

// Delay = min(base_delay * backoff_factor^attempt, max_delay)
```

### Recovery on Restart

1. On app start, query `runs WHERE status IN ('running', 'paused')`
2. For each interrupted run:
   - Load run config and schema
   - Find last completed search_query batch
   - Find URLs with `status = 'pending'` (not yet fetched)
   - Resume from there
3. User sees notification: "1 interrupted run found. Resume?"

---

## 21. Observability and Debugging

### Logging

- **Library:** `tracing` + `tracing-subscriber`
- **Log levels:** DEBUG, INFO, WARN, ERROR
- **File output:** `{app_data_dir}/query2table/logs/query2table_{date}.log`
- **Rotation:** Daily log rotation, keep 30 days
- **GUI output:** Streamed via Tauri events to LogViewer component
- **Structured fields:** run_id, role, url, query_text, elapsed_ms, token_count

### Key Log Events

| Event | Level | Data |
|-------|-------|------|
| Run started | INFO | run_id, query, config |
| Query interpreted | INFO | run_id, entity_type, attributes |
| Schema proposed | INFO | run_id, columns |
| Search query executed | INFO | run_id, query_text, provider, result_count |
| Page fetched | DEBUG | run_id, url, status, duration_ms |
| Entity extracted | DEBUG | run_id, url, entity_count, quality |
| Entity validated | DEBUG | run_id, entity_id, valid, confidence |
| Dedup match found | INFO | run_id, entity_a, entity_b, similarity |
| Stop condition met | INFO | run_id, reason, stats |
| Run completed | INFO | run_id, total_rows, total_cost, duration |
| Error occurred | ERROR | run_id, role, error_message, context |
| Retry attempted | WARN | run_id, role, attempt, max_retries |

### Performance Metrics (tracked in run stats)

- Total elapsed time
- Time per stage (search, fetch, extract, validate, dedup)
- Pages per second (fetch throughput)
- Rows per page (extraction yield)
- LLM tokens used + estimated cost
- Dedup ratio (rows before / rows after dedup)

---

## 22. Security / Compliance / Robots Considerations

### API Key Security
- Keys stored in SQLite settings table (local file, not encrypted at rest — acceptable for desktop app)
- Keys never logged (redacted in log output)
- Keys transmitted only to their respective API endpoints over HTTPS
- No telemetry or analytics that includes keys
- Tauri CSP configured to allow only required API domains

### Robots.txt Compliance
- Before fetching any URL, check robots.txt for the domain
- Cache robots.txt per domain (TTL: 24 hours)
- Default User-Agent: `Query2TableBot/1.0 (+https://github.com/user/query2table)`
- If robots.txt disallows, skip URL and log

### Rate Limiting
- Per-domain: minimum 2000ms between requests to same domain
- Global: maximum 20 requests/second across all domains
- API providers: honor their rate limits; back off on 429

### Content Policy
- Do not store raw HTML (only cleaned text)
- Do not bypass paywalls or authentication
- Do not scrape personally identifiable information unless explicitly requested by query
- Respect `noindex`/`nofollow` meta tags as best-effort

### Tauri Security
- CSP (Content Security Policy) in tauri.conf.json
- Scoped filesystem access (only app data directory)
- No arbitrary shell command execution from frontend

---

## 23. Performance and Cost Risks

| Risk | Impact | Mitigation |
|------|--------|-----------|
| **LLM cost spikes** | $5-20+ for large runs | Budget limit (default $1), token tracking, cheaper models for extraction |
| **Slow extraction** | Minutes per page if LLM is slow | Parallel extraction (3 workers), skip low-relevance pages early |
| **Search API costs** | Brave free tier exhausted quickly | Track API calls, warn user, support multiple providers |
| **Rate limiting by sites** | Blocked/throttled fetches | Per-domain rate limiting, User-Agent rotation, respect robots.txt |
| **Large result sets** | UI becomes sluggish with 1000+ rows | TanStack Table virtualization, paginated rendering |
| **SQLite write contention** | Slow writes during parallel pipeline | WAL mode, batch inserts, connection pool |
| **Memory usage** | Storing many page texts in memory | Stream processing; don't hold all pages in memory; write to DB immediately |
| **Headless browser size** (Phase 2) | Large binary size (~200MB Chrome) | Phase 2; optional download; fallback to HTTP-only |
| **OpenRouter downtime** | Pipeline stalls | Ollama fallback; resume capability |
| **Poor LLM extraction quality** | Low-quality/hallucinated rows | Validation layer, confidence scoring, user can inspect sources |

---

## 24. MVP Scope

### Included in MVP

- [x] Tauri + Svelte desktop app shell
- [x] Skeleton UI with dark/light theme
- [x] Settings page (API keys, models, providers, quality, stop conditions)
- [x] Full pipeline: Query → Schema → Search → Fetch → Extract → Validate → Dedup → Table
- [x] QueryInterpreter, SchemaPlanner, SearchPlanner, QueryExpander (LLM roles)
- [x] SearchExecutor, Fetcher, DocumentParser (deterministic roles)
- [x] Extractor, Validator, Deduplicator (hybrid roles)
- [x] StoppingController with all 6 stop conditions
- [x] Schema confirmation/edit UI
- [x] Streaming results table (TanStack Table)
- [x] Row detail panel with sources
- [x] Run progress indicator + pause/resume/cancel
- [x] OpenRouter LLM integration
- [x] Ollama LLM integration
- [x] Brave Search integration
- [x] Serper integration
- [x] HTTP fetcher with per-domain rate limiting
- [x] robots.txt compliance
- [x] HTML cleaning / boilerplate removal
- [x] Multilingual query expansion
- [x] Fuzzy string deduplication
- [x] Resume interrupted runs
- [x] Run history
- [x] Export (CSV, JSON, XLSX)
- [x] System tray + completion notifications
- [x] Structured logging (file + GUI log viewer)
- [x] SQLite local persistence (WAL mode)
- [x] CI/CD (GitHub Actions, 3 platforms)
- [x] Auto-updater (Tauri updater)
- [x] Full settings for precision/recall/strictness/budgets

### Excluded from MVP (Phase 2+)

- [ ] JS-rendered page support (headless browser)
- [ ] PDF document parsing
- [ ] Templates/presets
- [ ] Advanced LLM-assisted deduplication (ambiguous cases)
- [ ] Plugin system for custom extractors
- [ ] Manual cell editing in results table
- [ ] Intermediate row deletion during search
- [ ] i18n/localization of the app UI

---

## 25. Phase 2 / Phase 3 Extensions

### Phase 2 — Enhanced Content Access

| Feature | Description | Complexity |
|---------|-------------|-----------|
| JS-rendered pages | chromiumoxide for SPA/dynamic sites. Detect JS-heavy pages by checking if initial HTML has minimal content. | High |
| PDF parsing | pdf-extract for PDF text extraction. Detect PDF by Content-Type header or URL suffix. | Medium |
| Templates/presets | Pre-built query templates ("Find companies in X doing Y", "Find conferences about X"). Stored in SQLite. | Medium |
| Advanced LLM dedup | For ambiguous entity pairs (similarity 0.7-0.9), ask LLM to decide. Per-stage model selection for dedup. | Medium |

### Phase 3 — Power Features

| Feature | Description | Complexity |
|---------|-------------|-----------|
| Plugin system | WASM-based custom extractors/validators. User uploads .wasm plugin. | High |
| Scheduled runs | Re-run queries on schedule to track changes. | Medium |
| Comparison mode | Diff two runs of same query to see changes. | Medium |
| Collaborative export | Google Sheets / Notion integration. | Medium |
| Custom search providers | Plugin interface for additional search APIs (DuckDuckGo, Bing). | Medium |
| App UI localization | i18n for app interface. | Low |

---

## 26. Recommended Folder / Module Structure

```
query2table/
├── src-tauri/                          # Rust backend (Tauri)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/                   # Tauri v2 capabilities
│   ├── icons/
│   ├── migrations/                     # SQLite migrations (sqlx)
│   │   └── 001_initial.sql
│   └── src/
│       ├── main.rs                     # Tauri entry point
│       ├── lib.rs                      # Module declarations
│       ├── commands/                   # Tauri IPC command handlers
│       │   ├── mod.rs
│       │   ├── run.rs                  # start_run, pause_run, cancel_run, resume_run
│       │   ├── settings.rs            # get_settings, update_settings
│       │   ├── history.rs             # list_runs, get_run, delete_run
│       │   └── export.rs              # export_csv, export_json, export_xlsx
│       ├── orchestrator/              # Pipeline orchestration
│       │   ├── mod.rs
│       │   ├── pipeline.rs            # Main pipeline state machine
│       │   ├── state.rs               # RunState, PipelineState enums
│       │   ├── stop_controller.rs     # Stop condition evaluation
│       │   └── budget_tracker.rs      # LLM cost tracking
│       ├── roles/                     # Fixed pipeline roles
│       │   ├── mod.rs
│       │   ├── query_interpreter.rs   # NL query → structured intent
│       │   ├── schema_planner.rs      # Intent → table schema
│       │   ├── search_planner.rs      # Intent + schema → search queries
│       │   ├── query_expander.rs      # Translate/expand queries
│       │   ├── search_executor.rs     # Execute search API calls
│       │   ├── fetcher.rs             # HTTP page fetcher
│       │   ├── document_parser.rs     # HTML → clean text
│       │   ├── extractor.rs           # Text + schema → entity rows
│       │   ├── validator.rs           # Validate extracted rows
│       │   ├── deduplicator.rs        # Fuzzy dedup
│       │   └── ui_event_publisher.rs  # Emit Tauri events
│       ├── providers/                 # External API integrations
│       │   ├── mod.rs
│       │   ├── llm/
│       │   │   ├── mod.rs
│       │   │   ├── traits.rs          # LlmProvider trait
│       │   │   ├── openrouter.rs      # OpenRouter client
│       │   │   ├── ollama.rs          # Ollama client
│       │   │   └── manager.rs         # Provider selection + fallback
│       │   ├── search/
│       │   │   ├── mod.rs
│       │   │   ├── traits.rs          # SearchProvider trait
│       │   │   ├── brave.rs           # Brave Search client
│       │   │   ├── serper.rs          # Serper client
│       │   │   └── manager.rs         # Provider selection + fallback
│       │   └── http/
│       │       ├── mod.rs
│       │       ├── client.rs          # reqwest client wrapper
│       │       ├── rate_limiter.rs    # Per-domain rate limiter
│       │       └── robots.rs          # robots.txt fetcher + parser
│       ├── storage/                   # SQLite persistence
│       │   ├── mod.rs
│       │   ├── db.rs                  # Connection pool + init
│       │   ├── models.rs             # DB model structs
│       │   └── repository.rs         # CRUD operations
│       ├── export/                    # Export implementations
│       │   ├── mod.rs
│       │   ├── csv.rs
│       │   ├── json.rs
│       │   └── xlsx.rs
│       └── utils/
│           ├── mod.rs
│           ├── logging.rs            # tracing setup
│           └── id.rs                 # UUID generation
│
├── src/                               # Svelte frontend
│   ├── app.html
│   ├── app.css                        # Global styles + theme variables
│   ├── lib/
│   │   ├── components/
│   │   │   ├── layout/
│   │   │   │   ├── Sidebar.svelte
│   │   │   │   ├── LogPanel.svelte
│   │   │   │   └── AppShell.svelte
│   │   │   ├── query/
│   │   │   │   ├── QueryInput.svelte
│   │   │   │   ├── AdvancedOptions.svelte
│   │   │   │   └── SchemaEditor.svelte
│   │   │   ├── results/
│   │   │   │   ├── ResultsTable.svelte
│   │   │   │   ├── RowDetail.svelte
│   │   │   │   └── RunProgress.svelte
│   │   │   ├── history/
│   │   │   │   └── HistoryList.svelte
│   │   │   ├── settings/
│   │   │   │   ├── ApiKeysSection.svelte
│   │   │   │   ├── ModelsSection.svelte
│   │   │   │   ├── SearchSection.svelte
│   │   │   │   ├── ExecutionSection.svelte
│   │   │   │   ├── QualitySection.svelte
│   │   │   │   └── StopConditionsSection.svelte
│   │   │   └── common/
│   │   │       ├── ExportDialog.svelte
│   │   │       └── NotificationToast.svelte
│   │   ├── stores/
│   │   │   ├── runs.ts               # Current run state
│   │   │   ├── settings.ts           # Settings store (synced with backend)
│   │   │   ├── history.ts            # Run history
│   │   │   └── ui.ts                 # UI state (theme, sidebar, log panel)
│   │   ├── types/
│   │   │   └── index.ts              # TypeScript types mirroring Rust models
│   │   └── api/
│   │       └── tauri.ts              # Tauri invoke/listen wrappers
│   ├── routes/
│   │   ├── +layout.svelte            # AppShell wrapper
│   │   ├── +page.svelte              # Query page (default)
│   │   ├── history/
│   │   │   └── +page.svelte
│   │   ├── history/[id]/
│   │   │   └── +page.svelte          # View past run results
│   │   └── settings/
│   │       └── +page.svelte
│   └── global.d.ts
│
├── .ai-factory/
│   ├── DESCRIPTION.md
│   └── ARCHITECTURE.md
├── .github/
│   └── workflows/
│       └── release.yml               # Multi-platform build + release
├── AGENTS.md
├── TASKS.md
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
└── README.md
```

---

## 27. Testing Strategy

### Unit Tests (Rust)

| Module | Test Focus | Approach |
|--------|-----------|----------|
| `providers/llm/` | API request/response serialization | Mock HTTP responses |
| `providers/search/` | Query construction, response parsing | Mock HTTP responses |
| `providers/http/rate_limiter` | Rate limiting correctness | Time-based assertions |
| `providers/http/robots` | robots.txt parsing | Known robots.txt fixtures |
| `roles/document_parser` | HTML cleaning | HTML fixture files → expected text |
| `roles/deduplicator` | Similarity scoring, merge logic | Entity pairs with known similarity |
| `roles/validator` | Schema validation rules | Valid/invalid entity fixtures |
| `orchestrator/stop_controller` | Stop condition evaluation | RunState fixtures |
| `storage/repository` | CRUD operations | In-memory SQLite |
| `export/*` | Output format correctness | Compare with expected files |

### Integration Tests (Rust)

| Test | Scope | Approach |
|------|-------|----------|
| Full pipeline (mocked) | Orchestrator → all roles with mock providers | Mock LLM returns fixture JSON; mock search returns fixture results |
| Resume flow | Interrupt + restart | Create partial run in DB; verify resume picks up correctly |
| Budget enforcement | Stop on budget | Mock LLM with known token counts; verify stop at budget |
| Dedup pipeline | Extract → Dedup | Feed known duplicate entities; verify correct merging |

### E2E Tests (Optional, API-key-gated)

| Test | Scope | Approach |
|------|-------|----------|
| Real search + extract | Full pipeline with real APIs | Simple query ("find top 5 programming languages"); verify table has rows |
| Provider fallback | Primary fails → fallback works | Intentionally invalid primary key; valid fallback key |

### Frontend Tests

| Test | Scope | Approach |
|------|-------|----------|
| Component rendering | All Svelte components | Vitest + @testing-library/svelte |
| Settings persistence | Save → reload → verify | Mock Tauri invoke |
| Event handling | Streaming updates | Mock Tauri event listeners |

### Test Commands

```bash
# Rust unit + integration tests
cd src-tauri && cargo test

# Frontend tests
npm run test

# E2E (requires API keys in env)
cd src-tauri && cargo test --features e2e -- --ignored
```

---

## 28. Open Questions

| # | Question | Current Assumption | Impact if Wrong |
|---|----------|-------------------|-----------------|
| OQ1 | Optimal chunking size for extraction LLM? | 3000 tokens per chunk | May miss entities at chunk boundaries |
| OQ2 | Should we cache search results across runs? | No — each run is independent | Missed optimization for repeated similar queries |
| OQ3 | How to handle multi-page entities (e.g., company info spread across About + Products pages)? | MVP: treat each page independently | Lower recall for entities needing multiple sources |
| OQ4 | Should Validator LLM call be optional/skippable for speed? | It is optional (controlled by setting) | Users may get lower quality without it |
| OQ5 | What to do when primary and fallback search providers both fail? | Stop run with error | Run fails completely even if partial results exist |
| OQ6 | Optimal parallel extraction count vs cost? | 3 workers | Too few = slow; too many = cost spike |
| OQ7 | Should we support OpenAI API directly (not via OpenRouter)? | No — OpenRouter covers all models | Some users may prefer direct API |
| OQ8 | How to handle sites that return soft 200 with "access denied" body? | Best-effort detection via content analysis | May waste LLM tokens on useless pages |
| OQ9 | Skeleton UI vs alternative Svelte component library? | Skeleton UI | May need to evaluate alternatives if bundle size is concern |
| OQ10 | Tauri v2 plugin ecosystem maturity for auto-updater? | Mature enough for production | May need workarounds |

---

## 29. Final Recommended Build Order

### Stage 1: Walking Skeleton (Sprint 1, ~2 weeks)
**Goal:** App launches, navigates between pages, stores settings, shows logs.

1. `T001` — Initialize Tauri v2 + SvelteKit project
2. `T003` — Set up Rust module structure
3. `T002` — Configure Skeleton UI + dark/light theme
4. `T004` — Set up SQLite + sqlx + initial migration
5. `T005` — Settings storage (Rust)
6. `T007` — Logging infrastructure (tracing)
7. `T044` — App layout + sidebar navigation
8. `T006` — Settings UI (all sections)
9. `T008` — Log viewer panel

**Deliverable:** App that opens, has 3 pages (Query/History/Settings), persists settings, shows logs.

### Stage 2: Provider Integration (Sprint 2, ~1.5 weeks)
**Goal:** Can call LLM and search APIs from the app.

10. `T009` — LlmProvider trait
11. `T010` — OpenRouter client
12. `T011` — Ollama client
13. `T012` — LLM provider manager
14. `T013` — SearchProvider trait
15. `T014` — Brave Search client
16. `T015` — Serper client
17. `T016` — Search provider manager
18. `T017` — HTTP fetcher with rate limiter
19. `T018` — robots.txt checker

**Deliverable:** Backend can call any configured LLM, search provider, and fetch pages.

### Stage 3: Pipeline Roles (Sprint 3, ~2 weeks)
**Goal:** All individual roles work in isolation with unit tests.

20. `T019` — HTML cleaner / DocumentParser
21. `T020` — QueryInterpreter
22. `T021` — SchemaPlanner
23. `T023` — SearchPlanner
24. `T024` — QueryExpander
25. `T025` — SearchExecutor
26. `T026` — Extractor
27. `T027` — Validator
28. `T028` — Deduplicator
29. `T029` — StoppingController

**Deliverable:** Each role can be tested individually with mock inputs.

### Stage 4: Orchestrator (Sprint 4, ~1.5 weeks)
**Goal:** Full pipeline runs end-to-end in the backend.

30. `T030` — Pipeline state machine
31. `T033` — Parallel fetch worker pool
32. `T034` — Parallel extraction pool
33. `T035` — Budget tracker
34. `T031` — Run state persistence (resume)
35. `T032` — UIEventPublisher

**Deliverable:** Can run a full query from CLI/tests; results stored in SQLite; events emitted.

### Stage 5: Frontend Integration (Sprint 5, ~1.5 weeks)
**Goal:** Full UI for the complete pipeline.

36. `T036` — Query input UI
37. `T022` — Schema confirmation UI
38. `T037` — Results table (TanStack Table, streaming)
39. `T038` — Row detail panel (sources)
40. `T039` — Run progress indicator
41. `T040` — Run controls (pause/resume/cancel)

**Deliverable:** User can enter query, confirm schema, watch results stream in, see sources.

### Stage 6: Polish & Ship (Sprint 6, ~1.5 weeks)
**Goal:** Export, history, notifications, error handling, CI/CD.

42. `T045` — CSV export
43. `T046` — JSON export
44. `T047` — XLSX export
45. `T042` — Export dialog
46. `T041` — History page
47. `T043` — System tray + notifications
48. `T048` — Error handling + retry logic
49. `T049` — Integration tests
50. `T050` — CI/CD setup
51. `T051` — Auto-updater

**Deliverable:** Releasable MVP. Full pipeline, export, history, notifications, CI/CD.

### Stage 7: Hardening (Sprint 7, ~1 week)
**Goal:** Quality assurance and edge case handling.

52. `T052` — E2E tests
53. Bug fixes from testing
54. Performance profiling and optimization
55. Documentation (README)

**Deliverable:** Production-ready v1.0.

---

## Appendix A: Cargo.toml Dependencies (Estimated)

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-notification = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
tauri-plugin-updater = "2"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "cookies"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
scraper = "0.20"
ammonia = "4"
strsim = "0.11"
uuid = { version = "1", features = ["v4"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-appender = "0.2"
chrono = { version = "0.4", features = ["serde"] }
url = "2"
rust_xlsxwriter = "0.79"
csv = "1"
tokio-util = "0.7"
unicode-normalization = "0.1"
thiserror = "2"
anyhow = "1"
```

## Appendix B: npm Dependencies (Estimated)

```json
{
  "devDependencies": {
    "@sveltejs/kit": "^2",
    "@sveltejs/adapter-static": "^3",
    "svelte": "^5",
    "vite": "^6",
    "@tauri-apps/cli": "^2",
    "typescript": "^5",
    "vitest": "^2",
    "@testing-library/svelte": "^5"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-notification": "^2",
    "@tauri-apps/plugin-fs": "^2",
    "@skeletonlabs/skeleton": "^2",
    "@skeletonlabs/tw-plugin": "^0.4",
    "@tanstack/svelte-table": "^8"
  }
}
```

---

## E2E Pipeline Tests Plan

### Goal
Comprehensive e2e/integration tests covering the full pipeline (search → fetch → extract → dedup), with mock LLM/Search providers and a log capture system that saves test logs to files for later analysis. Tests go in `src-tauri/tests/` as Rust integration tests.

### Phase 1: Test Infrastructure Setup

- [x] **1.1** Add `wiremock = "0.6"` to `src-tauri/Cargo.toml` dev-deps; add `e2e` feature flag
- [x] **1.2** Add `pub fn with_provider(provider, config)` to `LlmManager` (`src-tauri/src/providers/llm/manager.rs`)
- [x] **1.3** Add `pub fn with_providers(primary, fallback, config)` to `SearchManager` (`src-tauri/src/providers/search/manager.rs`)
- [x] **1.4** Create test helpers module (`src-tauri/tests/common/mod.rs`):
  - `setup_test_db()` — in-memory SQLite with full migration
  - `TestLogCapture` — captures tracing events to per-test log files in `src-tauri/tests/logs/`
  - `MockLlmProvider` — content-based routing: detects role from system prompt, returns fixture JSON
  - `MockSearchProvider` — returns configurable fixture search results
  - `test_pipeline_config()` — returns PipelineConfig with sensible test defaults
  - `test_fetcher()` — returns HttpFetcher with no_proxy for local wiremock servers
- [x] **1.5** Create JSON fixture files in `src-tauri/tests/fixtures/`:
  - `interpret_response.json`, `schema_response.json`, `search_plan_response.json`
  - `expand_response.json`, `extract_response.json`, `extract_response_duplicates.json`

### Phase 2: Integration Tests (Mock Providers)

All tests use mock LLM/Search providers. No real API calls.

- [x] **2.1** Full pipeline happy path — mock LLM returns scripted responses, mock Search returns 3 URLs to wiremock, verify: run status=Completed, entity_rows > 0, schema confirmed
- [x] **2.2** Pipeline cancellation — send Cancel during running, verify: status=Cancelled or Completed
- [x] **2.3** Pipeline pause/resume — Pause during Running, Resume after 200ms, verify: completes normally
- [x] **2.4** Schema auto-confirmation — no events (auto-confirm), verify: DB schema confirmed with correct columns
- [x] **2.5** Budget stop condition — set max_budget_usd very low, verify: stops early
- [x] **2.6** Duration stop condition — set max_duration_seconds=1, verify: stops early  
- [x] **2.7** Row count stop condition — set target_row_count=1, verify: stops after reaching target
- [x] **2.8** LLM failure handling — mock LLM returns error for interpreter, verify: pipeline returns Err
- [x] **2.9** Search failure handling — mock Search returns error, verify: pipeline completes with 0 rows (graceful degradation)
- [x] **2.10** Empty search results — mock Search returns 0 results, verify: completes with 0 rows
- [x] **2.11** Extraction failure — mock LLM returns invalid JSON for extraction, verify: continues, 0 rows
- [x] **2.12** Deduplication — mock extractor returns same entities from 5 pages, verify: dedup stats recorded

### Phase 3: Log Capture System

- [x] **3.1** Implement `setup_test_logs()` using `tracing_subscriber` file writer layer
- [x] **3.2** Each test creates log file: `tests/logs/{test_name}_{timestamp}.log`
- [x] **3.3** DEBUG-level capture with target and level info
- [x] **3.4** Add `tests/logs/*.log` to `.gitignore`

### Phase 4: Real API E2E Tests (feature-gated)

- [ ] **4.1** Real search+extract test behind `#[cfg(feature = "e2e")]` + `#[ignore]`
- [ ] **4.2** Provider fallback test with invalid primary + valid secondary key

### Verification

1. `cd src-tauri && cargo test --test pipeline_integration -- --test-threads=1` — all 12 integration tests pass ✅
2. `cargo test --lib` — existing 144 unit tests still pass (regression) ✅
3. Log files created in `tests/logs/` with full pipeline traces ✅
