# AGENTS.md

> Project map for AI agents. Keep this file up-to-date as the project evolves.

Все задачи по проекту читать и помечать сделанное в TASKS.md.
Задачи можно считать выполненными, когда код для них написан и протестирован, а не просто когда они помечены как "готово" в таск-трекере.

## Project Overview
Query2Table is a local-first desktop application (Tauri v2 + Rust + Svelte) that converts natural-language research queries into structured tables with row-level sources. Uses an orchestrator with fixed roles to search the internet, fetch/parse pages, extract entities via LLMs, and deduplicate results.

## Tech Stack
- **Desktop Shell:** Tauri v2
- **Backend:** Rust (Tokio async runtime)
- **Frontend:** Svelte 5 (SvelteKit SPA)
- **Database:** SQLite (sqlx, WAL mode)
- **LLM:** OpenRouter (OpenAI-compatible) + Ollama (local)
- **Search:** Brave Search API + Serper API
- **Styling:** Plain CSS with design tokens in `src/app.css` (`--app-*`), shared `.button`/`.input` classes and `Badge`/`EmptyState` components

## Project Structure
```
query2table/
├── src-tauri/                  # Rust backend
│   ├── Cargo.toml              # Rust dependencies
│   ├── tauri.conf.json         # Tauri configuration
│   ├── migrations/             # SQLite schema migrations
│   └── src/
│       ├── main.rs             # Tauri entry point
│       ├── lib.rs              # Module declarations
│       ├── attachments/        # Attached files: parsing (parse/), fragments, storage, FTS search
│       ├── commands/           # Tauri IPC command handlers (run, settings, history, export, attachments)
│       ├── orchestrator/       # Pipeline state machines (table, images, links, research), stop controller, budget tracker
│       ├── roles/              # Fixed pipeline roles (interpreter, planner, executor, extractor, link_ranker, etc.)
│       ├── providers/          # External API clients (llm/, search/, http/)
│       ├── storage/            # SQLite models, repository, migrations
│       ├── export/             # CSV, JSON, XLSX export implementations
│       └── utils/              # Logging, ID generation
├── src/                        # Svelte frontend
│   ├── app.html                # HTML template
│   ├── app.css                 # Global styles + theme variables
│   ├── lib/
│   │   ├── components/         # UI components (layout, query, results, settings, common)
│   │   ├── stores/             # Svelte stores (runs, settings, history, ui)
│   │   ├── i18n/               # Interface languages: en.ts (reference), ru.ts, t() and language choice
│   │   ├── types/              # TypeScript type definitions
│   │   └── api/                # Tauri invoke/listen wrappers
│   └── routes/                 # SvelteKit pages (query, history, settings)
├── .ai-factory/
│   ├── DESCRIPTION.md          # Project specification and tech stack
│   └── ARCHITECTURE.md         # Architecture decisions and guidelines
├── AGENTS.md                   # This file — project structure map
├── TASKS.md                    # Full technical implementation plan
├── package.json                # Node.js/frontend dependencies
├── svelte.config.js            # SvelteKit configuration
├── vite.config.ts              # Vite build configuration
└── tsconfig.json               # TypeScript configuration
```

## Interface language
All user-visible frontend text goes through `t('key', params)` from `$lib/i18n`; never hard-code UI strings.
Add every new key to both `src/lib/i18n/en.ts` and `ru.ts` (keys are sorted; plural messages use `{ one, few, many, other }` with a numeric `count` param). `src/tests/i18n.test.ts` checks that both catalogs have the same keys and placeholders.
The language is the `ui_language` setting (`system`, `en`, `ru`); the backend uses it for notifications and the tray menu (`src-tauri/src/utils/i18n.rs`). Technical details (raw provider errors, logs) and model answers are not translated.

