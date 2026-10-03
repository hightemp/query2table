import { writable } from 'svelte/store';
import type { LogEntry, LogLevel } from '$lib/types';
import { persisted } from '$lib/utils/storage';

export const logs = writable<LogEntry[]>([]);
export const logFilter = writable<LogLevel | 'ALL'>('ALL');
export const logPanelOpen = writable(false);
/** Height chosen by dragging the panel edge; null keeps the default responsive height. */
export const logPanelHeight = persisted<number | null>('q2t-log-panel-height', null, (value) =>
	typeof value === 'number' && Number.isFinite(value) ? value : null
);

export function addLog(entry: LogEntry) {
	logs.update((items) => {
		const next = [...items, entry];
		// Keep last 1000 log entries
		if (next.length > 1000) {
			return next.slice(next.length - 1000);
		}
		return next;
	});
}

export function clearLogs() {
	logs.set([]);
}

/** Local wall-clock time of a log entry; unparseable timestamps are shown as received. */
export function logTime(timestamp: string): string {
	const date = new Date(timestamp);
	if (!timestamp || Number.isNaN(date.getTime())) return timestamp;
	return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false });
}

export function formatLogs(entries: LogEntry[]): string {
	return entries.map((entry) => `${entry.timestamp} ${entry.level} ${entry.message}`).join('\n');
}
