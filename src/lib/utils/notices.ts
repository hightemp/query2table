import type { Accounting, LlmIssueEvent, LlmIssueOutcome } from '$lib/types';
import { presentError } from '$lib/utils/errors';

export type NoticeLevel = 'info' | 'warning' | 'error';

export interface Notice {
	/** Stable identity of the kind of notice, used to remember dismissal. */
	key: string;
	level: NoticeLevel;
	title: string;
	/** What it meant for the result. */
	consequence: string;
	/** Requests (or, for costs, calls) the notice stands for. */
	count: number;
	turnIndex: number | null;
	action?: { label: string; href: string };
	/** Model request failures behind the notice, oldest first. */
	attempts: LlmIssueEvent[];
}

export const STAGE_LABELS: Record<string, string> = {
	interpreter: 'Analyzing query',
	planner: 'Planning schema',
	schema_planner: 'Planning schema',
	search_planner: 'Planning search queries',
	query_expander: 'Expanding search queries',
	extractor: 'Extracting data',
	image_ranker: 'Ranking images',
	link_ranker: 'Ranking links',
	research: 'Research',
	setup: 'Setting up the run',
	image_search_planner: 'Planning image searches',
	link_search_planner: 'Planning link searches',
};

const PROVIDER_NAMES: Record<string, string> = {
	brave: 'Brave Search',
	serper: 'Serper',
	openrouter: 'OpenRouter',
	ollama: 'Ollama',
	ollama_cloud: 'Ollama Cloud',
	openai_compatible: 'OpenAI-compatible',
};

const LEVEL_RANK: Record<NoticeLevel, number> = { error: 0, warning: 1, info: 2 };
const OUTCOME_LEVEL: Record<LlmIssueOutcome | 'unknown', NoticeLevel> = {
	retrying: 'info',
	recovered: 'info',
	continued: 'info',
	fallback: 'warning',
	skipped: 'warning',
	stopped: 'error',
	unknown: 'warning',
};

/**
 * What each stage does when a request finally fails; mirrors the backend for runs
 * recorded before outcomes were stored.
 */
function inferredOutcome(stage: string | null, code: string): LlmIssueOutcome | 'unknown' {
	if (code === 'budget_limit') return 'stopped';
	switch (stage) {
		case 'interpreter':
		case 'planner':
		case 'schema_planner':
		case 'search_planner':
		case 'query_expander':
		case 'setup':
			return 'stopped';
		case 'extractor':
			return 'skipped';
		case 'link_ranker':
			// An unreadable score ranks the page as irrelevant; a failed request drops it.
			return code === 'invalid_response' ? 'fallback' : 'skipped';
		case 'image_ranker':
			return code === 'invalid_response' ? 'skipped' : 'fallback';
		case 'image_search_planner':
		case 'link_search_planner':
			return 'fallback';
		case 'research':
			return 'continued';
		default:
			return 'unknown';
	}
}

function plural(count: number, word: string) {
	return `${count} ${word}${count === 1 ? '' : 's'}`;
}

function consequence(outcome: LlmIssueOutcome | 'unknown', stage: string | null, code: string, count: number) {
	switch (outcome) {
		case 'retrying':
			return 'Retrying the request…';
		case 'recovered':
			return count === 1
				? 'A retry succeeded. Nothing was lost.'
				: `Retries succeeded for ${count} requests. Nothing was lost.`;
		case 'continued':
			return 'The agent asked the model again and continued. The answer is not affected.';
		case 'fallback':
			if (stage === 'image_ranker') return 'Images are shown without relevance ranking.';
			if (stage === 'link_ranker')
				return `${plural(count, 'page')} could not be scored and ${count === 1 ? 'was' : 'were'} placed among weak links.`;
			if (stage === 'image_search_planner' || stage === 'link_search_planner')
				return 'Searches used simple variations of your query instead of planned ones.';
			return 'A simpler method was used, so results may be less precise.';
		case 'skipped':
			if (stage === 'extractor') return `Data from ${plural(count, 'page')} was skipped.`;
			if (stage === 'link_ranker') return `${plural(count, 'page')} left out of the links.`;
			if (stage === 'image_ranker')
				return count === 1 ? 'A batch of images was left out.' : `${count} batches of images were left out.`;
			return 'Some results were skipped.';
		case 'stopped':
			return code === 'budget_limit'
				? 'The spending limit stopped the run at this step.'
				: 'The run stopped at this step.';
		default:
			return 'Some results may be missing.';
	}
}

/** Splits issues into requests: one per call id, or consecutive attempts of one stage and model. */
function requests(issues: LlmIssueEvent[]): LlmIssueEvent[][] {
	const result: LlmIssueEvent[][] = [];
	const byCall = new Map<string, LlmIssueEvent[]>();
	let open: LlmIssueEvent[] | null = null;
	for (const issue of issues) {
		if (issue.call_id) {
			const call = byCall.get(issue.call_id);
			if (call) call.push(issue);
			else {
				const fresh = [issue];
				byCall.set(issue.call_id, fresh);
				result.push(fresh);
			}
			continue;
		}
		const last = open?.at(-1);
		if (
			open &&
			last &&
			last.will_retry &&
			last.stage === issue.stage &&
			last.model === issue.model &&
			issue.attempt === last.attempt + 1
		)
			open.push(issue);
		else {
			open = [issue];
			result.push(open);
		}
	}
	return result;
}

