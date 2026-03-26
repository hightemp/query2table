import { writable } from 'svelte/store';
import type { LogEntry, LogLevel } from '$lib/types';

export const logs = writable<LogEntry[]>([]);
export const logFilter = writable<LogLevel | 'ALL'>('ALL');
export const logPanelOpen = writable(false);

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
