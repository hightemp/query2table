import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { logs, logFilter, addLog, clearLogs } from '$lib/stores/logs';
import type { LogEntry } from '$lib/types';

describe('logs store', () => {
	it('starts empty', () => {
		clearLogs();
		expect(get(logs)).toEqual([]);
	});

	it('adds log entries', () => {
		clearLogs();
		const entry: LogEntry = {
			timestamp: '2024-01-01T00:00:00Z',
			level: 'INFO',
			message: 'test message',
		};
		addLog(entry);
		const items = get(logs);
		expect(items).toHaveLength(1);
		expect(items[0].message).toBe('test message');
	});

	it('clears logs', () => {
		addLog({ timestamp: '', level: 'INFO', message: 'a' });
		clearLogs();
		expect(get(logs)).toHaveLength(0);
	});

	it('caps at 1000 entries', () => {
		clearLogs();
		for (let i = 0; i < 1010; i++) {
			addLog({ timestamp: '', level: 'DEBUG', message: `msg-${i}` });
		}
		expect(get(logs).length).toBeLessThanOrEqual(1000);
	});

	it('filter defaults to ALL', () => {
		expect(get(logFilter)).toBe('ALL');
	});
});
