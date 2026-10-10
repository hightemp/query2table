# Changelog

## 0.9.0 — 2026-10-10

Query2Table comes to Android, reads the files you attach, and shows what a run is doing at every stage.

### Android app

- Query2Table now runs on Android phones (Android 7.0+): the same modes, settings and history, with a bottom tab bar and a layout for small screens. Install `Query2Table_0.9.0_android.apk` from the release assets.
- Data stays on the phone. Local model servers (Ollama, llama.cpp) can be used over your network by their address.
- Narrow desktop windows get the same compact layout.

### Attach files and images

- Attach PDF, Word, Excel, CSV, text and images to a query with the button, by dropping them on the form, or by pasting.
- Tables use attached files as sources next to web pages; rows link to the exact page, sheet or section. Research reads files on demand and cites them; Links and Images use them as context.
- **Files only** mode in Table and Research answers from your files without searching the web.
- Models that see images get pictures directly; scanned PDF pages and images can be read by a separate vision model. Choose it in Settings → LLM.
- In Images, an attached picture is a reference: the best candidates are compared with it.
- Attachments are kept with the run: History shows them, Run again reuses them, and exports turn file places into readable citations.

### Run progress

- Table, Links and Images runs show their stages (understanding the query, columns, planning, searching, reading, ranking, merging) with counts such as pages read and images ranked.
- Every search query is listed with its status, language, new results and errors. Saved runs show a folded summary.
- The separate “Activity” popover is gone; its information is in the stages.

### Tables

- Column names and descriptive values are written in the language of your query; names and titles stay as in the source.

### Look and controls

- More themes, including Solarized light and dark, and an interface size setting.
- All lists, checkboxes, radio buttons, number fields, sliders and tooltips are the app's own controls instead of the browser's, with a Cut / Copy / Paste menu in text fields.

### Settings

- New limits you can tune: model context size, file fragments per request, images sent to the model, and how many images are compared with a reference (Settings → Search).

Existing settings and history are kept; the database is upgraded automatically on first start.

Download the installer for your platform from the release assets below: Linux (DEB, RPM, AppImage), Windows (EXE, MSI), macOS Apple Silicon (DMG, app archive), or Android (APK).

## 0.8.0 — 2026-10-05

Query2Table now speaks Russian as well as English.

### Interface language

- Choose the language in Settings → Application: System, English or Русский. It applies at once, without a restart, and unsaved settings stay as they are. System follows the language of your operating system.
- Every screen is translated: queries and runs, tables, images, links, Research, History, Settings, logs, dialogs, menus and notifications. Error explanations and run notices explain what happened in the chosen language.
- Russian plural forms are used throughout (1 шаг, 3 шага, 5 шагов), and dates and numbers follow the chosen language.
- Desktop notifications about finished runs and the tray menu use the same language; the tray menu updates as soon as the language changes.
- Connection checks in Settings report their results in the chosen language.
- Technical details such as raw provider errors and logs stay in English, and model answers keep the language of your question.

### Also

- The theme switch in the sidebar names only the selected theme, so longer translations fit.
- Error hints point to the current names of Settings sections and fields.

Existing settings and history are kept; no database migration is needed.

Download the installer for your platform from the release assets below: Linux (DEB, RPM, AppImage), Windows (EXE, MSI), or macOS Apple Silicon (DMG, app archive).

## 0.7.0 — 2026-10-04

Research becomes a conversation, History becomes a searchable library, and Settings can test your connections before a run.

### Research

- Ask follow-up questions under an answer. The agent searches and reads again while taking earlier questions, answers and sources into account. Cancel stops only the current question.
- Answer, Activity and Sources tabs for every question. Sources list cited and read pages; Activity shows readable steps with page titles instead of raw page text.
- Copy an answer as Markdown, plain text or with its sources. Long answers get a contents list; text width and tables are easier to read.
- Suggested follow-up questions, the cost of each answer and of the whole conversation, and export of the whole conversation or a single question.
- The Max steps limit is now applied to the agent; the default is 100 steps (up to 200).

### Images and links

- Images: justified rows, filters by site and size, multi-select, save one or many images to a folder, and an app context menu instead of the webview one.
- Links: compact cards with descriptions and reasons, scores appear while ranking, visited state, grouping by site, hiding with undo, and copy formats for all or selected links.

### History

- Every run has its own page (`/history/<id>`): Back and reload keep your place, and the list returns to where you left it.
- Search queries and follow-up questions; filter by type and status; sort by date, results or cost; more runs load while scrolling.
- Compact rows grouped by day show results, duration, cost and model. Rename, pin, copy the query, export one or many runs, and delete with Undo.
- Run again repeats a run with its mode and limits (tables offer the earlier schema for review); Edit and run fills the query form.

### Run notices

- Model issues and cost warnings share one collapsible strip that no longer pushes results down.
- Each issue says what it meant for the result (retried, worked around, page skipped, simpler fallback, run stopped) and is grouped with repeats. Technical details stay collapsed. Notices can be marked as read.

### Settings

- Five sections with a section list, search (Ctrl+F), Advanced blocks, readiness status, and reset per field or section.
- Test connection for the LLM and search provider before saving; test proxies. Model lists for local Ollama and OpenAI-compatible servers.
- Sliders and range checks with errors at the field, switches for on/off options, hidden proxy passwords, theme choice, and settings import/export without API keys.
- Settings are saved in one transaction. “Page load timeout” and “Pages per query” now take effect (Links now reads at most 10 new pages per query by default); Precision/Recall and Evidence Strictness, which had no effect, were removed from the page.

### Also

- Design tokens replace Tailwind and Skeleton; toasts, keyboard shortcuts and theme persistence across the app.
- The dev server no longer applies component styles globally when CSS loads before its component.

Existing settings and history are kept; the database is updated automatically on first start.

Download the installer for your platform from the release assets below: Linux (DEB, RPM, AppImage), Windows (EXE, MSI), or macOS Apple Silicon (DMG, app archive).

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