## Themes
Colors come only from the `--app-*` tokens. Themes live in `src/lib/themes.ts` and as `:root[data-theme='<id>']` blocks in `src/app.css`; `src/tests/themes.test.ts` checks that both match and that text keeps 4.5:1 contrast. Dark themes also get the `.dark` class. Font sizes must use the `--app-text-*` tokens so the interface size (`data-scale`) applies.

## Attachments
Files attached to a query live in `src-tauri/src/attachments/` (plan: TASKS.md › Attachments). A place in a file is an `attachment://<id>?page=N` / `?sheet=S&rows=A-B` / `?section=H` URL (`Locator`), so file sources use the same `url` columns as web pages. Files are stored once by SHA-256 under `<data>/attachments`, fragments in `attachment_chunks` with the FTS5 index `attachment_chunks_fts`; `run_attachments` links them to runs and conversation turns. The query form uses `AttachmentBar` and the draft store `src/lib/stores/attachments.ts`.
Images go to models as `Message.images` (`ImageInput`; OpenAI-compatible sends `image_url` parts, Ollama `images`). `providers/llm/capabilities.rs` detects vision from catalogs and resolves `VisionPlan` from `llm_vision` / `vision_model`; `orchestrator/files.rs` (`RunFiles`) gives pipelines the files of a run: labelled `F1…`, an overview plus matching fragments for the first message, inline images for models that see them, and `read` for the research agent's `read_file` action (step type `read`, cited by `attachment://` URL). `PipelineConfig.web_search = false` is the files-only mode (Research and Table). In tables, the interpreter and planners get `QueryIntent.files_context`, and every file fragment is extracted like a page (row sources keep the `attachment://` address, without a fetched page). Links and Images use files as context only (`files::load_for_run`); in Images an attached picture is a reference: `ImageRanker::rank_with_reference` re-scores the top text-ranked candidates with the model for images. `attachments::vision::prepare` reads scanned pages (rendered with `hayro`, needs Rust ≥ 1.92) and describes pictures once, storing the text as fragments.

## UI controls
Do not use the browser's own controls. Use `Select` (lists; search appears from 8 options), `Checkbox`, `Radio`, `NumberInput` and `Slider` from `src/lib/components/common/`, and `use:tooltip` from `src/lib/actions/tooltip.ts` instead of `title` (`{ text, whenTruncated: true }` for clipped text). `<details>`, scrollbars and number spinners are styled globally in `src/app.css`. Right-click on text fields and selected text opens the app's menu (`src/lib/utils/textMenu.ts`). In Playwright tests pick list options with `choose()` from `tests/ui/fixtures.ts`.

## Key Entry Points
| File | Purpose |
|------|---------|
| src-tauri/src/main.rs | Tauri app entry point, plugin registration |
| src-tauri/src/lib.rs | Rust module tree declaration |
| src-tauri/src/commands/run.rs | IPC handlers for starting/pausing/cancelling runs |
| src-tauri/src/orchestrator/pipeline.rs | Main pipeline state machine |
| src/routes/+page.svelte | Default query input page |
| src/routes/+layout.svelte | App shell layout with sidebar |
| src-tauri/tauri.conf.json | Tauri configuration (CSP, windows, plugins) |
| src-tauri/Cargo.toml | Rust dependency manifest |
| package.json | Frontend dependency manifest |

## Documentation
| Document | Path | Description |
|----------|------|-------------|
| README | README.md | Project landing page and branding |
| TASKS.md | TASKS.md | Full technical implementation plan with 55 subtasks |
| AGENTS.md | AGENTS.md | This file — project structure map |

## AI Context Files
| File | Purpose |
|------|---------|
| AGENTS.md | This file — project structure map |
| .ai-factory/DESCRIPTION.md | Project specification and tech stack |
| .ai-factory/ARCHITECTURE.md | Architecture decisions and guidelines |
| TASKS.md | Detailed implementation plan and subtask breakdown |
| .ai-factory/PROMPT.md | Тут сама задача, чтобы понять что нужно сделать. Промпт главной задачи |
