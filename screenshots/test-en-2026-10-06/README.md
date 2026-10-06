# Query2Table — English native screenshots

Captured on 6 October 2026 from Query2Table 0.8.0 (source `2294701`). All **69 PNG screenshots are 1440 × 960**.

Open [index.html](index.html) for the gallery. Click a thumbnail to open its original PNG. [overview.jpg](overview.jpg) gives a quick preview. `manifest.json` records every image and its SHA-256 digest.

## Test environment and data

The actual Linux Tauri/GTK application ran on an isolated virtual display with separate XDG data and configuration directories. It used the existing local frontend dev server and its own Rust backend process. Only provider settings were copied from the current application; its history and results were not copied. Current provider credentials were used for live requests and are masked in the screenshots. No credentials or application database are included in this collection.

- **Main comparison table:** 17 rows, six data columns, confidence and row-level sources. The real pipeline collected 20 rows and merged three duplicates in 149 seconds, with 52 searches and 13 LLM calls. The run used OpenRouter `openai/gpt-4.1-mini` and Serper. Qdrant has three sources; Weaviate has two. Sorting, keyboard column resizing, global search, a language-specific filter, hidden columns, compact rows, next-row navigation and CSV export were exercised in the native UI.
- **Images:** 8 live results for James Webb Space Telescope imagery, with grid/list views and a loaded original-image preview.
- **Links:** 33 saved candidates, with 4 considered relevant by the pipeline; selection, contextual actions and the confirmation for opening many links were captured.
- **Research:** an English comparison of Qdrant and Milvus, with 8 activity steps and 7 displayed sources. The answer, activity, sources, follow-up limits and Markdown export were captured.
- Images, links and research used the copied Ollama Cloud / Brave configuration.

The comparison CSV in [examples/table-comparison.csv](examples/table-comparison.csv) contains all 17 rows and eight exported columns, including confidence and sources. [examples/research.md](examples/research.md) is the application-generated Markdown export. [examples/queries-and-results.json](examples/queries-and-results.json) records the queries and final statistics. [examples/sample-settings.json](examples/sample-settings.json) is the two-setting, credential-free file used for the import dialog.

## Observed limits

The saved model answers and extracted fields are shown as produced by the application. They include missing values, project aliases and entries outside the requested open-source list. The dataset was not manually corrected for presentation. Some fetched pages failed; the comparison run fetched 22 pages and recorded 12 failures. Cost warnings and partial/unknown amounts are left visible; a displayed partial amount is not a complete bill.

The first broad table attempt reached its three-minute limit without rows. A one-row Qdrant retry completed, and the main table screenshots were then replaced with the richer 17-row comparison requested for this collection. Run-control screenshots 03–05 show the earlier live attempt. A separate short run was deliberately cancelled to capture confirmation, cancellation and logs. Local Ollama / OpenAI-compatible settings show their actual unconfigured connection states.

## Screen inventory

