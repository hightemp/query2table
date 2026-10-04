import { writable, get } from 'svelte/store';
import type {
	SchemaColumn,
	Accounting,
	ProgressStats,
	RowAddedEvent,
	ImageResult,
	ImageAddedEvent,
	LinkResult,
	LinkAddedEvent,
	ResearchStep,
	ResearchStepEvent,
	ResearchAnswerEvent,
	ResearchResult,
	LlmIssueEvent,
	LogEntryEvent,
} from '$lib/types';
import {
	startRun as apiStartRun,
	cancelRun as apiCancelRun,
	pauseRun as apiPauseRun,
	resumeRun as apiResumeRun,
	confirmSchema as apiConfirmSchema,
	onStatusChanged,
	onRowAdded,
	onRowsReplaced,
	onAccounting,
	onProgressUpdate,
	onSchemaProposed,
	onRunError,
	onRunLogEntry,
	onImageAdded,
	onLinkAdded,
	onResearchStep,
	onResearchAnswer,
	onLlmIssue,
	askFollowUp as apiAskFollowUp,
	getRun,
	getResearchResult,
	getRunIssues,
	dismissRunNotices,
	type StopConditions,
} from '$lib/api/tauri';
import { addLog } from '$lib/stores/logs';

export interface RunRow {
	id: string;
	data: Record<string, unknown>;
	confidence: number;
	/** Saved sources, when known. */
	sources?: number;
}

export type RunControl = 'pause' | 'resume' | 'cancel' | 'confirm_schema';

export interface ResearchTurnState {
	index: number;
	question: string;
	answer: string | null;
	followUps: string[];
	/** running, completed, cancelled or failed */
	status: string;
	steps: ResearchStep[];
	/** Final usage and cost of the turn, once it has ended. */
	accounting: Accounting | null;
}

const TERMINAL = ['completed', 'failed', 'cancelled'];

function newTurn(index: number, question: string): ResearchTurnState {
	return { index, question, answer: null, followUps: [], status: 'running', steps: [], accounting: null };
}

/** Applies a change to one turn, adding turns up to `index` if events arrive first. */
function updateTurn(
	turns: ResearchTurnState[],
	index: number,
	change: (turn: ResearchTurnState) => ResearchTurnState
): ResearchTurnState[] {
	const next = [...turns];
	while (next.length <= index) next.push(newTurn(next.length, ''));
	next[index] = change(next[index]);
	return next;
}

export interface RunState {
	runId: string | null;
	query: string;
	runType: string;
	status: string;
	schema: SchemaColumn[];
	rows: RunRow[];
	imageResults: ImageResult[];
	linkResults: LinkResult[];
	/** Turns of a research conversation, in order. */
	researchTurns: ResearchTurnState[];
	/** All steps of the conversation. */
	researchSteps: ResearchStep[];
	/** The latest answer. */
	researchAnswer: string | null;
	progress: ProgressStats | null;
	error: string | null;
	controlPending: RunControl | null;
	controlError: string | null;
	pausedFrom: string | null;
	llmIssues: LlmIssueEvent[];
	/** Notice counts the reader marked as read; see `isDismissed`. */
	noticesDismissed: Record<string, number> | null;
	activity: LogEntryEvent[];
	accounting: Accounting | null;
	/** Stop conditions requested for this run, when known. */
	limits: import('$lib/api/tauri').StopConditions | null;
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
	researchTurns: [],
	researchSteps: [],
	researchAnswer: null,
	progress: null,
	error: null,
	controlPending: null,
	controlError: null,
	pausedFrom: null,
	llmIssues: [],
	noticesDismissed: null,
	activity: [],
	accounting: null,
	limits: null,
};

export const runState = writable<RunState>({ ...initialState });

/** A query to put in the form when the query page opens (Edit and run from History). */
export interface QueryDraft {
	query: string;
	runType: string;
	limits: Required<import('$lib/api/tauri').StopConditions> | null;
}
export const queryDraft = writable<QueryDraft | null>(null);

