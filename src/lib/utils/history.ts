import { intlLocale, t, type MessageKey } from '$lib/i18n';
import type { HistoryRun, HistorySort } from '$lib/types';
import type { StopConditions } from '$lib/api/tauri';

export interface RunSummary {
	results: string | null;
	duration: string | null;
	cost: string | null;
	model: string | null;
}

const SEARCH_PROVIDERS = ['brave', 'serper'];
const RESULT_KEYS: Record<string, [string, MessageKey]> = {
	table: ['rows_found', 'units.row'],
	links: ['link_count', 'units.link'],
	images: ['image_count', 'units.image'],
};

/** Money in a list row: cents, or "<$0.01" for tiny amounts. */
export function shortUsd(amount: number): string {
	if (amount > 0 && amount < 0.005) return '<$0.01';
	return `$${amount.toFixed(2)}`;
}

function parse(raw: string | null | undefined): Record<string, any> | null {
	if (!raw) return null;
	try {
		const value = JSON.parse(raw);
		return value && typeof value === 'object' ? value : null;
	} catch {
		return null;
	}
}

export function formatDuration(seconds: number): string {
	const s = Math.round(seconds);
	if (s < 60) return t('duration.s', { s });
	const h = Math.floor(s / 3600);
	const m = Math.floor((s % 3600) / 60);
	if (h) return m ? t('duration.hm', { h, m }) : t('duration.h', { h });
	const rest = s % 60;
	return rest ? t('duration.ms', { m, s: rest }) : t('duration.m', { m });
}

/** What a History row says about a run, from its saved stats. */
export function runSummary(run: HistoryRun): RunSummary {
	const stats = parse(run.stats);
	if (!stats) return { results: null, duration: null, cost: null, model: null };
	let results: string | null = null;
	if (run.run_type === 'research') {
		if (run.turn_count > 1) results = t('history.question', { count: run.turn_count });
		else if (typeof stats.steps === 'number') results = t('units.step', { count: stats.steps });
	} else {
		const [key, word] = RESULT_KEYS[run.run_type] ?? RESULT_KEYS.table;
		if (typeof stats[key] === 'number') results = t(word, { count: stats[key] });
	}
	const duration = typeof stats.elapsed_secs === 'number' ? formatDuration(stats.elapsed_secs) : null;
	const accounting = stats.accounting;
	let cost: string | null = null;
	if (accounting && typeof accounting.spent_usd === 'number') {
		cost =
			accounting.spent_usd <= 0 && accounting.unpriced_calls > 0
				? t('history.costUnknown')
				: shortUsd(accounting.spent_usd);
	} else if (typeof stats.spent_usd === 'number') cost = shortUsd(stats.spent_usd);
	// The model that handled most of the run's model requests.
	const models = ((accounting?.breakdown ?? []) as { provider: string; model: string; calls: number }[])
		.filter((line) => !SEARCH_PROVIDERS.includes(line.provider) && line.model)
		.sort((a, b) => b.calls - a.calls);
	return { results, duration, cost, model: models[0]?.model ?? null };
}

function startOfDay(ms: number) {
	const day = new Date(ms);
	day.setHours(0, 0, 0, 0);
	return day.getTime();
}

/** "just now", "25 min ago", "3 h ago" today; a time or date before. */
export function relativeTime(seconds: number, now = Date.now()): string {
	const ms = seconds * 1000;
	const diff = Math.max(0, now - ms);
	if (ms >= startOfDay(now)) {
		if (diff < 60_000) return t('history.justNow');
		if (diff < 3_600_000) return t('history.minutesAgo', { count: Math.floor(diff / 60_000) });
		return t('history.hoursAgo', { count: Math.floor(diff / 3_600_000) });
	}
	const date = new Date(ms);
	const time = date.toLocaleTimeString(intlLocale(), { hour: '2-digit', minute: '2-digit' });
	if (ms >= startOfDay(now) - 6 * 86_400_000)
		return `${date.toLocaleDateString(intlLocale(), { weekday: 'short' })} ${time}`;
	return date.toLocaleDateString(intlLocale(), { day: 'numeric', month: 'short', year: 'numeric' });
}

/** Day heading for a run: Today, Yesterday, This week, or its month. */
export function dateGroup(seconds: number, now = Date.now()): string {
	const ms = seconds * 1000;
	const today = startOfDay(now);
	if (ms >= today) return t('history.today');
	if (ms >= today - 86_400_000) return t('history.yesterday');
	const weekday = (new Date(now).getDay() + 6) % 7; // Monday is 0
	if (ms >= today - weekday * 86_400_000) return t('history.thisWeek');
	const month = new Date(ms).toLocaleDateString(intlLocale(), { month: 'long', year: 'numeric' });
	return month.charAt(0).toUpperCase() + month.slice(1);
}

export interface RunGroup {
	/** Null when the order is not by date. */
	label: string | null;
	runs: HistoryRun[];
}

/** Pinned runs first; then day groups when sorted by date. */
export function groupRuns(runs: HistoryRun[], now: number, sort: HistorySort): RunGroup[] {
	const groups: RunGroup[] = [];
	const pinned = runs.filter((run) => run.pinned_at);
	if (pinned.length) groups.push({ label: t('history.pinned'), runs: pinned });
	const rest = runs.filter((run) => !run.pinned_at);
	if (sort !== 'newest' && sort !== 'oldest') {
		if (rest.length) groups.push({ label: null, runs: rest });
		return groups;
	}
	for (const run of rest) {
		const label = dateGroup(run.created_at, now);
		const last = groups.at(-1);
		if (last && last.label === label) last.runs.push(run);
		else groups.push({ label, runs: [run] });
	}
	return groups;
}

/** Stop conditions saved with a run, for Run again; null for runs saved before they were. */
export function runLimits(config: string | null | undefined): Required<StopConditions> | null {
	const stop = parse(config)?.stop;
	if (
		!stop ||
		typeof stop.target_row_count !== 'number' ||
		typeof stop.max_budget_usd !== 'number' ||
		typeof stop.max_duration_seconds !== 'number'
	)
		return null;
	return {
		target_row_count: stop.target_row_count,
		max_budget_usd: stop.max_budget_usd,
		max_duration_seconds: stop.max_duration_seconds,
	};
}
