import { describe, expect, it } from 'vitest';
import type { Accounting, LlmIssueEvent } from '$lib/types';
import { buildNotices, dismissalOf, isDismissed, noticesSummary } from '$lib/utils/notices';
import { llmIssue } from './fixtures/llm-issue';

const issue = (patch: Partial<LlmIssueEvent>): LlmIssueEvent => ({ ...llmIssue, ...patch });
const accounting = (patch: Partial<Accounting> = {}): Accounting => ({
	spent_usd: 0,
	max_budget_usd: 1,
	reported_usd: 0,
	estimated_usd: 0,
	reported_calls: 0,
	estimated_calls: 0,
	unpriced_calls: 0,
	pending_calls: 0,
	missing_usage_calls: 0,
	prompt_tokens: 0,
	completion_tokens: 0,
	reasoning_tokens: 0,
	cached_prompt_tokens: 0,
	llm_calls: 0,
	search_calls: 0,
	fetch_calls: 0,
	breakdown: [],
	...patch,
});
const unread = issue({
	code: 'invalid_response',
	stage: 'research',
	message: 'The model response did not contain a valid research action',
	attempt: 1,
	max_attempts: 1,
});

describe('run notices', () => {
	it('says what a recorded outcome means for the result and how serious it is', () => {
		const [notice] = buildNotices([{ ...unread, outcome: 'continued', turn_index: 1 }], null, 'completed');
		expect(notice).toMatchObject({
			level: 'info',
			title: 'The response could not be read',
			consequence: 'The agent asked the model again and continued. The answer is not affected.',
			count: 1,
			turnIndex: 1,
		});
		const levels = (['recovered', 'continued', 'fallback', 'skipped', 'stopped'] as const).map(
			(outcome) => buildNotices([issue({ outcome, stage: 'extractor' })], null, 'completed')[0].level
		);
		expect(levels).toEqual(['info', 'info', 'warning', 'warning', 'error']);
	});

	it('infers the outcome of runs recorded before outcomes existed', () => {
		const infer = (patch: Partial<LlmIssueEvent>, status = 'completed') =>
			buildNotices([issue(patch)], null, status)[0];
		// The heat-pump conversation: an unreadable research reply the agent worked around.
		expect(infer(unread)).toMatchObject({ level: 'info', consequence: expect.stringMatching(/not affected/) });
		expect(infer({ stage: 'extractor', code: 'timeout' })).toMatchObject({
			level: 'warning',
			consequence: 'Data from 1 page was skipped.',
		});
		expect(infer({ stage: 'link_search_planner', code: 'timeout' }).consequence).toBe(
			'Searches used simple variations of your query instead of planned ones.'
		);
		expect(infer({ stage: 'image_ranker', code: 'timeout' }).consequence).toBe(
			'Images are shown without relevance ranking.'
		);
		expect(infer({ stage: 'image_ranker', code: 'invalid_response' }).consequence).toBe(
			'A batch of images was left out.'
		);
		expect(infer({ stage: 'link_ranker', code: 'timeout' }).consequence).toBe('1 page left out of the links.');
		expect(infer({ stage: 'link_ranker', code: 'invalid_response' }).consequence).toBe(
			'1 page could not be scored and was placed among weak links.'
		);
		expect(infer({ stage: 'interpreter', code: 'auth' })).toMatchObject({ level: 'error' });
		expect(infer({ stage: null, code: 'timeout' })).toMatchObject({
			level: 'warning',
			consequence: 'Some results may be missing.',
		});
		// A scheduled retry with nothing after it succeeded once the run finished…
		expect(infer({ code: 'rate_limit', will_retry: true })).toMatchObject({
			level: 'info',
			consequence: 'A retry succeeded. Nothing was lost.',
		});
		// …and is still under way while the run is.
		expect(infer({ code: 'rate_limit', will_retry: true }, 'running').consequence).toBe(
			'Retrying the request…'
		);
	});

	it('follows one request through its retries', () => {
		const attempts = [
			issue({ code: 'rate_limit', stage: 'extractor', attempt: 1, will_retry: true, call_id: 'c1' }),
			issue({ code: 'timeout', stage: 'extractor', attempt: 2, will_retry: true, call_id: 'c1' }),
			issue({ code: 'timeout', stage: 'extractor', attempt: 3, will_retry: false, call_id: 'c1', outcome: 'skipped' }),
		];
		const [notice] = buildNotices(attempts, null, 'completed');
		expect(notice).toMatchObject({ count: 1, title: 'The request timed out', level: 'warning' });
		expect(notice.attempts).toHaveLength(3);
		// Without call ids, consecutive attempts of the same stage and model are one request.
		const legacy = attempts.map(({ call_id: _, outcome: __, ...rest }) => rest);
		expect(buildNotices(legacy, null, 'completed')).toHaveLength(1);
		// A retry that succeeded is recorded as recovered.
		const recovered = buildNotices(
			[attempts[0], issue({ code: 'rate_limit', stage: 'extractor', attempt: 2, call_id: 'c1', outcome: 'recovered' })],
			null,
			'completed'
		);
		expect(recovered).toMatchObject([{ level: 'info', consequence: 'A retry succeeded. Nothing was lost.' }]);
	});

	it('groups repeated failures with a count and lists the worst first', () => {
		const pages = ['a', 'b', 'c'].map((id) =>
			issue({ code: 'invalid_response', stage: 'extractor', call_id: id, outcome: 'skipped' })
		);
		const notices = buildNotices(
			[issue({ code: 'rate_limit', stage: 'extractor', call_id: 'r', outcome: 'recovered' }), ...pages],
			accounting({ unpriced_calls: 2, breakdown: [] }),
			'completed'
		);
		expect(notices.map((n) => [n.level, n.count])).toEqual([
			['warning', 3],
			['warning', 2],
			['info', 1],
		]);
		expect(notices[0].consequence).toBe('Data from 3 pages was skipped.');
	});

	it('turns cost warnings into notices with an action', () => {
		const notices = buildNotices(
			[],
			accounting({
				spent_usd: 1,
				unpriced_calls: 3,
				breakdown: [
					{ provider: 'brave', model: '', calls: 2, reported_usd: 0, estimated_usd: 0, unpriced_calls: 2, prompt_tokens: 0, completion_tokens: 0, pricing: null },
					{ provider: 'ollama_cloud', model: 'm', calls: 1, reported_usd: 0, estimated_usd: 0, unpriced_calls: 1, prompt_tokens: 0, completion_tokens: 0, pricing: null },
				],
			}),
			'completed'
		);
		expect(notices).toMatchObject([
			{ key: 'cost:limit', level: 'warning', title: 'Spending limit reached' },
			{
				key: 'cost:unknown',
				level: 'warning',
				title: '3 requests with unknown cost',
				consequence: 'They are not counted toward the spending limit (Brave Search: 2 · Ollama Cloud: 1).',
				action: { label: 'Set prices', href: '/settings#settings-llm' },
			},
		]);
	});

	it('offers settings only where they can fix the problem', () => {
		const [auth] = buildNotices([issue({ code: 'auth', stage: 'interpreter' })], null, 'failed');
		expect(auth.action?.label).toBe('Open Settings');
		const [recovered] = buildNotices([issue({ code: 'auth', outcome: 'recovered' })], null, 'completed');
		expect(recovered.action).toBeUndefined();
	});

	it('summarises the strip and remembers what was dismissed', () => {
		const notices = buildNotices(
			[unread, issue({ stage: 'extractor', code: 'timeout', call_id: 'x', outcome: 'skipped' })],
			null,
			'completed'
		);
		expect(noticesSummary(notices)).toEqual({
			level: 'warning',
			count: 2,
			text: 'The request timed out',
		});
		expect(noticesSummary(notices.filter((n) => n.level === 'info')).level).toBe('info');
		const dismissed = dismissalOf(notices);
		expect(isDismissed(notices, dismissed)).toBe(true);
		expect(isDismissed(notices, null)).toBe(false);
		// One more failure of the same kind brings the strip back.
		const more = buildNotices(
			[unread, ...['x', 'y'].map((id) => issue({ stage: 'extractor', code: 'timeout', call_id: id, outcome: 'skipped' }))],
			null,
			'completed'
		);
		expect(isDismissed(more, dismissed)).toBe(false);
	});
});
