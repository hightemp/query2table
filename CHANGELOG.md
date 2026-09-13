# Changelog

## 0.6.0 — 2026-09-13

Query2Table now has a calmer, consistent desktop workspace across Query, Settings, History, all result modes, and dialogs.

### Interface and settings

- Refresh both light and dark themes with consistent controls, spacing, focus indicators, and compact status displays.
- Fix Settings scrollbars touching the cards; keep Save/Discard visible and add quick section navigation. Application files remains the last section.
- Preserve edits made during an in-flight save, retain unsaved values after partial failures, and offer Save / Discard / Stay when leaving Settings.
- Keep searchable model menus inside the window and support opening above or below their field.

### Results and run feedback

- Add continuous virtual scrolling, local filtering, keyboard sorting, and readable two-line previews to tables. Incoming rows preserve the reading position and current filter/sort.
- Display arrays, objects, missing values, zero, and false correctly. Row details show complete values, copy actions, extraction confidence, and saved source evidence.
- Put the Research answer before expandable activity; improve Markdown lists, wide tables, code, images, and long links.
- Improve image grid/list views and keyboard previews; prevent delayed image requests from replacing the current selection and explain failed previews.
- Make links and relevance scores easier to read, with consistent external-browser opening and copy actions.
- Unify dialogs with keyboard focus management. Export blocks dismissal while saving and confirms success; history deletion requires confirmation and reports failures.
- Separate run activity from the clearable diagnostic log, remove duplicate log events, use mode-specific counters, and stop presenting page-fetch ratios as overall completion.
- Preserve schema drafts through pause/resume, validate column names, and prevent repeated confirmation. Show model issues compactly with corrective actions and expandable details.

### Validation and screenshots

- Add production-preview regression checks for Chromium and WebKit, covering both themes and desktop window sizes down to 900×600.
- Refresh screenshots using a new, isolated run of “Find all YC-backed AI startups from 2024 with their funding amount, CEO name, and website”.

Existing settings and history are retained. This UI update requires no database format migration.

Known limitation observed in the live demo: year constraints are not consistently enforced by extraction. The screenshots preserve the actual output of the example query.

![Example YC startup research results](https://raw.githubusercontent.com/hightemp/query2table/v0.6.0/screenshots/yc-startups-table.png)

![Full result values and source evidence](https://raw.githubusercontent.com/hightemp/query2table/v0.6.0/screenshots/yc-startups-sources.png)

Download the installer for your platform from the release assets below: Linux (DEB, RPM, AppImage), Windows (EXE, MSI), or macOS Apple Silicon (DMG, app archive).

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
