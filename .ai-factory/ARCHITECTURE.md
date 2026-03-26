# Architecture: Modular Monolith with Orchestrator Pattern

## Overview
Query2Table uses a **modular monolith** architecture with an **orchestrator + fixed roles** pattern. The application runs as a single Tauri desktop process with a Rust backend (async Tokio runtime) and a Svelte frontend (SPA in webview). All modules live in one binary — no microservices, no network boundaries between components.

The orchestrator pattern was chosen over free-form agents because:
1. **Predictability** — Every pipeline step has defined inputs/outputs and fixed behavior
2. **Debuggability** — State transitions are explicit and logged
3. **Cost control** — The orchestrator tracks budget and enforces limits
4. **Resume capability** — Pipeline state is persisted at checkpoints for crash recovery

## Decision Rationale
- **Project type:** Data pipeline desktop app with streaming results
- **Tech stack:** Rust + Tauri + Svelte + SQLite
- **Key factor:** Complex multi-stage pipeline requires explicit control flow and state management, not loose agent orchestration. Each "role" is a deterministic or LLM-backed function with a strict contract.

## Folder Structure
```
src-tauri/src/
├── main.rs                     # Tauri bootstrap
├── lib.rs                      # Module declarations
├── commands/                   # IPC boundary (thin handlers)
│   ├── mod.rs
│   ├── run.rs
│   ├── settings.rs
│   ├── history.rs
│   └── export.rs
├── orchestrator/               # Pipeline control
│   ├── mod.rs
│   ├── pipeline.rs             # State machine
│   ├── state.rs                # State types
│   ├── stop_controller.rs      # Stop conditions
│   └── budget_tracker.rs       # Cost tracking
├── roles/                      # Fixed-function pipeline stages
│   ├── mod.rs
│   ├── query_interpreter.rs
│   ├── schema_planner.rs
│   ├── search_planner.rs
│   ├── query_expander.rs
│   ├── search_executor.rs
│   ├── fetcher.rs
│   ├── document_parser.rs
│   ├── extractor.rs
│   ├── validator.rs
│   ├── deduplicator.rs
│   └── ui_event_publisher.rs
├── providers/                  # External service adapters
│   ├── mod.rs
│   ├── llm/                    # LLM providers (trait + implementations)
│   ├── search/                 # Search providers (trait + implementations)
│   └── http/                   # HTTP client, rate limiter, robots
├── storage/                    # Persistence layer
│   ├── mod.rs
│   ├── db.rs
│   ├── models.rs
│   └── repository.rs
├── export/                     # Export formats
│   ├── mod.rs
│   ├── csv.rs
│   ├── json.rs
│   └── xlsx.rs
└── utils/
    ├── mod.rs
    ├── logging.rs
    └── id.rs
```

## Dependency Rules

The architecture enforces a strict dependency direction. Inner layers MUST NOT depend on outer layers.

```
   Commands (IPC boundary)
       │ depends on ▼
   Orchestrator (pipeline control)
       │ depends on ▼
   Roles (pipeline stages)
       │ depends on ▼
   Providers (external APIs)   Storage (SQLite)
       │ depends on ▼             │ depends on ▼
   Utils (logging, id gen)     Utils
```

