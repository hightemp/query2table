import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
	StartRunResponse,
	RunInfo,
	RunLogEntry,
	SchemaColumn,
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
	stopConditions?: StopConditions
): Promise<StartRunResponse> {
	return invoke('start_run', {
		query,
		runType: runType ?? null,
		stopConditions: stopConditions ?? null,
	});
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

export async function getRunIssues(runId: string): Promise<LlmIssueEvent[]> {
	return invoke('get_run_issues', { runId });
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
	stopConditions: StopConditions
): Promise<void> {
	return invoke('ask_follow_up', { runId, question, stopConditions });
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
