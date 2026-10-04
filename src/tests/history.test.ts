import { describe, expect, it } from 'vitest';
import type { HistoryRun } from '$lib/types';
import {
	dateGroup,
	formatDuration,
	groupRuns,
	relativeTime,
	runLimits,
	runSummary,
} from '$lib/utils/history';

// Saturday 2026-10-03 15:00 local time.
const now = new Date(2026, 9, 3, 15, 0, 0).getTime();
const at = (days: number, hours = 15, minutes = 0) =>
	Math.floor(new Date(2026, 9, 3 - days, hours, minutes).getTime() / 1000);
const run = (patch: Partial<HistoryRun>): HistoryRun => ({
	id: 'r',
	query: 'Query',
	title: null,
	status: 'completed',
	run_type: 'table',
	stats: null,
	error: null,
	created_at: at(0),
	completed_at: null,
	pinned_at: null,
	dismissed_notices: null,
	turn_count: 0,
	...patch,
});
const accounting = (patch: Record<string, unknown>) => ({
	spent_usd: 0,
	unpriced_calls: 0,
	breakdown: [
		{ provider: 'brave', model: 'web search', calls: 9, unpriced_calls: 0 },
		{ provider: 'ollama_cloud', model: 'deepseek-v4.1-flash', calls: 3, unpriced_calls: 0 },
	],
	...patch,
});

describe('history rows', () => {
	it('summarise results, duration, cost and model from saved stats', () => {
		const table = run({
			stats: JSON.stringify({ rows_found: 48, elapsed_secs: 109, accounting: accounting({ spent_usd: 0.031 }) }),
		});
		expect(runSummary(table)).toEqual({
			results: '48 rows',
			duration: '1 m 49 s',
			cost: '$0.03',
			model: 'deepseek-v4.1-flash',
		});
		expect(runSummary(run({ run_type: 'links', stats: JSON.stringify({ link_count: 1 }) })).results).toBe('1 link');
		expect(runSummary(run({ run_type: 'images', stats: JSON.stringify({ image_count: 50 }) })).results).toBe('50 images');
		expect(runSummary(run({ run_type: 'research', turn_count: 1, stats: JSON.stringify({ steps: 16 }) })).results).toBe('16 steps');
		expect(runSummary(run({ run_type: 'research', turn_count: 2, stats: JSON.stringify({ steps: 0 }) })).results).toBe('2 questions');
		// Calls without a price make the cost unknown rather than free.
		expect(runSummary(run({ stats: JSON.stringify({ accounting: accounting({ unpriced_calls: 3 }) }) })).cost).toBe('cost unknown');
		expect(runSummary(run({ stats: JSON.stringify({ spent_usd: 0.5 }) })).cost).toBe('$0.50');
		expect(runSummary(run({ stats: null }))).toEqual({ results: null, duration: null, cost: null, model: null });
		expect(runSummary(run({ stats: 'not json' })).results).toBeNull();
	});

	it('formats durations compactly', () => {
		expect(formatDuration(9)).toBe('9 s');
		expect(formatDuration(60)).toBe('1 m');
		expect(formatDuration(3725)).toBe('1 h 2 m');
	});

	it('uses relative time today and clock or date before', () => {
		expect(relativeTime(at(0, 14, 59) + 30, now)).toBe('just now');
		expect(relativeTime(at(0, 14, 35), now)).toBe('25 min ago');
		expect(relativeTime(at(0, 12), now)).toBe('3 h ago');
		expect(relativeTime(at(1, 9, 5), now)).toMatch(/9:05|09:05/);
		expect(relativeTime(at(40), now)).toMatch(/2026|26/);
	});

	it('groups runs by day and keeps pinned runs on top', () => {
		expect(dateGroup(at(0, 1), now)).toBe('Today');
		expect(dateGroup(at(1, 23), now)).toBe('Yesterday');
		expect(dateGroup(at(3), now)).toBe('This week');
		expect(dateGroup(at(40), now)).toBe('August 2026');
		const runs = [
			run({ id: 'p', created_at: at(40), pinned_at: 1 }),
			run({ id: 'a', created_at: at(0) }),
			run({ id: 'b', created_at: at(0, 9) }),
			run({ id: 'c', created_at: at(1) }),
		];
		expect(groupRuns(runs, now, 'newest').map((g) => [g.label, g.runs.map((r) => r.id)])).toEqual([
			['Pinned', ['p']],
			['Today', ['a', 'b']],
			['Yesterday', ['c']],
		]);
		// Other orders are not by date, so day headings would mislead.
		expect(groupRuns(runs, now, 'cost').map((g) => g.label)).toEqual(['Pinned', null]);
	});

	it('reads the limits a run was started with', () => {
		expect(
			runLimits('{"mode":"table","stop":{"target_row_count":7,"max_budget_usd":2,"max_duration_seconds":300}}')
		).toEqual({ target_row_count: 7, max_budget_usd: 2, max_duration_seconds: 300 });
		expect(runLimits('{"mode":"links"}')).toBeNull();
		expect(runLimits('oops')).toBeNull();
	});
});