- ✅ `commands` → `orchestrator` → `roles` → `providers`, `storage`
- ✅ `roles` → `providers` (roles call LLM/search/HTTP providers)
- ✅ `roles` → `storage` (roles read/write via repository)
- ✅ `orchestrator` → `storage` (orchestrator persists pipeline state)
- ✅ Any module → `utils`
- ❌ `providers` → `roles` (providers don't know about pipeline roles)
- ❌ `storage` → `roles` (storage doesn't know about business logic)
- ❌ `roles` → `orchestrator` (roles don't control the pipeline)
- ❌ `roles` → `commands` (roles don't know about IPC)
- ❌ Any backend module → frontend code

## Layer/Module Communication

### Backend Layers

**Commands → Orchestrator:**
Commands are thin IPC handlers. They deserialize frontend requests, call orchestrator methods, and serialize responses.

```rust
#[tauri::command]
async fn start_run(
    state: State<'_, AppState>,
    query: String,
    config: RunConfig,
) -> Result<RunId, AppError> {
    state.orchestrator.start(query, config).await
}
```

**Orchestrator → Roles:**
The orchestrator calls roles sequentially or in parallel as needed. Roles are stateless functions — they receive inputs and return outputs.

```rust
// Orchestrator drives the pipeline
let intent = query_interpreter.interpret(&query, &llm).await?;
let schema = schema_planner.plan(&intent, &llm).await?;
// ... wait for user confirmation ...
let search_plan = search_planner.plan(&intent, &schema, &llm).await?;
```

**Roles → Providers:**
Roles interact with external services through provider traits.

```rust
// Role uses LLM provider via trait
pub trait LlmProvider: Send + Sync {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        model: &str,
        json_mode: bool,
    ) -> Result<String>;
}
```

**Roles → Storage:**
Roles persist results through a repository (not raw SQL).

```rust
pub trait Repository: Send + Sync {
    async fn insert_entity_row(&self, row: &EntityRow) -> Result<()>;
    async fn get_entity_rows(&self, run_id: &str) -> Result<Vec<EntityRow>>;
    // ...
}
```

### Frontend ↔ Backend

**Commands (invoke):** Frontend calls Rust functions via `invoke()`. Used for request-response: start_run, get_settings, export.

**Events (emit/listen):** Backend pushes real-time updates via Tauri events. Used for streaming: row_added, progress_update, log_entry.

```
Frontend                          Backend
   │                                │
   │──invoke("start_run")─────────►│
   │◄─────────Result<RunId>────────│
   │                                │
   │◄──event("row_added")─────────│  (streaming)
   │◄──event("progress_update")───│
   │◄──event("log_entry")─────────│
   │                                │
   │──invoke("cancel_run")────────►│
   │◄─────────Result<()>──────────│
```

## Key Design Patterns

### 1. Pipeline State Machine
The orchestrator implements an explicit state machine:

```
Pending → SchemaReview → Running → Completed
                │          │
                │          ├── Paused → Running (resume)
                │          └── Failed
                └── Cancelled
```

Each state transition is persisted to SQLite. On restart, the pipeline resumes from the last persisted state.

### 2. Provider Traits
All external services are behind traits. This enables:
- Swapping providers (OpenRouter ↔ Ollama, Brave ↔ Serper)
- Mock implementations for testing
- Adding new providers without changing roles

### 3. Channel-Based Worker Pools
Async workers communicate via bounded `tokio::sync::mpsc` channels:
- URL queue (SearchExecutor → Fetchers)
- Page queue (Fetchers → Extractors)
- Row queue (Extractors → Orchestrator)

Bounded channels provide natural backpressure.

### 4. Event Sourcing for Resume
Every significant action is logged to SQLite with status transitions. On crash recovery:
1. Load run state from `runs` table
2. Find incomplete work items (search_queries, search_results with `status = 'pending'`)
3. Resume from there

## Error Handling Strategy

- **Result<T, AppError>** everywhere — no panics in business logic
- **AppError** is an enum with variants for each error category
- **thiserror** for error derivation
- **Retries** with exponential backoff for transient failures (network, rate limits)
- **Graceful degradation** — if one page fails to extract, skip it and continue

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("LLM error: {0}")]
    Llm(String),
    #[error("Search API error: {0}")]
    Search(String),
    #[error("Fetch error: {url}: {message}")]
    Fetch { url: String, message: String },
    #[error("Storage error: {0}")]
    Storage(#[from] sqlx::Error),
    #[error("Export error: {0}")]
    Export(String),
    #[error("Configuration error: {0}")]
    Config(String),
}
```

## Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Rust modules | snake_case | `query_interpreter.rs` |
| Rust structs | PascalCase | `QueryInterpreter` |
| Rust traits | PascalCase | `LlmProvider` |
| Rust functions | snake_case | `interpret_query()` |
| Tauri commands | snake_case | `start_run` |
| Tauri events | snake_case with colon namespace | `run:row_added` |
| SQLite tables | snake_case plural | `entity_rows` |
| SQLite columns | snake_case | `created_at` |
| Svelte components | PascalCase | `ResultsTable.svelte` |
| Svelte stores | camelCase | `currentRun` |
| CSS classes | kebab-case | `.results-table` |
| TypeScript types | PascalCase | `EntityRow` |
