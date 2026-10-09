import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
	SearchQueryInfo,
	AttachmentFragment,
	AttachmentInfo,
	AttachResult,
	RunAttachment,
	StartRunResponse,
	RunInfo,
	RunLogEntry,
	SchemaColumn,
	HistoryRun,
	StatusChangedEvent,
	RowAddedEvent,
	AccountingEvent,
	RowsReplacedEvent,
	ProgressEvent,
	LogEntryEvent,
	SchemaProposedEvent,
	RunErrorEvent,
	LlmIssueEvent,
	ImageResult,
	ImageAddedEvent,
	LinkResult,
	LinkAddedEvent,
	ResearchResult,
	ResearchStepEvent,
	ResearchAnswerEvent,
	RowSource,
} from '$lib/types';

export interface Setting {
	key: string;
	value: string;
}

export interface AppPaths {
	data_dir: string;
	database_file: string;
	log_dir: string;
}

export type AppLocation = 'data' | 'database' | 'logs';

export function getAppPaths(): Promise<AppPaths> {
	return invoke('get_app_paths');
}

export function copyAppPath(location: AppLocation): Promise<void> {
	return invoke('copy_app_path', { location });
}

export function copyText(text: string): Promise<void> {
	return invoke('copy_text', { text });
}

export function pasteText(): Promise<string> {
	return invoke('paste_text');
}

export function getRowSources(rowId: string): Promise<RowSource[]> {
	return invoke('get_row_sources', { rowId });
}

export function openAppFolder(location: AppLocation): Promise<void> {
	return invoke('open_app_folder', { location });
}

export async function getSettings(): Promise<Setting[]> {
	return invoke('get_settings');
}

export async function getSetting(key: string): Promise<string | null> {
	return invoke('get_setting', { key });
}

export async function updateSetting(key: string, value: string): Promise<void> {
	return invoke('update_setting', { key, value });
}

export async function listOllamaCloudModels(baseUrl: string, apiKey: string): Promise<string[]> {
	return invoke('list_ollama_cloud_models', { baseUrl, apiKey });
}

export async function listOpenRouterModels(apiKey: string): Promise<string[]> {
	return invoke('list_openrouter_models', { apiKey });
}

export interface LogEntry {
	timestamp: string;
	level: string;
	message: string;
	target?: string;
}

export function onLogEvent(callback: (entry: LogEntry) => void): Promise<UnlistenFn> {
	return listen<LogEntry>('log-event', (event) => {
		callback(event.payload);
	});
}

// --- Run commands ---

export interface StopConditions {
	target_row_count?: number;
	max_budget_usd?: number;
	max_duration_seconds?: number;
}

export async function startRun(
	query: string,
	runType?: string,
	stopConditions?: StopConditions,
	/** Table runs: columns to offer for review instead of planning new ones. */
	schema?: SchemaColumn[] | null,
	/** Ids of attached files. */
	attachments?: string[],
	/** `files`: work from the attached files only, without web search. */
	sourceMode?: SourceMode
): Promise<StartRunResponse> {
	return invoke('start_run', {
		query,
		runType: runType ?? null,
		stopConditions: stopConditions ?? null,
		schema: schema ?? null,
		attachments: attachments?.length ? attachments : null,
		sourceMode: sourceMode ?? null,
	});
}

/** Where a run looks: attached files and the web, or the files only. */
export type SourceMode = 'web' | 'files';

// --- Attachments ---

export function addAttachments(paths: string[]): Promise<AttachResult[]> {
	return invoke('add_attachments', { paths });
}

/** Attaches pasted or dropped file content; the bytes travel as the raw request body. */
export async function addAttachmentData(file: File): Promise<AttachResult> {
	const bytes = new Uint8Array(await file.arrayBuffer());
	return invoke('add_attachment_data', bytes, { headers: { 'x-file-name': encodeURIComponent(file.name || 'pasted') } });
}

export function getAttachments(ids: string[]): Promise<AttachmentInfo[]> {
	return invoke('get_attachments', { ids });
}

export function removeAttachment(id: string): Promise<void> {
	return invoke('remove_attachment', { id });
}

export function getRunAttachments(runId: string): Promise<RunAttachment[]> {
	return invoke('get_run_attachments', { runId });
}

export function openAttachment(id: string): Promise<void> {
	return invoke('open_attachment', { id });
}

export interface VisionStatus {
	/** The main model gets images directly. */
	main_sees: boolean;
	/** What the provider's catalog says about the main model; null when unknown. */
	detected: boolean | null;
	/** The model that reads pictures and scanned pages; null when none can. */
	reader: string | null;
}

/** Who reads images with the saved settings, with optional unsaved edits applied. */
export function getVisionStatus(overrides?: Record<string, string>): Promise<VisionStatus> {
	return invoke('get_vision_status', { overrides: overrides ?? null });
}

export function getAttachmentFragment(url: string): Promise<AttachmentFragment | null> {
	return invoke('get_attachment_fragment', { url });
}

// --- History list ---

export interface HistoryPage {
	runs: HistoryRun[];
	counts: Record<string, number>;
}

