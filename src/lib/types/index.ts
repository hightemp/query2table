export type RunStatus =
	'pending' | 'schema_review' | 'running' | 'paused' | 'completed' | 'failed' | 'cancelled';

export interface Run {
	id: string;
	query: string;
	status: RunStatus;
	created_at: string;
	updated_at: string;
	row_count: number;
}

export interface SchemaColumn {
	name: string;
	type: string;
	description: string;
	required: boolean;
}

export interface EntityRow {
	id: string;
	run_id: string;
	data: Record<string, unknown>;
	status: string;
	confidence: number;
}

export interface RowSource {
	id: string;
	row_id: string;
	url: string;
	title: string | null;
	snippet: string | null;
}

export type LogLevel = 'DEBUG' | 'INFO' | 'WARN' | 'ERROR';

export interface LogEntry {
	run_id?: string;
	role?: string;
	timestamp: string;
	level: LogLevel;
	message: string;
	target?: string;
}

export interface SettingGroup {
	label: string;
	description: string;
	settings: SettingDef[];
}

export interface SettingDef {
	provider?: 'openrouter' | 'ollama' | 'ollama_cloud' | 'openai_compatible';
	key: string;
	label: string;
	description: string;
	type: 'text' | 'password' | 'number' | 'select' | 'toggle';
	options?: { label: string; value: string }[];
	placeholder?: string;
}

// --- Run API response types ---

export interface StartRunResponse {
	run_id: string;
}

export interface RunInfo {
	id: string;
	query: string;
	status: string;
	run_type: string;
	stats: string | null;
	error: string | null;
	created_at: number;
	/** JSON map of dismissed notice kinds to counts. */
	dismissed_notices?: string | null;
	/** Name given in History; the query is shown when missing. */
	title?: string | null;
	pinned_at?: number | null;
	/** JSON: mode and the stop conditions the run started with. */
	config?: string;
}

/** A run in the History list. */
export interface HistoryRun {
	id: string;
	query: string;
	title: string | null;
	status: string;
	run_type: string;
	stats: string | null;
	error: string | null;
	created_at: number;
	completed_at: number | null;
	pinned_at: number | null;
	dismissed_notices: string | null;
	/** Questions asked in a research conversation. */
	turn_count: number;
	/** Distinct files attached to the run. */
	attachment_count?: number;
}

export type HistorySort = 'newest' | 'oldest' | 'results' | 'cost';

export interface HistoryFilter {
	search: string;
	/** all, table, images, links or research */
	runType: string;
	/** any, completed, failed, cancelled or active */
	status: string;
	sort: HistorySort;
}

export interface RunLogEntry {
	id: string;
	level: string;
	role: string | null;
	message: string;
	created_at: number;
}

// --- Event payload types ---

export interface StatusChangedEvent {
	run_id: string;
	status: string;
}

export interface RowAddedEvent {
	run_id: string;
	row_id: string;
	data: Record<string, unknown>;
	confidence: number;
}

export interface RowsReplacedEvent {
	run_id: string;
	rows: {
		id: string;
		data: Record<string, unknown>;
		confidence: number;
		/** Missing from events published by older versions. */
		source_count?: number;
	}[];
}

export interface ProgressStats {
	rows_found: number;
	pages_fetched: number;
	pages_total: number;
	queries_executed: number;
	queries_total: number;
	elapsed_secs: number;
	spent_usd: number;
}

export interface ProgressEvent {
	run_id: string;
	stats: ProgressStats;
}

export interface PriceQuote {
	input_per_million: number;
	output_per_million: number;
	cached_input_per_million?: number | null;
	cache_write_per_million?: number | null;
	per_request: number;
	source: string;
	credit_based: boolean;
}

export interface CostLine {
	provider: string;
	model: string;
	requested_model?: string;
	calls: number;
	reported_usd: number;
	estimated_usd: number;
	unpriced_calls: number;
	prompt_tokens: number;
	completion_tokens: number;
	pricing: PriceQuote | null;
}

export interface Accounting {
	spent_usd: number;
	max_budget_usd: number;
	reported_usd: number;
	estimated_usd: number;
	reported_calls: number;
	estimated_calls: number;
	unpriced_calls: number;
	pending_calls: number;
	missing_usage_calls: number;
	prompt_tokens: number;
	completion_tokens: number;
	reasoning_tokens: number;
	cached_prompt_tokens: number;
	llm_calls: number;
	search_calls: number;
	fetch_calls: number;
	breakdown: CostLine[];
}

export interface AccountingEvent {
	run_id: string;
	accounting: Accounting;
}

export interface LogEntryEvent {
	run_id: string;
	level: string;
	role: string;
	message: string;
}

export interface SchemaProposedEvent {
	run_id: string;
	columns: SchemaColumn[];
}

export interface RunErrorEvent {
	run_id: string;
	error: string;
}

