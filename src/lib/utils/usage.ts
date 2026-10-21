/** Periods, sizes and CSV of the Statistics section. Days and months follow the local calendar. */
import { formatNumber, intlLocale, t } from '$lib/i18n';
import type { UsageReport, UsageRequest } from '$lib/types';
import type { Chart } from './chart';

export type UsagePeriodKind = 'month' | 'year' | 'all';

export interface UsagePeriod {
	request: UsageRequest;
	/** Chart labels, one per bucket. */
	labels: string[];
	/** Bucket names for files: 2026-10-01, 2026-10 or 2026. */
	keys: string[];
	title: string;
}

/** All time is shown by month up to this many months, by year beyond. */
const MAX_MONTHS = 36;

const seconds = (date: Date) => Math.floor(date.getTime() / 1000);
const pad = (n: number) => String(n).padStart(2, '0');

function months(from: Date, to: Date): Date[] {
	const list: Date[] = [];
	for (let d = new Date(from); d < to; d = new Date(d.getFullYear(), d.getMonth() + 1, 1)) list.push(d);
	return list;
}

export function usagePeriod(kind: UsagePeriodKind, anchor: Date, firstAt: number | null): UsagePeriod {
	const year = anchor.getFullYear();
	const month = anchor.getMonth();
	const shortMonth = (d: Date) => d.toLocaleString(intlLocale(), { month: 'short' }).replace('.', '');
	if (kind === 'month') {
		const from = new Date(year, month, 1);
		const to = new Date(year, month + 1, 1);
		const days: Date[] = [];
		for (let d = new Date(from); d < to; d = new Date(d.getFullYear(), d.getMonth(), d.getDate() + 1)) days.push(d);
		return {
			request: { from: seconds(from), to: seconds(to), buckets: days.map(seconds), previous_from: seconds(new Date(year, month - 1, 1)) },
			labels: days.map((d) => String(d.getDate())),
			keys: days.map((d) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`),
			title: from.toLocaleString(intlLocale(), { month: 'long', year: 'numeric' }).replace(/^./, (c) => c.toUpperCase()),
		};
	}
	if (kind === 'year') {
		const from = new Date(year, 0, 1);
		const list = months(from, new Date(year + 1, 0, 1));
		return {
			request: { from: seconds(from), to: seconds(new Date(year + 1, 0, 1)), buckets: list.map(seconds), previous_from: seconds(new Date(year - 1, 0, 1)) },
			labels: list.map(shortMonth),
			keys: list.map((d) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}`),
			title: String(year),
		};
	}
	const first = firstAt === null ? anchor : new Date(firstAt * 1000);
	const to = new Date(year, month + 1, 1);
	let from = new Date(first.getFullYear(), first.getMonth(), 1);
	let buckets = months(from, to);
	let labels = buckets.map((d) => `${shortMonth(d)} ${d.getFullYear()}`);
	let keys = buckets.map((d) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}`);
	if (buckets.length > MAX_MONTHS) {
		from = new Date(first.getFullYear(), 0, 1);
		buckets = [];
		for (let y = from.getFullYear(); y <= year; y++) buckets.push(new Date(y, 0, 1));
		labels = keys = buckets.map((d) => String(d.getFullYear()));
	}
	return {
		request: { from: seconds(from), to: seconds(to), buckets: buckets.map(seconds), previous_from: seconds(from) },
		labels,
		keys,
		title: t('usage.allTime'),
	};
}

/** The anchor of the period before (-1) or after (1). */
export function shiftAnchor(kind: UsagePeriodKind, anchor: Date, step: -1 | 1): Date {
	if (kind === 'year') return new Date(anchor.getFullYear() + step, 0, 1);
	return new Date(anchor.getFullYear(), anchor.getMonth() + step, 1);
}

/** Whether a later period has started already. */
export function canGoForward(kind: UsagePeriodKind, anchor: Date, now: Date): boolean {
	if (kind === 'all') return false;
	const next = shiftAnchor(kind, anchor, 1);
	return next <= now;
}

export function formatBytes(bytes: number): string {
	const units: [number, 'usage.bytes.gb' | 'usage.bytes.mb' | 'usage.bytes.kb'][] = [
		[1024 ** 3, 'usage.bytes.gb'],
		[1024 ** 2, 'usage.bytes.mb'],
		[1024, 'usage.bytes.kb'],
	];
	for (const [size, key] of units) {
		if (bytes >= size) return t(key, { size: formatNumber(bytes / size, { maximumFractionDigits: 1 }) });
	}
	return t('usage.bytes.b', { size: formatNumber(bytes) });
}

function csvCell(value: string | number): string {
	const text = String(value);
	return /[",\n]/.test(text) ? `"${text.replace(/"/g, '""')}"` : text;
}

