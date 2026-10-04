import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { askFollowUp, dismissNotices, openConversation, resetRun, runState, startNewRun } from '$lib/stores/run';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

const callbacks = new Map<string, (event: { payload: unknown }) => void>();
const apiInvoke = vi.mocked(invoke);
const emit = (name: string, payload: Record<string, unknown>) =>
	callbacks.get(`run:${name}`)?.({ payload: { run_id: 'run-1', ...payload } });
const limits = { target_row_count: 16, max_budget_usd: 1, max_duration_seconds: 600 };

describe('research conversations', () => {
	beforeEach(() => {
		resetRun();
		callbacks.clear();
		apiInvoke.mockReset().mockImplementation(async (command) => {
			if (command === 'start_run') return { run_id: 'run-1' } as never;
			if (command === 'get_run')
				return { id: 'run-1', query: 'Find proxies', status: 'completed', run_type: 'research', stats: null, error: null, created_at: 1, dismissed_notices: '{"cost:unknown":1}' } as never;
			if (command === 'get_run_issues')
				return [{ run_id: 'run-1', code: 'invalid_response', stage: 'research', turn_index: 0, outcome: 'continued' }] as never;
			if (command === 'get_research_result')
				return {
					answer_markdown: 'Use Proxy-Seller.',
					steps: [
						{ id: 's0', turn_index: 0, step_index: 0, step_type: 'search', content: 'proxies', url: null },
						{ id: 's1', turn_index: 1, step_index: 0, step_type: 'think', content: 'Compare', url: null },
					],
					turns: [
						{ turn_index: 0, question: 'Find proxies', answer_markdown: 'Use Proxy-Seller.', follow_ups: ['Cheapest?'], status: 'completed', limits, accounting: { spent_usd: 0.25 } },
						{ turn_index: 1, question: 'Mobile only?', answer_markdown: null, follow_ups: [], status: 'cancelled', limits, accounting: null },
					],
				} as never;
			return undefined as never;
		});
		vi.mocked(listen).mockReset().mockImplementation(async (event, callback) => {
			callbacks.set(event as string, callback as (event: { payload: unknown }) => void);
			return () => {};
		});
	});
	afterEach(resetRun);

	it('keeps notices of earlier questions and remembers their dismissal', async () => {
		await openConversation('run-1');
		expect(get(runState).llmIssues).toMatchObject([{ code: 'invalid_response', turn_index: 0 }]);
		expect(get(runState).noticesDismissed).toEqual({ 'cost:unknown': 1 });

		await askFollowUp('Which are cheapest?', limits);
		expect(get(runState).llmIssues).toHaveLength(1);

		await dismissNotices({ 'llm:x': 2 });
		expect(get(runState).noticesDismissed).toEqual({ 'llm:x': 2 });
		expect(apiInvoke).toHaveBeenCalledWith('dismiss_run_notices', { runId: 'run-1', dismissed: { 'llm:x': 2 } });

		await startNewRun('Other', 'research', limits);
		expect(get(runState).noticesDismissed).toBeNull();
		expect(get(runState).llmIssues).toEqual([]);
	});

	it('groups live steps and the answer into the first turn', async () => {
		await startNewRun('Find proxies', 'research', limits);
		emit('research_step', { step_id: 'a', turn_index: 0, step_index: 0, step_type: 'search', content: 'q', url: null });
		emit('research_answer', { turn_index: 0, markdown: 'Answer', follow_ups: ['Cheapest?'] });
		emit('status_changed', { status: 'completed' });
		const [turn] = get(runState).researchTurns;
		expect(turn).toMatchObject({ index: 0, question: 'Find proxies', answer: 'Answer', followUps: ['Cheapest?'], status: 'completed' });
		expect(turn.steps.map((s) => s.id)).toEqual(['a']);
		expect(get(runState).researchAnswer).toBe('Answer');
	});

	it('asks a follow-up in the same run and cancels only that turn', async () => {
		await startNewRun('Find proxies', 'research', limits);
		emit('research_answer', { turn_index: 0, markdown: 'Answer', follow_ups: [] });
		emit('status_changed', { status: 'completed' });

		await askFollowUp('Which are cheapest?', { ...limits, target_row_count: 8 });
		expect(apiInvoke).toHaveBeenCalledWith('ask_follow_up', {
			runId: 'run-1',
			question: 'Which are cheapest?',
			stopConditions: { ...limits, target_row_count: 8 },
		});
		expect(get(runState).status).toBe('running');
		expect(get(runState).limits?.target_row_count).toBe(8);
		emit('research_step', { step_id: 'b', turn_index: 1, step_index: 0, step_type: 'fetch', content: 'Prices', url: 'https://p.example' });
		emit('status_changed', { status: 'cancelled' });

		const turns = get(runState).researchTurns;
		expect(turns.map((t) => [t.question, t.status])).toEqual([
			['Find proxies', 'completed'],
			['Which are cheapest?', 'cancelled'],
		]);
		expect(turns[1].steps.map((s) => s.id)).toEqual(['b']);
		expect(turns[0].answer).toBe('Answer');
	});

	it('does not ask while a turn is still being answered', async () => {
		await startNewRun('Find proxies', 'research', limits);
		await expect(askFollowUp('Too early', limits)).rejects.toThrow('still answering');
		expect(apiInvoke).not.toHaveBeenCalledWith('ask_follow_up', expect.anything());
	});

	it('opens a saved conversation from history to continue it', async () => {
		await openConversation('run-1');
		const state = get(runState);
		expect(state).toMatchObject({ runId: 'run-1', runType: 'research', status: 'completed', query: 'Find proxies' });
		expect(state.researchTurns.map((t) => [t.question, t.status, t.steps.length])).toEqual([
			['Find proxies', 'completed', 1],
			['Mobile only?', 'cancelled', 1],
		]);
		expect(state.researchTurns[0].accounting?.spent_usd).toBe(0.25);
		expect(state.limits).toEqual(limits);

		await askFollowUp('And free ones?', limits);
		expect(get(runState).researchTurns).toHaveLength(3);
		emit('research_answer', { turn_index: 2, markdown: 'Free lists.', follow_ups: [] });
		expect(get(runState).researchTurns[2].answer).toBe('Free lists.');
	});
});
