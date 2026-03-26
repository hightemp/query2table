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
