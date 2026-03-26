# Project: Query2Table

## Overview
A fully local desktop application that accepts natural-language research queries, searches the public internet asynchronously, extracts structured entities, and builds a table of results with row-level sources. A universal research tool: the query can target companies, people, events, products, articles, jobs, laws, websites, or any other entity type.

## Core Features
- Natural language query input with automatic schema inference
- User confirmation/editing of inferred table schema before execution
- Adaptive multi-step internet search with query expansion and multilingual support
- Asynchronous pipeline with streaming results into a live table
- Row-level source attachment (evidence for each extracted entity)
- Entity deduplication across domains, languages, and naming variations
- Configurable stop conditions (row count, budget, time, saturation)
- Full run history with persistence and resume capability
- Export to CSV, JSON, XLSX
- Local-first: no cloud backend, all data stored locally in SQLite
- System tray with completion notifications

## Tech Stack
- **Desktop Shell:** Tauri v2
- **Backend Language:** Rust
- **Frontend Framework:** Svelte (SvelteKit in SPA mode)
- **Frontend Styling:** Plain CSS + Skeleton UI design system
- **Database:** SQLite (via sqlx)
- **Async Runtime:** Tokio
- **HTTP Client:** reqwest
- **HTML Parsing:** scraper + ammonia
- **Search APIs:** Brave Search API, Serper API (user-configurable primary + fallback)
- **LLM Access:** OpenRouter (OpenAI-compatible API), Ollama (local models)
- **Default LLM Model:** openai/gpt-5.4-mini (configurable per stage)
- **Table Component:** TanStack Table (Svelte adapter)

## Architecture Notes
- Orchestrator + fixed roles pattern (not free-form agents)
- Event-sourced pipeline state for resume capability
- Tauri IPC for frontend-backend communication
- Event streaming from Rust to Svelte via Tauri events
- Per-domain rate limiting for ethical web scraping
- robots.txt compliance

## Non-Functional Requirements
- **Logging:** Configurable via LOG_LEVEL, file logs + GUI log viewer
- **Error handling:** Structured error types, retries with exponential backoff
- **Security:** API keys stored locally (never transmitted except to configured APIs), robots.txt respect
- **Performance:** 8 parallel fetch workers, 3 parallel LLM calls, rate limiting per domain
- **Platforms:** Windows, macOS, Linux
- **Auto-update:** Tauri updater
- **CI/CD:** GitHub Actions for multi-platform builds
- **Theme:** Dark/light theme support

## MVP Scope
- Full agent search pipeline (query → schema → search → fetch → extract → validate → dedup → table)
- Resume interrupted runs
- Export (CSV, JSON, XLSX)
- Multilingual query expansion
- Full settings panel (API keys, models, quality controls)
- Run history
- System tray notifications
- Logging (file + GUI)

## Phase 2 (Post-MVP)
- JS-rendered page support (headless browser via chromiumoxide)
- PDF document parsing
- Templates/presets system
- Advanced LLM-assisted deduplication
- Plugin system for custom extractors