export type LlmIssueCode =
	| 'budget_limit'
	| 'output_limit'
	| 'context_limit'
	| 'rate_limit'
	| 'quota'
	| 'auth'
	| 'access_denied'
	| 'timeout'
	| 'connection'
	| 'invalid_response'
	| 'empty_response'
	| 'not_configured'
	| 'unsupported_setting'
	| 'model_not_found'
	| 'provider_error';

export interface LlmIssueEvent {
	run_id: string;
	code: LlmIssueCode;
	provider: string;
	model: string;
	stage: string | null;
	message: string;
	max_tokens: number;
	prompt_tokens: number | null;
	completion_tokens: number | null;
	reasoning_tokens: number | null;
	retry_after_ms: number | null;
	attempt: number;
	max_attempts: number;
	will_retry: boolean;
	/** What the failure meant for the result; missing in runs recorded before outcomes. */
	outcome?: LlmIssueOutcome | null;
	/** Shared by the attempts of one request. */
	call_id?: string | null;
	/** Research conversation turn the request belonged to. */
	turn_index?: number | null;
	/** Unix time in milliseconds. */
	at?: number | null;
}

/**
 * retrying: another attempt follows. recovered: a later attempt succeeded.
 * continued: the step worked around it with no loss. fallback: a simpler method was used.
 * skipped: part of the result was dropped. stopped: the run ended at this step.
 */
export type LlmIssueOutcome = 'retrying' | 'recovered' | 'continued' | 'fallback' | 'skipped' | 'stopped';

// --- Image Search types ---

export interface ImageResult {
	id: string;
	image_url: string;
	thumbnail_url: string;
	title: string;
	source_url: string;
	width: number | null;
	height: number | null;
	relevance_score: number | null;
}

export interface ImageAddedEvent {
	run_id: string;
	image_id: string;
	image_url: string;
	thumbnail_url: string;
	title: string;
	source_url: string;
	width: number | null;
	height: number | null;
	relevance_score: number | null;
}

// --- Link Search types ---

export interface LinkResult {
	id: string;
	url: string;
	title: string;
	/** What the page contains. */
	description: string;
	/** Why the page matches the query; empty for runs saved before it was recorded. */
	reason?: string;
	relevance_score: number | null;
	/** Scored below the run's relevance threshold: shown collapsed and not exported. */
	low_relevance?: boolean;
	hidden?: boolean;
	/** Unix seconds when the link was first opened. */
	visited_at?: number | null;
	/** Unix seconds when the link was found. */
	created_at?: number;
}

export interface LinkAddedEvent {
	run_id: string;
	link_id: string;
	url: string;
	title: string;
	description: string;
	relevance_score: number | null;
	reason?: string;
	low_relevance?: boolean;
}

// --- Research (agentic) types ---

export interface ResearchStep {
	id: string;
	/** Turn of the conversation the step belongs to; 0 for the first question. */
	turn_index?: number;
	step_index: number;
	step_type: string;
	content: string;
	url: string | null;
}

export interface ResearchTurnInfo {
	turn_index: number;
	question: string;
	answer_markdown: string | null;
	follow_ups: string[];
	status: string;
	limits: { target_row_count?: number; max_budget_usd?: number; max_duration_seconds?: number };
	accounting: Accounting | null;
}

export interface ResearchResult {
	answer_markdown: string | null;
	steps: ResearchStep[];
	/** Missing in data from versions before conversations. */
	turns?: ResearchTurnInfo[];
}

export interface ResearchStepEvent {
	run_id: string;
	step_id: string;
	turn_index?: number;
	step_index: number;
	step_type: string;
	content: string;
	url: string | null;
}

export interface ResearchAnswerEvent {
	run_id: string;
	turn_index?: number;
	follow_ups?: string[];
	markdown: string;
}

// --- Attachments ---

export type AttachmentKind = 'document' | 'spreadsheet' | 'text' | 'image';

export interface AttachmentInfo {
	id: string;
	file_name: string;
	kind: AttachmentKind;
	mime: string;
	size: number;
	/** `needs_vision`: some pages are scans that need a model that sees images. */
	status: 'ready' | 'needs_vision';
	page_count: number | null;
	sheet_count: number | null;
	char_count: number;
	scanned_pages: number;
	/** Small preview of images, as a data URL. */
	thumbnail: string | null;
	created_at: number;
}

export interface AttachError {
	file_name: string;
	code: string;
	message: string;
}

export interface AttachResult {
	attachment: AttachmentInfo | null;
	error: AttachError | null;
}

export interface RunAttachment {
	turn_index: number;
	attachment: AttachmentInfo;
}

export interface AttachmentFragment {
	attachment_id: string;
	file_name: string;
	locator: { page?: number; sheet?: string; rows?: [number, number]; section?: string };
	text: string;
}

// --- Search queries and stages of a run ---

export interface SearchQueryInfo {
	id: string;
	query_text: string;
	/** ISO code, or empty when the mode does not plan languages. */
	language: string;
	/** pending, running, completed, failed or skipped */
	status: string;
	/** New results the query added. */
	result_count: number;
	error: string | null;
}
