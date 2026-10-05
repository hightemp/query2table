import { t } from '$lib/i18n';
import { get, writable } from 'svelte/store';
import type { HistoryFilter, HistoryRun } from '$lib/types';
import {
	deleteRuns,
	listHistory,
	pinRun,
	purgeRuns,
	renameRun,
	restoreRuns,
} from '$lib/api/tauri';
import { toast } from '$lib/stores/toasts';
import { errorText } from '$lib/utils/errors';

const PAGE_SIZE = 50;
/** A little longer than the Undo toast stays visible. */
const UNDO_MS = 8500;

export interface HistoryState {
	filter: HistoryFilter;
	runs: HistoryRun[];
	/** Matching runs of each type, ignoring the type filter. */
	counts: Record<string, number>;
	loading: boolean;
	hasMore: boolean;
	error: string;
	/** Where the list was scrolled, restored when coming back from a run. */
	scrollTop: number;
	/** The list has been loaded at least once. */
	loaded: boolean;
}

const DEFAULT_FILTER: HistoryFilter = { search: '', runType: 'all', status: 'any', sort: 'newest' };
const initial: HistoryState = {
	filter: DEFAULT_FILTER,
	runs: [],
	counts: {},
	loading: false,
	hasMore: false,
	error: '',
	scrollTop: 0,
	loaded: false,
};

export const history = writable<HistoryState>({ ...initial });
let generation = 0;

export function resetHistory() {
	generation++;
	history.set({ ...initial });
}

function request(filter: HistoryFilter, offset: number) {
	const search = filter.search.trim();
	return listHistory({
		search: search || null,
		run_type: filter.runType === 'all' ? null : filter.runType,
		status: filter.status === 'any' ? null : filter.status,
		sort: filter.sort,
		limit: PAGE_SIZE,
		offset,
	});
}

async function load(offset: number) {
	const current = ++generation;
	const { filter } = get(history);
	history.update((s) => ({ ...s, loading: true, error: '' }));
	try {
		const page = await request(filter, offset);
		if (current !== generation) return;
		history.update((s) => ({
			...s,
			runs: offset ? [...s.runs, ...page.runs.filter((run) => !s.runs.some((r) => r.id === run.id))] : page.runs,
			counts: page.counts,
			hasMore: page.runs.length === PAGE_SIZE,
			loading: false,
			loaded: true,
		}));
	} catch (error) {
		if (current !== generation) return;
		history.update((s) => ({ ...s, loading: false, loaded: true, error: errorText(error) }));
	}
}

/** Loads the first page for the current filter. */
export function loadHistory() {
	return load(0);
}

export function loadMoreHistory() {
	const state = get(history);
	if (state.loading || !state.hasMore) return Promise.resolve();
	return load(state.runs.length);
}

export function setHistoryFilter(patch: Partial<HistoryFilter>) {
	history.update((s) => ({ ...s, filter: { ...s.filter, ...patch }, scrollTop: 0 }));
	return load(0);
}

/** Removes runs at once; Undo restores them, otherwise they are purged after the toast. */
export async function deleteWithUndo(ids: string[]) {
	await deleteRuns(ids);
	const removed = get(history).runs.filter((run) => ids.includes(run.id));
	history.update((s) => ({ ...s, runs: s.runs.filter((run) => !ids.includes(run.id)) }));
	let undone = false;
	const timer = setTimeout(() => {
		if (!undone) void purgeRuns(ids).catch(() => {}); // purged on the next start otherwise
	}, UNDO_MS);
	toast(t('history.deleted', { count: ids.length }), 'info', {
		label: t('common.undo'),
		run: async () => {
			undone = true;
			clearTimeout(timer);
			try {
				await restoreRuns(ids);
				history.update((s) => ({
					...s,
					runs: [...s.runs, ...removed.filter((run) => !s.runs.some((r) => r.id === run.id))],
				}));
				await loadHistory();
			} catch (error) {
				toast(errorText(error), 'error');
			}
		},
	});
}

/** A blank title brings back the query. */
export async function renameHistoryRun(id: string, title: string) {
	const value = title.trim() || null;
	await renameRun(id, value);
	history.update((s) => ({ ...s, runs: s.runs.map((run) => (run.id === id ? { ...run, title: value } : run)) }));
}

export async function togglePin(run: HistoryRun) {
	await pinRun(run.id, !run.pinned_at);
	await loadHistory();
}

export function rememberHistoryScroll(scrollTop: number) {
	history.update((s) => ({ ...s, scrollTop }));
}