const csv = (rows: (string | number)[][]) => rows.map((row) => row.map(csvCell).join(',')).join('\n');

/** The period by bucket (`days`, also months or years) or by model, as CSV. */
export function usageCsv(report: UsageReport, keys: string[], view: 'days' | 'models'): string {
	if (view === 'models') {
		return csv([
			['provider', 'model', 'calls', 'input_tokens', 'output_tokens', 'cached_tokens', 'cost_usd', 'calls_without_price'],
			...report.by_model.map((m) => [m.provider, m.model, m.calls, m.prompt_tokens, m.completion_tokens, m.cached_tokens, m.cost_usd, m.unpriced_calls]),
		]);
	}
	return csv([
		[
			'period',
			'cost_usd',
			'reported_usd',
			'estimated_usd',
			'calls_without_price',
			'model_calls',
			'input_tokens',
			'output_tokens',
			'reasoning_tokens',
			'cached_tokens',
			'searches',
			'downloads',
			'web_bytes',
			'model_bytes_sent',
			'model_bytes_received',
		],
		...report.series.map(({ totals: s }, i) => [
			keys[i] ?? String(i),
			s.cost_usd,
			s.reported_usd,
			s.estimated_usd,
			s.unpriced_calls,
			s.llm_calls,
			s.prompt_tokens,
			s.completion_tokens,
			s.reasoning_tokens,
			s.cached_tokens,
			s.searches,
			s.downloads,
			s.web_bytes,
			s.model_bytes_sent,
			s.model_bytes_received,
		]),
	]);
}

export type UsageMetric = 'cost' | 'tokens' | 'searches' | 'data';
export const USAGE_METRICS: UsageMetric[] = ['cost', 'tokens', 'searches', 'data'];

const MB = 1024 * 1024;
const megabytes = (bytes: number) => Math.round((bytes / MB) * 100) / 100;

/** The chart of a metric over the period's buckets. */
export function usageChart(report: UsageReport, labels: string[], metric: UsageMetric, step: 'day' | 'month' | 'year'): Chart {
	const points = report.series.map((p) => p.totals);
	const series = (name: string, values: number[]) => ({ name, type: null, axis: 'left' as const, data: values });
	const lines = {
		cost: [series(t('usage.metric.cost'), points.map((p) => p.cost_usd))],
		tokens: [series(t('usage.tokens.input'), points.map((p) => p.prompt_tokens)), series(t('usage.tokens.output'), points.map((p) => p.completion_tokens))],
		searches: [series(t('usage.metric.searches'), points.map((p) => p.searches))],
		data: [
			series(t('usage.data.web'), points.map((p) => megabytes(p.web_bytes))),
			series(t('usage.data.sent'), points.map((p) => megabytes(p.model_bytes_sent))),
			series(t('usage.data.received'), points.map((p) => megabytes(p.model_bytes_received))),
		],
	}[metric];
	return {
		type: 'bar',
		title: t(`usage.chartTitle.${step}`, { metric: t(`usage.metric.${metric}`) }),
		x: labels,
		y: [],
		series: lines,
		stacked: lines.length > 1,
		horizontal: false,
		unit: metric === 'cost' ? 'USD' : metric === 'data' ? 'MB' : '',
		unitRight: '',
		xTitle: '',
		yTitle: '',
	};
}
