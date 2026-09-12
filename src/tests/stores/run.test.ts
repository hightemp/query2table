import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { llmIssue } from '../fixtures/llm-issue';
import {
	cancelCurrentRun, confirmCurrentSchema, pauseCurrentRun, resetRun,
	resumeCurrentRun, runState, startNewRun,
} from '$lib/stores/run';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

const callbacks = new Map<string, (event: { payload: unknown }) => void>();
const unlisten = vi.fn();
const apiInvoke = vi.mocked(invoke);
const apiListen = vi.mocked(listen);

function emit(name: string, payload: Record<string, unknown>) {
	callbacks.get(`run:${name}`)?.({ payload });
}

function status(value: string, runId = 'run-1') {
	emit('status_changed', { run_id: runId, status: value });
}

describe('run controls and startup events', () => {
	beforeEach(() => {
		resetRun();
		callbacks.clear();
		vi.clearAllMocks();
		apiInvoke.mockReset().mockImplementation(async (command) => (
			command === 'start_run' ? { run_id: 'run-1' } : undefined
		) as never);
		apiListen.mockReset().mockImplementation(async (event, callback) => {
			callbacks.set(event as string, callback as (event: { payload: unknown }) => void);
			return unlisten;
		});
	});

	afterEach(resetRun);

	it('replays early status and schema events only for the returned run ID', async () => {
		apiInvoke.mockImplementationOnce(async () => {
			status('running');
			status('failed', 'another-run');
			emit('schema_proposed', { run_id: 'run-1', columns: [{ name: 'Channel', type: 'text' }] });
			return { run_id: 'run-1' } as never;
		});
		await startNewRun('Find robot channels');
		expect(get(runState)).toMatchObject({
			runId: 'run-1', status: 'schema_review', schema: [{ name: 'Channel' }],
		});
	});

	it('buffers early model issues without failing the run and bounds retained events', async () => {
		apiInvoke.mockImplementationOnce(async () => {
			emit('llm_issue', { ...llmIssue });
			return { run_id: 'run-1' } as never;
		});
		await startNewRun('query');
		expect(get(runState)).toMatchObject({ status: 'pending', llmIssues: [llmIssue] });
		for (let i = 0; i < 120; i++) emit('llm_issue', { ...llmIssue, message: `issue-${i}` });
		expect(get(runState).llmIssues).toHaveLength(100);
		expect(get(runState).llmIssues[0].message).toBe('issue-20');
		status('completed');
		expect(get(runState).llmIssues).toHaveLength(100);
		const stale = callbacks.get('run:llm_issue')!;
		resetRun();
		await startNewRun('new query');
		stale({ payload: llmIssue });
		emit('llm_issue', { ...llmIssue, run_id: 'different-run' });
		expect(get(runState).llmIssues).toEqual([]);
	});

	it('shows startup failures and releases listeners instead of remaining pending', async () => {
		apiInvoke.mockRejectedValueOnce('Provider not configured');
		await expect(startNewRun('query')).rejects.toBe('Provider not configured');
		expect(get(runState)).toMatchObject({ status: 'failed', error: 'Provider not configured' });
		expect(unlisten).toHaveBeenCalledTimes(11);
	});

	it('cleans partial subscriptions and reports a listener setup failure', async () => {
		apiListen.mockRejectedValueOnce(new Error('Listener unavailable'));
		await expect(startNewRun('query')).rejects.toThrow('Listener unavailable');
		expect(get(runState).status).toBe('failed');
		expect(apiInvoke).not.toHaveBeenCalled();
		expect(unlisten).toHaveBeenCalledTimes(10);
	});

	it('queues Cancel during startup and waits for backend cancellation before changing status', async () => {
		let finishStart!: (value: { run_id: string }) => void;
		apiInvoke.mockImplementationOnce(() => new Promise((resolve) => { finishStart = resolve; }));
		const starting = startNewRun('query');
		await vi.waitFor(() => expect(finishStart).toBeTypeOf('function'));
		const cancelling = cancelCurrentRun();
		expect(get(runState)).toMatchObject({ status: 'pending', controlPending: 'cancel' });
		expect(apiInvoke).not.toHaveBeenCalledWith('cancel_run', expect.anything());
		finishStart({ run_id: 'run-1' });
		await Promise.all([starting, cancelling]);
		expect(apiInvoke).toHaveBeenCalledWith('cancel_run', { runId: 'run-1' });
		expect(get(runState)).toMatchObject({ status: 'pending', controlPending: 'cancel' });
		status('cancelled');
		expect(get(runState)).toMatchObject({ status: 'cancelled', controlPending: null });
	});

	it('keeps Pause and Resume pending until backend status acknowledgments arrive', async () => {
		await startNewRun('query');
		await pauseCurrentRun();
		await pauseCurrentRun();
		expect(apiInvoke.mock.calls.filter(([command]) => command === 'pause_run')).toHaveLength(1);
		expect(get(runState)).toMatchObject({ status: 'pending', controlPending: 'pause' });
		status('paused');
		expect(get(runState)).toMatchObject({ status: 'paused', pausedFrom: 'pending', controlPending: null });
		await resumeCurrentRun();
		expect(get(runState)).toMatchObject({ status: 'paused', controlPending: 'resume' });
		status('pending');
		expect(get(runState)).toMatchObject({ status: 'pending', pausedFrom: null, controlPending: null });
	});

	it('shows inactive-run command errors without inventing a cancelled status and allows retry', async () => {
		await startNewRun('query');
		status('running');
		apiInvoke.mockRejectedValueOnce('Run is not active');
		await expect(cancelCurrentRun()).resolves.toBeUndefined();
		expect(get(runState)).toMatchObject({
			status: 'running', controlPending: null, controlError: 'Could not cancel: Run is not active',
		});
		await cancelCurrentRun();
		expect(get(runState)).toMatchObject({ controlPending: 'cancel', controlError: null });
	});

	it('allows cancellation while a pause is pending and does not clear Cancel on the pause event', async () => {
		await startNewRun('query');
		status('running');
		await pauseCurrentRun();
		await cancelCurrentRun();
		status('paused');
		expect(get(runState)).toMatchObject({ status: 'paused', controlPending: 'cancel' });
		status('cancelled');
		expect(get(runState).controlPending).toBeNull();
	});

	it('retains schema-review origin when paused and surfaces schema-confirm errors', async () => {
		await startNewRun('query');
		emit('schema_proposed', { run_id: 'run-1', columns: [{ name: 'Channel', type: 'text' }] });
		await pauseCurrentRun();
		status('paused');
		expect(get(runState).pausedFrom).toBe('schema_review');
		status('schema_review');
		apiInvoke.mockRejectedValueOnce('Control channel closed');
		await confirmCurrentSchema(get(runState).schema);
		expect(get(runState)).toMatchObject({
			status: 'schema_review', controlError: 'Could not confirm schema: Control channel closed',
		});
	});

	it('ignores old listener callbacks and a late startup response after reset', async () => {
		let finishStart!: (value: { run_id: string }) => void;
		apiInvoke.mockImplementationOnce(() => new Promise((resolve) => { finishStart = resolve; }));
		const starting = startNewRun('old query');
		await vi.waitFor(() => expect(finishStart).toBeTypeOf('function'));
		const staleCallback = callbacks.get('run:status_changed')!;
		resetRun();
		await startNewRun('new query');
		finishStart({ run_id: 'old-run' });
		await starting;
		staleCallback({ payload: { run_id: 'run-1', status: 'failed' } });
		expect(get(runState)).toMatchObject({ query: 'new query', runId: 'run-1', status: 'pending' });
	});
});