| Screenshot | Screen or dialog |
| --- | --- |
| [01-query-table.png](01-query-table.png) | New query — Table mode |
| [02-query-stop-conditions.png](02-query-stop-conditions.png) | Comparison query and stop conditions |
| [03-run-in-progress.png](03-run-in-progress.png) | Live table run |
| [04-run-paused.png](04-run-paused.png) | Paused run |
| [05-schema-review.png](05-schema-review.png) | Proposed schema editor |
| [06-settings-llm.png](06-settings-llm.png) | LLM settings |
| [07-settings-model-picker.png](07-settings-model-picker.png) | Cloud model picker and connection check |
| [08-settings-search.png](08-settings-search.png) | Search settings |
| [09-settings-runs.png](09-settings-runs.png) | Run settings |
| [10-settings-runs-advanced.png](10-settings-runs-advanced.png) | Advanced run settings |
| [11-settings-network.png](11-settings-network.png) | Network and proxy settings |
| [12-settings-application.png](12-settings-application.png) | Application settings and file locations |
| [13-settings-openrouter.png](13-settings-openrouter.png) | OpenRouter settings |
| [14-settings-ollama.png](14-settings-ollama.png) | Local Ollama settings |
| [15-settings-openai-compatible.png](15-settings-openai-compatible.png) | OpenAI-compatible settings |
| [16-settings-llm-advanced.png](16-settings-llm-advanced.png) | Advanced LLM settings |
| [17-settings-unsaved-dialog.png](17-settings-unsaved-dialog.png) | Unsaved settings dialog |
| [18-history.png](18-history.png) | Run history |
| [19-settings-import-file-dialog.png](19-settings-import-file-dialog.png) | Import settings file chooser |
| [20-settings-import-preview.png](20-settings-import-preview.png) | Imported settings review |
| [21-research-answer.png](21-research-answer.png) | Research answer |
| [22-research-activity.png](22-research-activity.png) | Research activity |
| [23-research-sources.png](23-research-sources.png) | Research sources |
| [24-research-export-dialog.png](24-research-export-dialog.png) | Research export dialog |
| [25-research-followup-limits.png](25-research-followup-limits.png) | Follow-up limits |
| [26-usage-and-cost.png](26-usage-and-cost.png) | Usage and cost details |
| [27-run-notices.png](27-run-notices.png) | Run notices |
| [28-images-grid.png](28-images-grid.png) | Image grid |
| [29-images-list.png](29-images-list.png) | Image list |
| [30-image-preview.png](30-image-preview.png) | Image preview dialog |
| [31-image-context-menu.png](31-image-context-menu.png) | Image context menu |
| [32-links-results.png](32-links-results.png) | Link results |
| [33-links-selection.png](33-links-selection.png) | Selected links |
| [34-links-open-confirm-dialog.png](34-links-open-confirm-dialog.png) | Open multiple links confirmation |
| [35-link-context-menu.png](35-link-context-menu.png) | Link context menu |
| [36-history-bulk-export-dialog.png](36-history-bulk-export-dialog.png) | Bulk export dialog |
| [37-history-context-menu.png](37-history-context-menu.png) | History context menu |
| [38-history-filtered.png](38-history-filtered.png) | Filtered history |
| [39-query-images.png](39-query-images.png) | New query — Images mode |
| [40-query-links.png](40-query-links.png) | New query — Links mode |
| [41-query-research.png](41-query-research.png) | New query — Research mode |
| [42-settings-export-file-dialog.png](42-settings-export-file-dialog.png) | Export settings file chooser |
| [43-settings-search-connection.png](43-settings-search-connection.png) | Search connection check |
| [44-table-results.png](44-table-results.png) | Comparison table — 17 rows |
| [45-row-details-and-sources.png](45-row-details-and-sources.png) | Qdrant details and three sources |
| [46-table-columns-menu.png](46-table-columns-menu.png) | Table column menu |
| [47-table-export-dialog.png](47-table-export-dialog.png) | Table export dialog |
| [48-export-save-file-dialog.png](48-export-save-file-dialog.png) | Export save file chooser |
| [49-export-complete.png](49-export-complete.png) | Export complete |
| [50-table-dark.png](50-table-dark.png) | Comparison table — dark theme |
| [51-images-dark.png](51-images-dark.png) | Image grid — dark theme |
| [52-links-dark.png](52-links-dark.png) | Links — dark theme |
| [53-research-dark.png](53-research-dark.png) | Research — dark theme |
| [54-history-dark.png](54-history-dark.png) | History — dark theme |
| [55-settings-dark.png](55-settings-dark.png) | Settings — dark theme |
| [56-cancel-run-dialog.png](56-cancel-run-dialog.png) | Cancel run confirmation |
| [57-run-cancelled.png](57-run-cancelled.png) | Cancelled run |
| [58-log-panel.png](58-log-panel.png) | Log panel |
| [59-bulk-export-folder-dialog.png](59-bulk-export-folder-dialog.png) | Bulk export folder chooser |
| [60-bulk-export-complete.png](60-bulk-export-complete.png) | Bulk export complete |
| [61-research-recommendation.png](61-research-recommendation.png) | Research recommendation and sources |
| [62-sidebar-collapsed.png](62-sidebar-collapsed.png) | Comparison table — collapsed sidebar |
| [63-row-details-next.png](63-row-details-next.png) | Next row details |
| [64-table-filtered.png](64-table-filtered.png) | Table search — Rust |
| [65-table-sorted-by-language.png](65-table-sorted-by-language.png) | Table sorted by language |
| [66-table-compact-rows.png](66-table-compact-rows.png) | Compact table rows |
| [67-table-selected-columns.png](67-table-selected-columns.png) | Selected table columns |
| [68-table-column-filter.png](68-table-column-filter.png) | Language column filter — Go |
| [69-table-more-results.png](69-table-more-results.png) | More table results |

The collection covers every application route, the four result modes, all settings sections and providers, and the application dialogs found in the current source. Selected main screens are also captured in the dark theme.