function settledOutcome(attempts: LlmIssueEvent[], runStatus: string): LlmIssueOutcome | 'unknown' {
	const last = attempts[attempts.length - 1];
	if (last.outcome) return last.outcome;
	if (last.will_retry)
		return runStatus === 'running' || runStatus === 'pending' || runStatus === 'paused'
			? 'retrying'
			: 'recovered';
	return inferredOutcome(last.stage, last.code);
}

function llmNotices(issues: LlmIssueEvent[], runStatus: string) {
	const groups = new Map<string, { notice: Notice; order: number; outcome: LlmIssueOutcome | 'unknown' }>();
	requests(issues).forEach((attempts, order) => {
		const outcome = settledOutcome(attempts, runStatus);
		// The recovered record repeats the failure it recovered from; describe that failure.
		const failure = [...attempts].reverse().find((a) => a.outcome !== 'recovered') ?? attempts[0];
		const turnIndex = attempts.find((a) => a.turn_index != null)?.turn_index ?? null;
		const key = ['llm', failure.code, failure.stage ?? '', failure.model, outcome, turnIndex ?? ''].join(':');
		const existing = groups.get(key);
		if (existing) {
			existing.notice.count += 1;
			existing.notice.attempts.push(...attempts);
			existing.order = order;
			return;
		}
		const explanation = presentError(failure.message, 'run', failure.code);
		const level = OUTCOME_LEVEL[outcome];
		groups.set(key, {
			order,
			outcome,
			notice: {
				key,
				level,
				title: explanation.title,
				consequence: '',
				count: 1,
				turnIndex,
				action:
					level !== 'info' && explanation.settingsHref
						? { label: 'Open Settings', href: explanation.settingsHref }
						: undefined,
				attempts: [...attempts],
			},
		});
	});
	return [...groups.values()]
		.sort((a, b) => b.order - a.order)
		.map(({ notice, outcome }) => {
			const last = notice.attempts.find((a) => a.outcome !== 'recovered') ?? notice.attempts[0];
			return { ...notice, consequence: consequence(outcome, last.stage, last.code, notice.count) };
		});
}

function costNotices(accounting: Accounting | null): Notice[] {
	if (!accounting) return [];
	const notices: Notice[] = [];
	const base = { count: 1, turnIndex: null, attempts: [] };
	if (accounting.spent_usd >= accounting.max_budget_usd - accounting.max_budget_usd * 1e-12)
		notices.push({
			...base,
			key: 'cost:limit',
			level: 'warning',
			title: 'Spending limit reached',
			consequence: 'New requests have stopped; requests already in flight may still be charged.',
		});
	if (accounting.unpriced_calls > 0) {
		const totals = new Map<string, number>();
		for (const line of accounting.breakdown) {
			if (line.unpriced_calls > 0)
				totals.set(line.provider, (totals.get(line.provider) ?? 0) + line.unpriced_calls);
		}
		const providers = [...totals].map(([provider, n]) => `${PROVIDER_NAMES[provider] ?? provider}: ${n}`).join(' · ');
		// Search prices live in the Search section, model rates under LLM.
		const section = [...totals.keys()].some((p) => !['brave', 'serper'].includes(p)) ? 'llm' : 'search';
		notices.push({
			...base,
			key: 'cost:unknown',
			count: accounting.unpriced_calls,
			level: 'warning',
			title: `${plural(accounting.unpriced_calls, 'request')} with unknown cost`,
			consequence: `${accounting.unpriced_calls === 1 ? 'It is' : 'They are'} not counted toward the spending limit${providers ? ` (${providers})` : ''}.`,
			action: { label: 'Set prices', href: `/settings#settings-${section}` },
		});
	}
	return notices;
}

/** Everything worth telling about a run's requests and costs, most serious first. */
export function buildNotices(
	issues: LlmIssueEvent[],
	accounting: Accounting | null,
	runStatus: string
): Notice[] {
	const all = [...llmNotices(issues, runStatus), ...costNotices(accounting)];
	// Stable sort: within a level, model issues (latest first) come before costs.
	return all.sort((a, b) => LEVEL_RANK[a.level] - LEVEL_RANK[b.level]);
}

export function noticesSummary(notices: Notice[]): { level: NoticeLevel; count: number; text: string } {
	const top = notices[0];
	return { level: top?.level ?? 'info', count: notices.length, text: top?.title ?? '' };
}

/** What a dismissal remembers: how many of each kind of notice had been seen. */
export function dismissalOf(notices: Notice[]): Record<string, number> {
	return Object.fromEntries(notices.map((notice) => [notice.key, notice.count]));
}

/** Dismissed until a new kind of notice appears or one grows. */
export function isDismissed(notices: Notice[], dismissed: Record<string, number> | null | undefined): boolean {
	if (!dismissed || !notices.length) return false;
	return notices.every((notice) => (dismissed[notice.key] ?? -1) >= notice.count);
}
