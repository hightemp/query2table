# AGENTS.md

> Project map for AI agents. Keep this file up-to-date as the project evolves.

Все задачи по проекту читать и помечать сделанное в TASKS.md.
Задачи можно считать выполненными, когда код для них написан и протестирован, а не просто когда они помечены как "готово" в таск-трекере.

## Project Overview
Query2Table is a local-first desktop application (Tauri v2 + Rust + Svelte) that converts natural-language research queries into structured tables with row-level sources. Uses an orchestrator with fixed roles to search the internet, fetch/parse pages, extract entities via LLMs, and deduplicate results.

## Tech Stack
- **Desktop Shell:** Tauri v2 (Linux, Windows, macOS) plus an Android app from the same code
- **Backend:** Rust (Tokio async runtime)
- **Frontend:** Svelte 5 (SvelteKit SPA)
- **Database:** SQLite (sqlx, WAL mode)
- **LLM:** OpenRouter, Anthropic (Messages API), Ollama (local and Cloud), OpenAI-compatible presets (DeepSeek, Groq, Mistral, Gemini, xAI, OpenAI, LM Studio, llama.cpp) and a custom OpenAI-compatible address
- **Search:** Brave Search API + Serper API
- **Styling:** Plain CSS with design tokens in `src/app.css` (`--app-*`), shared `.button`/`.input` classes and `Badge`/`EmptyState` components

## Project Structure
```
query2table/
├── src-tauri/                  # Rust backend
│   ├── Cargo.toml              # Rust dependencies
│   ├── tauri.conf.json         # Tauri configuration
│   ├── migrations/             # SQLite schema migrations
│   ├── gen/android/            # Android Studio project (committed; signing via git-ignored keystore.properties)
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
Images go to models as `Message.images` (`ImageInput`; OpenAI-compatible sends `image_url` parts, Ollama `images`). `providers/llm/capabilities.rs` detects vision from catalogs and resolves `VisionPlan` from `llm_vision` / `vision_model`; `orchestrator/files.rs` (`RunFiles`) gives pipelines the files of a run: labelled `F1…`, an overview plus matching fragments for the first message, inline images for models that see them, and `read` for the research agent's `read_file` action (step type `read`, cited by `attachment://` URL). `PipelineConfig.web_search = false` is the files-only mode (Research and Table). In tables, the interpreter and planners get `QueryIntent.files_context`, and every file fragment is extracted like a page (row sources keep the `attachment://` address, without a fetched page). Links and Images use files as context only (`files::load_for_run`); exports and copied answers turn `attachment://` places into readable text (`readable_source` in `commands/export.rs`, `readableAnswer` in `utils/research.ts`), and runs record `source_mode` in their config so Run again repeats the files and the mode; in Images an attached picture is a reference: `ImageRanker::rank_with_reference` re-scores the top text-ranked candidates with the model for images. User-tunable limits live in `PipelineConfig` (`image_compare_max`, `max_file_fragments`, `max_inline_images`, `research_max_images`, and `context: ContextBudget` from the `llm_context_size` preset); safety limits (archive and text size caps, download sizes) and algorithm details (fragment size, batch sizes, render sizes) stay constants. `attachments::vision::prepare` reads scanned pages (rendered with `hayro`, needs Rust ≥ 1.92) and describes pictures once, storing the text as fragments.

