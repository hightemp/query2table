import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import {
	deleteWithUndo,
	history,
	loadHistory,
	loadMoreHistory,
	renameHistoryRun,
	resetHistory,
	setHistoryFilter,
	togglePin,
} from '$lib/stores/history';
import { toasts } from '$lib/stores/toasts';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const apiInvoke = vi.mocked(invoke);
const run = (id: string, patch: Record<string, unknown> = {}) => ({
	id,
	query: `Query ${id}`,
	title: null,
	status: 'completed',
	run_type: 'table',
	stats: null,
	error: null,
	created_at: 1,
	completed_at: null,
	pinned_at: null,
	dismissed_notices: null,
	turn_count: 0,
	...patch,
});
const calls = (command: string) => apiInvoke.mock.calls.filter(([name]) => name === command);

describe('history list', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		resetHistory();
		toasts.set([]);
		apiInvoke.mockReset().mockImplementation(async (command, args: any) => {
			if (command === 'list_history') {
				const { offset, limit } = args.filter;
				const all = Array.from({ length: 60 }, (_, i) => run(`r${i}`));
				return { runs: all.slice(offset, offset + limit), counts: { table: 60 } } as never;
			}
			return undefined as never;
		});
	});
	afterEach(() => vi.useRealTimers());

	it('loads pages of 50 with the filter and counts', async () => {
		await loadHistory();
		expect(calls('list_history')[0][1]).toEqual({
			filter: { search: null, run_type: null, status: null, sort: 'newest', limit: 50, offset: 0 },
		});
		expect(get(history)).toMatchObject({ hasMore: true, counts: { table: 60 }, loading: false });
		expect(get(history).runs).toHaveLength(50);
		await loadMoreHistory();
		expect(calls('list_history')[1][1]).toMatchObject({ filter: { offset: 50 } });
		expect(get(history).runs).toHaveLength(60);
		expect(get(history).hasMore).toBe(false);
	});

	it('reloads from the start when the filter changes and ignores stale pages', async () => {
		let release!: () => void;
		apiInvoke.mockImplementationOnce(
			() => new Promise((resolve) => (release = () => resolve({ runs: [run('stale')], counts: {} } as never)))
		);
		const stale = loadHistory();
		await setHistoryFilter({ search: '  heat ', runType: 'research', status: 'failed', sort: 'cost' });
		release();
		await stale;
		expect(calls('list_history').at(-1)![1]).toEqual({
			filter: { search: 'heat', run_type: 'research', status: 'failed', sort: 'cost', limit: 50, offset: 0 },
		});
		expect(get(history).runs.map((r) => r.id)).not.toContain('stale');
		expect(get(history).filter).toMatchObject({ search: '  heat ', runType: 'research' });
	});

	it('deletes at once, offers undo, and purges when the undo expires', async () => {
		await loadHistory();
		await deleteWithUndo(['r0', 'r1']);
		expect(calls('delete_runs')[0][1]).toEqual({ runIds: ['r0', 'r1'] });
		expect(get(history).runs.map((r) => r.id)).not.toContain('r0');
		const [notice] = get(toasts);
		expect(notice.message).toBe('2 runs deleted');
		expect(notice.action?.label).toBe('Undo');
		await vi.advanceTimersByTimeAsync(9000);
		expect(calls('purge_runs')[0][1]).toEqual({ runIds: ['r0', 'r1'] });
	});

	it('restores deleted runs on undo and does not purge them', async () => {
		await loadHistory();
		await deleteWithUndo(['r2']);
		await get(toasts)[0].action!.run();
		expect(calls('restore_runs')[0][1]).toEqual({ runIds: ['r2'] });
		expect(get(history).runs.map((r) => r.id)).toContain('r2');
		await vi.advanceTimersByTimeAsync(9000);
		expect(calls('purge_runs')).toHaveLength(0);
	});

	it('keeps runs in place when deleting fails', async () => {
		await loadHistory();
		apiInvoke.mockImplementationOnce(async () => {
			throw new Error('A running run cannot be deleted. Cancel it first.');
		});
		await expect(deleteWithUndo(['r3'])).rejects.toThrow('running run');
		expect(get(history).runs.map((r) => r.id)).toContain('r3');
	});

	it('renames and pins runs', async () => {
		await loadHistory();
		await renameHistoryRun('r4', '  Robots ');
		expect(calls('rename_run')[0][1]).toEqual({ runId: 'r4', title: 'Robots' });
		expect(get(history).runs.find((r) => r.id === 'r4')?.title).toBe('Robots');
		await renameHistoryRun('r4', ' ');
		expect(calls('rename_run')[1][1]).toEqual({ runId: 'r4', title: null });
		await togglePin(get(history).runs.find((r) => r.id === 'r5')!);
		expect(calls('pin_run')[0][1]).toEqual({ runId: 'r5', pinned: true });
		// Pinning moves the run, so the list is loaded again.
		expect(calls('list_history').length).toBeGreaterThan(1);
	});
});
