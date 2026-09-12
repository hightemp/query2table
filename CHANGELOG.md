# Changelog

## 0.5.0 — 2026-09-12

- Add direct Ollama Cloud and configurable OpenAI-compatible providers, including llama.cpp, alongside OpenRouter and local Ollama.
- Load OpenRouter and Ollama Cloud model catalogs into searchable selectors.
- Add configurable thinking/reasoning effort, with safe Auto defaults and model compatibility guidance.
- Explain output/context limits, provider quotas, rate limits, authentication, connection, and response errors with corrective actions. Preserve model/stage/token diagnostics in run history, including skipped pages.
- Fix reasoning-only Ollama responses exhausting the answer budget and failed runs remaining visibly running.
- Make Pause, Resume, and Cancel responsive during requests in every search mode; preserve pending requests and schema edits across pause and stop owned workers on cancellation.
- Preserve legacy settings during automatic migration and expose data/database/log paths with Copy path and Open folder actions at the bottom of Settings.
- Refresh application branding, desktop icons, and README artwork.

Settings are updated automatically when the new version starts. Existing data locations and saved settings are retained.

Downloads are available on the [v0.5.0 release page](https://github.com/hightemp/query2table/releases/tag/v0.5.0).