const BUSY = ['pending', 'running', 'paused', 'schema_review'];
/** A run is being worked on in the query page. */
export function runInProgress(): boolean {
	return BUSY.includes(get(runState).status);
}

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
		callback: (event: T) => void
	) {
		return listen((event) => {
			if (currentGeneration !== generation) return;
			const deliver = () => {
				if (currentGeneration === generation && get(runState).runId === event.run_id)
					callback(event);
			};
			if (get(runState).runId === null) earlyEvents.push(deliver);
			else deliver();
		});
	}

	const subscriptions = await Promise.allSettled([
		subscribe(onStatusChanged, (e) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				const acknowledged =
					['completed', 'failed', 'cancelled'].includes(e.status) ||
					(s.controlPending === 'pause' && e.status === 'paused') ||
					(s.controlPending === 'resume' && e.status !== 'paused') ||
					(s.controlPending === 'confirm_schema' && e.status === 'running');
				// A research turn ends with the run; earlier turns keep their own status.
				const last = s.researchTurns.length - 1;
				const researchTurns =
					last >= 0 && TERMINAL.includes(e.status) && s.researchTurns[last].status === 'running'
						? updateTurn(s.researchTurns, last, (turn) => ({
								...turn,
								status: e.status,
								accounting: s.accounting,
							}))
						: s.researchTurns;
				return {
					...s,
					status: e.status,
					researchTurns,
					pausedFrom: e.status === 'paused' ? (s.pausedFrom ?? s.status) : null,
					controlPending: acknowledged ? null : s.controlPending,
				};
			});
		}),
		subscribe(onRowAdded, (e: RowAddedEvent) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				// A new row is saved together with the page it was extracted from.
				const row: RunRow = {
					id: e.row_id,
					data: e.data,
					confidence: e.confidence,
					sources: 1,
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
		subscribe(onRowsReplaced, (event) => {
			runState.update((state) => ({
				...state,
				rows: event.rows.map(({ source_count, ...row }) => ({ ...row, sources: source_count })),
				progress: state.progress ? { ...state.progress, rows_found: event.rows.length } : null,
			}));
		}),
		subscribe(onAccounting, (event) => {
			runState.update((state) => ({ ...state, accounting: event.accounting }));
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
			// The shell owns the diagnostic log. Run activity survives clearing that log.
			runState.update((s) => ({ ...s, activity: [...s.activity, e].slice(-100) }));
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
					reason: e.reason ?? '',
					relevance_score: e.relevance_score,
					low_relevance: e.low_relevance ?? false,
					hidden: false,
					visited_at: null,
					created_at: Date.now() / 1000,
				};
				return { ...s, linkResults: [...s.linkResults, link] };
			});
		}),
		subscribe(onResearchStep, (e: ResearchStepEvent) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				const turnIndex = e.turn_index ?? Math.max(0, s.researchTurns.length - 1);
				const step: ResearchStep = {
					id: e.step_id,
					turn_index: turnIndex,
					step_index: e.step_index,
					step_type: e.step_type,
					content: e.content,
					url: e.url,
				};
				return {
					...s,
					researchSteps: [...s.researchSteps, step],
					researchTurns: updateTurn(s.researchTurns, turnIndex, (turn) => ({
						...turn,
						steps: [...turn.steps, step],
					})),
				};
			});
		}),
		subscribe(onResearchAnswer, (e: ResearchAnswerEvent) => {
			runState.update((s) => {
				if (s.runId !== e.run_id) return s;
				const turnIndex = e.turn_index ?? Math.max(0, s.researchTurns.length - 1);
				return {
					...s,
					researchAnswer: e.markdown,
					researchTurns: updateTurn(s.researchTurns, turnIndex, (turn) => ({
						...turn,
						answer: e.markdown,
						followUps: e.follow_ups ?? [],
					})),
				};
			});
		}),
	]);

	const unsubs = subscriptions.flatMap((result) =>
		result.status === 'fulfilled' ? [result.value] : []
	);
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