## Model providers
OpenRouter, Anthropic, Ollama and Ollama Cloud have clients of their own; every other service speaking the OpenAI Chat Completions API is a preset run by `OpenAiCompatibleProvider` (`providers/llm/presets.rs`, mirrored in `src/lib/settings/llmProviders.ts`; a Rust test checks both lists match). A preset is an `llm_provider` value with its own settings `<id>_api_key`, `<id>_model`, `<id>_json_mode` and, for local servers, `<id>_url`; `LlmManager::config_from_settings` resolves it into the `openai_*` fields of `LlmConfig` and keeps the preset in `LlmConfig.preset` (its id is the provider name in accounting and the scope of manual rates). "Custom address" is `openai_compatible` with the `openai_*` settings. Preset flags cover request differences (`reasoning_effort` support, `max_completion_tokens`, OpenAI reasoning models without temperature). Prices of cloud presets are set by hand (Settings › Model › Pricing); local presets and models the local Ollama runs itself cost $0 (`PriceQuote::local`; cloud models through it are not free); preset catalogs say nothing about images, so "Pictures: auto" treats their models as text-only. On the frontend use `providerName`, `modelKey`, `apiKeyKey`, `urlKey`, `endpointOf` from `llmProviders.ts` instead of per-provider maps.
Anthropic (`providers/llm/anthropic.rs`, settings `anthropic_api_key`, `anthropic_model`, `anthropic_native_pdf`) uses the Messages API: top-level `cache_control` caches the growing prompt, usage counts cache reads and writes (`input_tokens` excludes them), every Claude model sees images. Reasoning follows the model generation: Claude 4.5 and earlier get `temperature` and `thinking.budget_tokens`; later models reject other temperatures and budgets, so they get `thinking: adaptive` with `output_config.effort` ("Off" = low effort, since some models always think). There is no JSON mode: the rule goes into the system prompt and a ```json fence is removed. With "Claude reads PDF files itself" (`LlmConfig::native_pdf`), the research first message carries attached PDFs as `Message.documents` (`DocumentInput`, at most 20 MB and 100 pages in all; larger ones stay text), their text fragments are left out of the context and their scanned pages are not transcribed; other modes read PDFs as text.

## Run stages
Table, Links and Images runs report their stage (`run:stage`: interpret, schema, plan, search, read, rank, compare, dedup) and every search query (`run:search_queries`, `run:search_query`) through `orchestrator/search_log.rs`; query status, new-result count and error are kept in `search_queries` (`get_run_queries`). The query page shows `RunStages` in place of results not found yet, then as a folded line; saved runs show the folded summary.

## Updates
Releases publish a signed `latest.json` (tauri-action; the Android job adds an `android-universal` entry with the APK and its signature). `src-tauri/src/updates/` has the commands `check_update`, `install_update` (progress as `update:progress`), `restart_app`: desktop uses `tauri-plugin-updater`, Android downloads the APK, verifies it with `updates/signature.rs` (same minisign key, signature bound to the version) and opens the installer through `AndroidFilesPlugin.installApk`. Settings live in `plugins.updater` of `tauri.conf.json` (`requireSignedVersion` on; the public key is written by `make updater-keys`); update artifacts are built only with `src-tauri/tauri.updater.conf.json` (CI), so local builds need no key. The frontend store is `src/lib/stores/updates.ts` (daily startup check, skipped version), with `UpdateDialog`, the sidebar notice and Settings › Application › Updates (`AppUpdates`).

## Images in research answers
The research agent has `image_search` and, for models that see images, `view_image` (`Tools.images`/`Tools.view`, setting `research_images`). Page images reach it as `![alt](absolute url)` (`absolute_images` in `roles/research_agent.rs`). Images it came across are remembered per turn (`SeenImage`); after the answer, `keep_answer_images` in `orchestrator/research_pipeline.rs` downloads only those the answer shows into the `research_images` table (downscaled, with the source page). Image downloading and downscaling live in `providers/http/images.rs`. The frontend never loads answer images from the web: `Markdown.svelte` fills placeholders from `get_research_image` (data URL) given the run's `get_research_images` list from `ResearchView`.

## Math formulas
Research answers render LaTeX. Parse Markdown with `markdown` from `src/lib/utils/math.ts` (a `Marked` instance with the formula extension), not the global `marked`: it yields `math` tokens for `\(…\)`, `\[…\]`, `$$…$$` and `$…$` (pandoc's rule, so prices stay text) and renders them as `.math[data-tex]` placeholders. `Markdown.svelte` draws them with MathJax 4 from `src/lib/utils/mathRender.ts`, loaded on demand; the font's glyph files come from the bundle through `import.meta.glob`. Copies and exports keep the LaTeX source.

## UI controls
Do not use the browser's own controls. Use `Select` (lists; search appears from 8 options), `Checkbox`, `Radio`, `NumberInput` and `Slider` from `src/lib/components/common/`, and `use:tooltip` from `src/lib/actions/tooltip.ts` instead of `title` (`{ text, whenTruncated: true }` for clipped text). `<details>`, scrollbars and number spinners are styled globally in `src/app.css`. Right-click on text fields and selected text opens the app's menu (`src/lib/utils/textMenu.ts`). In Playwright tests pick list options with `choose()` from `tests/ui/fixtures.ts`.

## Android
`make android-apk` builds `dist/Query2Table_<version>_android.apk`; `make android-run` installs and opens it on the connected phone or emulator, `make android-logs` follows its log, `make android-dev` runs with live reload. Desktop-only code (tray) is behind `#[cfg(desktop)]`; mobile keeps its data in the app's private storage (`utils/paths.rs`). On Android the file dialog returns `content://` addresses, not paths: read and write picked files with `utils/files.rs` (fs plugin; `write_through` for exporters that need a path), never `std::fs`. A folder cannot be picked and written on Android: several files go to Pictures/Download `Query2Table` via `utils/android_files.rs` and `gen/android/.../AndroidFilesPlugin.kt`, which also opens attached files in other apps through the FileProvider (paths cannot be opened there). Android's IPC carries JSON only, so binary uploads go as base64 (`add_attachment_data`); the frontend checks `isAndroid()` from `$lib/utils/platform`. The phone layout is CSS at `max-width: 640px` (bottom tab bar in `Sidebar.svelte`), tested in `tests/ui/mobile.spec.ts`. Releases add the APK in the `android` job of `.github/workflows/release.yml`, signed with the `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_PASSWORD`, `ANDROID_KEY_ALIAS` secrets (`make android-keystore` creates the key).

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