export async function listHistory(filter: {
	search: string | null;
	run_type: string | null;
	status: string | null;
	sort: string;
	limit: number;
	offset: number;
}): Promise<HistoryPage> {
	return invoke('list_history', { filter });
}

/** Hides runs; `restoreRuns` brings them back until `purgeRuns` or the app closes. */
export async function deleteRuns(runIds: string[]): Promise<void> {
	return invoke('delete_runs', { runIds });
}

export async function restoreRuns(runIds: string[]): Promise<void> {
	return invoke('restore_runs', { runIds });
}

export async function purgeRuns(runIds: string[]): Promise<void> {
	return invoke('purge_runs', { runIds });
}

export async function renameRun(runId: string, title: string | null): Promise<void> {
	return invoke('rename_run', { runId, title });
}

export async function pinRun(runId: string, pinned: boolean): Promise<void> {
	return invoke('pin_run', { runId, pinned });
}

/** One file per run in `dir`; research runs are Markdown. Returns the written paths. */
export async function exportRuns(runIds: string[], dir: string, format: string): Promise<string[]> {
	return invoke('export_runs', { runIds, dir, format });
}

export async function cancelRun(runId: string): Promise<void> {
	return invoke('cancel_run', { runId });
}

export async function pauseRun(runId: string): Promise<void> {
	return invoke('pause_run', { runId });
}

export async function resumeRun(runId: string): Promise<void> {
	return invoke('resume_run', { runId });
}

export async function confirmSchema(runId: string, columns: SchemaColumn[]): Promise<void> {
	return invoke('confirm_schema', { runId, columns });
}

export async function getRun(runId: string): Promise<RunInfo | null> {
	return invoke('get_run', { runId });
}

export async function listRuns(limit?: number, offset?: number): Promise<RunInfo[]> {
	return invoke('list_runs', { limit: limit ?? null, offset: offset ?? null });
}

export async function deleteRun(runId: string): Promise<void> {
	return invoke('delete_run', { runId });
}

export async function getRunLogs(runId: string): Promise<RunLogEntry[]> {
	return invoke('get_run_logs', { runId });
}

// --- Run event listeners ---

export function onStatusChanged(cb: (e: StatusChangedEvent) => void): Promise<UnlistenFn> {
	return listen<StatusChangedEvent>('run:status_changed', (event) => cb(event.payload));
}

export function onRowAdded(cb: (e: RowAddedEvent) => void): Promise<UnlistenFn> {
	return listen<RowAddedEvent>('run:row_added', (event) => cb(event.payload));
}

export function onRowsReplaced(cb: (e: RowsReplacedEvent) => void): Promise<UnlistenFn> {
	return listen<RowsReplacedEvent>('run:rows_replaced', (event) => cb(event.payload));
}

export function onAccounting(cb: (event: AccountingEvent) => void): Promise<UnlistenFn> {
	return listen<AccountingEvent>('run:accounting', (event) => cb(event.payload));
}

export function onProgressUpdate(cb: (e: ProgressEvent) => void): Promise<UnlistenFn> {
	return listen<ProgressEvent>('run:progress_update', (event) => cb(event.payload));
}

export function onRunStage(cb: (e: { run_id: string; stage: string }) => void): Promise<UnlistenFn> {
	return listen<{ run_id: string; stage: string }>('run:stage', (event) => cb(event.payload));
}

export function onSearchQueries(cb: (e: { run_id: string; queries: SearchQueryInfo[] }) => void): Promise<UnlistenFn> {
	return listen<{ run_id: string; queries: SearchQueryInfo[] }>('run:search_queries', (event) => cb(event.payload));
}

export function onSearchQuery(cb: (e: { run_id: string; query: SearchQueryInfo }) => void): Promise<UnlistenFn> {
	return listen<{ run_id: string; query: SearchQueryInfo }>('run:search_query', (event) => cb(event.payload));
}

/** The search queries of a run and what happened to each. */
export function getRunQueries(runId: string): Promise<SearchQueryInfo[]> {
	return invoke('get_run_queries', { runId });
}

export function onRunLogEntry(cb: (e: LogEntryEvent) => void): Promise<UnlistenFn> {
	return listen<LogEntryEvent>('run:log_entry', (event) => cb(event.payload));
}

export function onSchemaProposed(cb: (e: SchemaProposedEvent) => void): Promise<UnlistenFn> {
	return listen<SchemaProposedEvent>('run:schema_proposed', (event) => cb(event.payload));
}

export function onRunError(cb: (e: RunErrorEvent) => void): Promise<UnlistenFn> {
	return listen<RunErrorEvent>('run:error', (event) => cb(event.payload));
}

export function onLlmIssue(cb: (e: LlmIssueEvent) => void): Promise<UnlistenFn> {
	return listen<LlmIssueEvent>('run:llm_issue', (event) => cb(event.payload));
}

// --- Settings page ---

/** Saves several settings at once: all of them or none. */
export async function updateSettings(values: Record<string, string>): Promise<void> {
	return invoke('update_settings', { values });
}

