# AGENTS.md

> Project map for AI agents. Keep this file up-to-date as the project evolves.

Все задачи по проекту читать и помечать сделанное в TASKS.md.

## Project Overview
Query2Table is a local-first desktop application (Tauri v2 + Rust + Svelte) that converts natural-language research queries into structured tables with row-level sources. Uses an orchestrator with fixed roles to search the internet, fetch/parse pages, extract entities via LLMs, and deduplicate results.

## Tech Stack
- **Desktop Shell:** Tauri v2
- **Backend:** Rust (Tokio async runtime)
- **Frontend:** Svelte 5 (SvelteKit SPA) + Skeleton UI
- **Database:** SQLite (sqlx, WAL mode)
- **LLM:** OpenRouter (OpenAI-compatible) + Ollama (local)
- **Search:** Brave Search API + Serper API
- **Styling:** Plain CSS + Skeleton UI design system

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
│       ├── commands/           # Tauri IPC command handlers (run, settings, history, export)
│       ├── orchestrator/       # Pipeline state machine, stop controller, budget tracker
│       ├── roles/              # Fixed pipeline roles (13 roles: interpreter, planner, executor, etc.)
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
| TASKS.md | TASKS.md | Full technical implementation plan with 55 subtasks |
| AGENTS.md | AGENTS.md | This file — project structure map |

## AI Context Files
| File | Purpose |
|------|---------|
| AGENTS.md | This file — project structure map |
| .ai-factory/DESCRIPTION.md | Project specification and tech stack |
| .ai-factory/ARCHITECTURE.md | Architecture decisions and guidelines |
| TASKS.md | Detailed implementation plan and subtask breakdown |
