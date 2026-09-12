import { writable, get } from 'svelte/store';
import type {
	SchemaColumn,
	ProgressStats,
	RowAddedEvent,
	ImageResult,
	ImageAddedEvent,
	LinkResult,
	LinkAddedEvent,
	ResearchStep,
	ResearchStepEvent,
	ResearchAnswerEvent,
	LlmIssueEvent,
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
	onImageAdded,
	onLinkAdded,
	onResearchStep,
	onResearchAnswer,
	onLlmIssue,
} from '$lib/api/tauri';
import { addLog } from '$lib/stores/logs';

export interface RunRow {
	id: string;
	data: Record<string, unknown>;
	confidence: number;
}

export type RunControl = 'pause' | 'resume' | 'cancel' | 'confirm_schema';

export interface RunState {
	runId: string | null;
	query: string;
	runType: string;
	status: string;
	schema: SchemaColumn[];
	rows: RunRow[];
	imageResults: ImageResult[];
	linkResults: LinkResult[];
	researchSteps: ResearchStep[];
	researchAnswer: string | null;
	progress: ProgressStats | null;
	error: string | null;
	controlPending: RunControl | null;
	controlError: string | null;
	pausedFrom: string | null;
	llmIssues: LlmIssueEvent[];
}

const initialState: RunState = {
	runId: null,
	query: '',
	runType: 'table',
	status: 'idle',
	schema: [],
	rows: [],
	imageResults: [],
	linkResults: [],
	researchSteps: [],
	researchAnswer: null,
	progress: null,
	error: null,
	controlPending: null,
	controlError: null,
	pausedFrom: null,
	llmIssues: [],
};

export const runState = writable<RunState>({ ...initialState });

// Track event unsubscribers
let unlisteners: (() => void)[] = [];
let generation = 0;
let pendingStart: Promise<void> | null = null;
let controlRequest = 0;

async function subscribeEvents(currentGeneration: number, earlyEvents: (() => void)[]) {
	// The spawned backend can publish before start_run returns its run ID.
	// Buffer these events and replay after binding the returned ID.
	function subscribe<T extends { run_id: string }>(
		listen: (callback: (event: T) => void) => Promise<() => void>,
		callback: (event: T) => void,
	) {
		return listen((event) => {
			if (currentGeneration !== generation) return;
			const deliver = () => {
				if (currentGeneration === generation && get(runState).runId === event.run_id) callback(event);
			};
			if (get(runState).runId === null) earlyEvents.push(deliver);
			else deliver();
		});
	}

	const subscriptions = await Promise.allSettled([
		subscribe(onStatusChanged, (e) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				const acknowledged = ['completed', 'failed', 'cancelled'].includes(e.status)
					|| (s.controlPending === 'pause' && e.status === 'paused')
					|| (s.controlPending === 'resume' && e.status !== 'paused')
					|| (s.controlPending === 'confirm_schema' && e.status === 'running');
				return {
					...s,
					status: e.status,
					pausedFrom: e.status === 'paused' ? (s.pausedFrom ?? s.status) : null,
					controlPending: acknowledged ? null : s.controlPending,
				};
			});
		}),
		subscribe(onRowAdded, (e: RowAddedEvent) => {
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
		subscribe(onProgressUpdate, (e) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				return { ...s, progress: e.stats };
			});
		}),
		subscribe(onSchemaProposed, (e) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				return { ...s, schema: e.columns, status: 'schema_review' };
			});
		}),
		subscribe(onRunError, (e) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				return { ...s, error: e.error, status: 'failed', controlPending: null };
			});
		}),
		subscribe(onLlmIssue, (issue) => {
			runState.update((s) => ({ ...s, llmIssues: [...s.llmIssues, issue].slice(-100) }));
		}),
		subscribe(onRunLogEntry, (e) => {
			const current = get(runState);
			if (current.runId !== e.run_id) return;
			addLog({
				timestamp: new Date().toISOString(),
				level: e.level as 'DEBUG' | 'INFO' | 'WARN' | 'ERROR',
				message: `[${e.role}] ${e.message}`,
			});
		}),
		subscribe(onImageAdded, (e: ImageAddedEvent) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				const img: ImageResult = {
					id: e.image_id,
					image_url: e.image_url,
					thumbnail_url: e.thumbnail_url,
					title: e.title,
					source_url: e.source_url,
					width: e.width,
					height: e.height,
					relevance_score: e.relevance_score,
				};
				return { ...s, imageResults: [...s.imageResults, img] };
			});
		}),
		subscribe(onLinkAdded, (e: LinkAddedEvent) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				const link: LinkResult = {
					id: e.link_id,
					url: e.url,
					title: e.title,
					description: e.description,
					relevance_score: e.relevance_score,
				};
				return { ...s, linkResults: [...s.linkResults, link] };
			});
		}),
		subscribe(onResearchStep, (e: ResearchStepEvent) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				const step: ResearchStep = {
					id: e.step_id,
					step_index: e.step_index,
					step_type: e.step_type,
					content: e.content,
					url: e.url,
				};
				return { ...s, researchSteps: [...s.researchSteps, step] };
			});
		}),
		subscribe(onResearchAnswer, (e: ResearchAnswerEvent) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				return { ...s, researchAnswer: e.markdown };
			});
		}),
	]);

	const unsubs = subscriptions.flatMap((result) => result.status === 'fulfilled' ? [result.value] : []);
	const failed = subscriptions.find((result) => result.status === 'rejected');
	if (currentGeneration !== generation || failed) {
		for (const unsubscribe of unsubs) unsubscribe();
		if (failed?.status === 'rejected') throw failed.reason;
		return;
	}
	unlisteners = unsubs;
}