export async function startNewRun(
	query: string,
	runType: string = 'table',
	stopConditions?: import('$lib/api/tauri').StopConditions,
	/** Table runs: columns to offer for review instead of planning new ones. */
	schema?: SchemaColumn[] | null
) {
	const currentGeneration = ++generation;
	unsubscribeEvents();
	runState.set({
		...initialState,
		query,
		runType,
		status: 'pending',
		limits: stopConditions ?? null,
		researchTurns: runType === 'research' ? [newTurn(0, query)] : [],
	});

	const earlyEvents: (() => void)[] = [];
	const start = (async () => {
		try {
			await subscribeEvents(currentGeneration, earlyEvents);
			if (currentGeneration !== generation) return;
			const resp = await apiStartRun(query, runType, stopConditions, schema);
			if (currentGeneration !== generation) return;
			runState.update((s) => ({ ...s, runId: resp.run_id }));
			for (const deliver of earlyEvents) deliver();
		} catch (error) {
			if (currentGeneration === generation) {
				unsubscribeEvents();
				runState.update((s) => ({
					...s,
					status: 'failed',
					error: String(error),
					controlPending: null,
				}));
				addLog({
					timestamp: new Date().toISOString(),
					level: 'ERROR',
					message: `[run] Start failed: ${String(error)}`,
				});
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
		addLog({
			timestamp: new Date().toISOString(),
			level: 'INFO',
			message: `[run] Requesting ${action}`,
		});
		await send(runId);
		// Keep the pending indicator until the backend publishes the new status.
	} catch (error) {
		if (
			currentGeneration !== generation ||
			request !== controlRequest ||
			get(runState).status === 'failed'
		)
			return;
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

/** Asks a follow-up question in the open research conversation. */
export async function askFollowUp(question: string, stopConditions: StopConditions) {
	const state = get(runState);
	if (!state.runId || state.runType !== 'research') throw new Error('No research conversation is open.');
	if (!TERMINAL.includes(state.status))
		throw new Error('This conversation is still answering a question.');
	const index = state.researchTurns.length;
	runState.update((s) => ({
		...s,
		status: 'pending',
		error: null,
		controlError: null,
		limits: stopConditions,
		accounting: null,
		progress: null,
		researchTurns: [...s.researchTurns, newTurn(index, question)],
	}));
	try {
		await apiAskFollowUp(state.runId, question, stopConditions);
		runState.update((s) => (s.status === 'pending' ? { ...s, status: 'running' } : s));
	} catch (error) {
		runState.update((s) => ({
			...s,
			status: state.status,
			error: String(error),
			researchTurns: s.researchTurns.filter((turn) => turn.index !== index),
		}));
		throw error;
	}
}

/**
 * Turns of a saved research run. Data saved before conversations becomes one turn.
 * A turn still marked running after the app was closed has nothing behind it and counts as cancelled.
 */
export function researchTurnsFrom(
	result: ResearchResult,
	query: string,
	runStatus: string
): ResearchTurnState[] {
	const turns = result.turns?.length
		? result.turns
		: [
				{
					turn_index: 0,
					question: query,
					answer_markdown: result.answer_markdown,
					follow_ups: [],
					status: runStatus,
					limits: {},
					accounting: null,
				},
			];
	return turns.map((turn) => ({
		index: turn.turn_index,
		question: turn.question,
		answer: turn.answer_markdown,
		followUps: turn.follow_ups ?? [],
		status: turn.status === 'running' ? 'cancelled' : turn.status,
		steps: result.steps.filter((step) => (step.turn_index ?? 0) === turn.turn_index),
		accounting: turn.accounting ?? null,
	}));
}

/** Opens a saved research conversation in the run view, so it can be continued. */
export async function openConversation(runId: string) {
	const currentGeneration = ++generation;
	unsubscribeEvents();
	const [run, result, issues] = await Promise.all([
		getRun(runId),
		getResearchResult(runId),
		// Notices of earlier questions are helpful but not required to continue.
		getRunIssues(runId).catch(() => []),
	]);
	if (!run) throw new Error('Run not found');
	const turns = researchTurnsFrom(result, run.query, run.status);
	const last = result.turns?.at(-1);
	runState.set({
		...initialState,
		runId,
		query: run.query,
		runType: 'research',
		status: TERMINAL.includes(run.status) ? run.status : 'cancelled',
		researchTurns: turns,
		researchSteps: result.steps,
		researchAnswer: result.answer_markdown,
		limits: last?.limits && Object.keys(last.limits).length ? last.limits : null,
		llmIssues: issues,
		noticesDismissed: parseDismissed(run.dismissed_notices),
	});
	await subscribeEvents(currentGeneration, []);
}

export function parseDismissed(raw: string | null | undefined): Record<string, number> | null {
	if (!raw) return null;
	try {
		const value = JSON.parse(raw);
		return value && typeof value === 'object' ? value : null;
	} catch {
		return null;
	}
}

/** Marks the run's current notices as read. */
export async function dismissNotices(dismissed: Record<string, number>) {
	const runId = get(runState).runId;
	if (!runId) return;
	runState.update((s) => (s.runId === runId ? { ...s, noticesDismissed: dismissed } : s));
	await dismissRunNotices(runId, dismissed);
}

export function resetRun() {
	generation++;
	pendingStart = null;
	unsubscribeEvents();
	runState.set({ ...initialState });
}
