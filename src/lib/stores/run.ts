import { writable, get } from 'svelte/store';
import type {
	SchemaColumn,
	ProgressStats,
	RowAddedEvent,
} from '$lib/types';
import {
	startRun as apiStartRun,
	cancelRun as apiCancelRun,
	pauseRun as apiPauseRun,
	resumeRun as apiResumeRun,
	confirmSchema as apiConfirmSchema,
	onStatusChanged,
	onRowAdded,
	onProgressUpdate,
	onSchemaProposed,
	onRunError,
	onRunLogEntry,
} from '$lib/api/tauri';
import { addLog } from '$lib/stores/logs';

export interface RunRow {
	id: string;
	data: Record<string, unknown>;
	confidence: number;
}

export interface RunState {
	runId: string | null;
	query: string;
	status: string;
	schema: SchemaColumn[];
	rows: RunRow[];
	progress: ProgressStats | null;
	error: string | null;
}

const initialState: RunState = {
	runId: null,
	query: '',
	status: 'idle',
	schema: [],
	rows: [],
	progress: null,
	error: null,
};

export const runState = writable<RunState>({ ...initialState });

// Track event unsubscribers
let unlisteners: (() => void)[] = [];

async function subscribeEvents() {
	unsubscribeEvents();

	const unsubs = await Promise.all([
		onStatusChanged((e) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				return { ...s, status: e.status };
			});
		}),
		onRowAdded((e: RowAddedEvent) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				const row: RunRow = {
					id: e.row_id,
					data: e.data,
					confidence: e.confidence,
				};
				return { ...s, rows: [...s.rows, row] };
			});
		}),
		onProgressUpdate((e) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				return { ...s, progress: e.stats };
			});
		}),
		onSchemaProposed((e) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				return { ...s, schema: e.columns, status: 'schema_review' };
			});
		}),
		onRunError((e) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				return { ...s, error: e.error, status: 'failed' };
			});
		}),
		onRunLogEntry((e) => {
			const current = get(runState);
			if (current.runId !== e.run_id) return;
			addLog({
				timestamp: new Date().toISOString(),
				level: e.level as 'DEBUG' | 'INFO' | 'WARN' | 'ERROR',
				message: `[${e.role}] ${e.message}`,
			});
		}),
	]);

	unlisteners = unsubs;
}

function unsubscribeEvents() {
	for (const fn of unlisteners) fn();
	unlisteners = [];
}

export async function startNewRun(query: string, stopConditions?: import('$lib/api/tauri').StopConditions) {
	runState.set({
		...initialState,
		query,
		status: 'pending',
	});

	await subscribeEvents();

	const resp = await apiStartRun(query, stopConditions);
	runState.update((s) => ({ ...s, runId: resp.run_id }));
}

export async function cancelCurrentRun() {
	const { runId } = get(runState);
	if (!runId) return;
	await apiCancelRun(runId);
}

export async function pauseCurrentRun() {
	const { runId } = get(runState);
	if (!runId) return;
	await apiPauseRun(runId);
}

export async function resumeCurrentRun() {
	const { runId } = get(runState);
	if (!runId) return;
	await apiResumeRun(runId);
}

export async function confirmCurrentSchema(columns: SchemaColumn[]) {
	const { runId } = get(runState);
	if (!runId) return;
	await apiConfirmSchema(runId, columns);
}

export function resetRun() {
	unsubscribeEvents();
	runState.set({ ...initialState });
}