function unsubscribeEvents() {
	for (const fn of unlisteners) fn();
	unlisteners = [];
}

export async function startNewRun(query: string, runType: string = 'table', stopConditions?: import('$lib/api/tauri').StopConditions) {
	const currentGeneration = ++generation;
	unsubscribeEvents();
	runState.set({
		...initialState,
		query,
		runType,
		status: 'pending',
	});

	const earlyEvents: (() => void)[] = [];
	const start = (async () => {
		try {
			await subscribeEvents(currentGeneration, earlyEvents);
			if (currentGeneration !== generation) return;
			const resp = await apiStartRun(query, runType, stopConditions);
			if (currentGeneration !== generation) return;
			runState.update((s) => ({ ...s, runId: resp.run_id }));
			for (const deliver of earlyEvents) deliver();
		} catch (error) {
			if (currentGeneration === generation) {
				unsubscribeEvents();
				runState.update((s) => ({ ...s, status: 'failed', error: String(error), controlPending: null }));
				addLog({ timestamp: new Date().toISOString(), level: 'ERROR', message: `[run] Start failed: ${String(error)}` });
			}
			throw error;
		}
	})();
	pendingStart = start;
	try {
		await start;
	} finally {
		if (pendingStart === start) pendingStart = null;
	}
}

async function requestControl(action: RunControl, send: (runId: string) => Promise<void>) {
	const state = get(runState);
	if (['idle', 'completed', 'failed', 'cancelled'].includes(state.status)) return;
	if (state.controlPending && (action !== 'cancel' || state.controlPending === 'cancel')) return;
	const currentGeneration = generation;
	const request = ++controlRequest;
	runState.update((s) => ({ ...s, controlPending: action, controlError: null }));
	try {
		// A click during startup must not silently disappear while the ID is pending.
		if (pendingStart) await pendingStart;
		if (currentGeneration !== generation || request !== controlRequest) return;
		const { runId, status } = get(runState);
		if (!runId || ['completed', 'failed', 'cancelled'].includes(status)) return;
		addLog({ timestamp: new Date().toISOString(), level: 'INFO', message: `[run] Requesting ${action}` });
		await send(runId);
		// Keep the pending indicator until the backend publishes the new status.
	} catch (error) {
		if (currentGeneration !== generation || request !== controlRequest || get(runState).status === 'failed') return;
		const message = `Could not ${action.replace('_', ' ')}: ${String(error)}`;
		runState.update((s) => ({ ...s, controlPending: null, controlError: message }));
		addLog({ timestamp: new Date().toISOString(), level: 'ERROR', message: `[run] ${message}` });
	}
}

export function cancelCurrentRun() {
	return requestControl('cancel', apiCancelRun);
}

export function pauseCurrentRun() {
	return requestControl('pause', apiPauseRun);
}

export function resumeCurrentRun() {
	return requestControl('resume', apiResumeRun);
}

export function confirmCurrentSchema(columns: SchemaColumn[]) {
	return requestControl('confirm_schema', (runId) => apiConfirmSchema(runId, columns));
}

export function resetRun() {
	generation++;
	pendingStart = null;
	unsubscribeEvents();
	runState.set({ ...initialState });
}
