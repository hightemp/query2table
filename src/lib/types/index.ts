export type RunStatus = 'pending' | 'schema_review' | 'running' | 'paused' | 'completed' | 'failed' | 'cancelled';

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
	data: Record<string, string>;
	status: string;
	confidence: number;
}

export interface RowSource {
	id: string;
	row_id: string;
	url: string;
	title: string;
	snippet: string;
}

export type LogLevel = 'DEBUG' | 'INFO' | 'WARN' | 'ERROR';

export interface LogEntry {
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
}