export interface ConnectionReport {
	ok: boolean;
	/** English text, used when there is no translation for `code`. */
	message: string;
	code?: string | null;
	params?: Record<string, string>;
}

/** Checks LLM credentials and model with unsaved form values; generates nothing. */
export async function testLlmConnection(settings: Record<string, string>): Promise<ConnectionReport> {
	return invoke('test_llm_connection', { settings });
}

/** Runs one (billable) search with unsaved form values. */
export async function testSearchConnection(settings: Record<string, string>): Promise<ConnectionReport> {
	return invoke('test_search_connection', { settings });
}

export async function testProxy(url: string): Promise<ConnectionReport> {
	return invoke('test_proxy', { url });
}

export async function listOllamaModels(baseUrl: string): Promise<string[]> {
	return invoke('list_ollama_models', { baseUrl });
}

export async function listOpenAiModels(baseUrl: string, apiKey: string): Promise<string[]> {
	return invoke('list_openai_models', { baseUrl, apiKey });
}

/** Writes settings without API keys or proxies to a JSON file. */
export async function exportSettings(path: string): Promise<void> {
	return invoke('export_settings', { path });
}

/** Settings from a file, to apply as unsaved changes. */
export async function readSettingsFile(path: string): Promise<Record<string, string>> {
	return invoke('read_settings_file', { path });
}

export async function getRunIssues(runId: string): Promise<LlmIssueEvent[]> {
	return invoke('get_run_issues', { runId });
}

/** Remembers how many of each kind of notice the reader has seen. */
export async function dismissRunNotices(runId: string, dismissed: Record<string, number>): Promise<void> {
	return invoke('dismiss_run_notices', { runId, dismissed });
}

// --- History data fetching ---

export interface RunSchemaInfo {
	columns: SchemaColumn[];
	confirmed: boolean;
}

export interface EntityRowInfo {
	id: string;
	data: Record<string, unknown>;
	confidence: number;
	status: string;
	source_count?: number;
}

export async function getRunSchema(runId: string): Promise<RunSchemaInfo | null> {
	return invoke('get_run_schema', { runId });
}

export async function getRunRows(runId: string): Promise<EntityRowInfo[]> {
	return invoke('get_run_rows', { runId });
}

export async function getImageResults(runId: string): Promise<ImageResult[]> {
	return invoke('get_image_results', { runId });
}

export async function proxyImage(url: string): Promise<string> {
	return invoke('proxy_image', { url });
}

/** Saves one image (or its fallback, usually the thumbnail) and returns the written path. */
export function saveImage(url: string, fallbackUrl: string | null, path: string): Promise<string> {
	return invoke('save_image', { url, fallbackUrl, path });
}

export interface SaveImagesResult {
	saved: string[];
	failed: [string, string][];
}

export function saveImages(
	items: { url: string; fallback_url: string | null; name: string }[],
	directory: string
): Promise<SaveImagesResult> {
	return invoke('save_images', { items, directory });
}

export function onImageAdded(cb: (e: ImageAddedEvent) => void): Promise<UnlistenFn> {
	return listen<ImageAddedEvent>('run:image_added', (event) => cb(event.payload));
}

export async function getLinkResults(runId: string): Promise<LinkResult[]> {
	return invoke('get_link_results', { runId });
}

export function setLinkVisited(linkId: string, visited: boolean): Promise<void> {
	return invoke('set_link_visited', { linkId, visited });
}

export function setLinkHidden(linkId: string, hidden: boolean): Promise<void> {
	return invoke('set_link_hidden', { linkId, hidden });
}

export function onLinkAdded(cb: (e: LinkAddedEvent) => void): Promise<UnlistenFn> {
	return listen<LinkAddedEvent>('run:link_added', (event) => cb(event.payload));
}

export async function getResearchResult(runId: string): Promise<ResearchResult> {
	return invoke('get_research_result', { runId });
}

/** Asks a follow-up question in a research conversation. */
export function askFollowUp(
	runId: string,
	question: string,
	stopConditions: StopConditions,
	attachments?: string[],
	sourceMode?: SourceMode
): Promise<void> {
	return invoke('ask_follow_up', {
		runId,
		question,
		stopConditions,
		attachments: attachments?.length ? attachments : null,
		sourceMode: sourceMode ?? null,
	});
}

export function onResearchStep(cb: (e: ResearchStepEvent) => void): Promise<UnlistenFn> {
	return listen<ResearchStepEvent>('run:research_step', (event) => cb(event.payload));
}

export function onResearchAnswer(cb: (e: ResearchAnswerEvent) => void): Promise<UnlistenFn> {
	return listen<ResearchAnswerEvent>('run:research_answer', (event) => cb(event.payload));
}

// --- Export commands ---

/** `turnIndex` exports one turn of a research conversation instead of all of it. */
export async function exportRun(
	runId: string,
	format: string,
	path: string,
	turnIndex: number | null = null
): Promise<void> {
	return invoke('export_run', { request: { run_id: runId, format, path, turn_index: turnIndex } });
}
