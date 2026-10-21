import { describe, expect, it } from 'vitest';
import { canGoForward, formatBytes, shiftAnchor, usageCsv, usagePeriod } from '$lib/utils/usage';
import type { UsageReport, UsageTotals } from '$lib/types';

const at = (...parts: [number, number, number?]) => new Date(parts[0], parts[1], parts[2] ?? 1).getTime() / 1000;

describe('statistics periods', () => {
	it('a month has a bucket per local day and starts after the month before', () => {
		const period = usagePeriod('month', new Date(2026, 1, 14, 15, 30), null);
		expect(period.request.from).toBe(at(2026, 1));
		expect(period.request.to).toBe(at(2026, 2));
		expect(period.request.previous_from).toBe(at(2026, 0));
		expect(period.request.buckets).toHaveLength(28);
		expect(period.request.buckets[13]).toBe(at(2026, 1, 14));
		expect(period.labels.slice(0, 3)).toEqual(['1', '2', '3']);
		expect(period.title).toBe('February 2026');
		expect(period.keys[0]).toBe('2026-02-01');
	});

	it('a year has a bucket per month', () => {
		const period = usagePeriod('year', new Date(2026, 6, 3), null);
		expect([period.request.from, period.request.to, period.request.previous_from]).toEqual([at(2026, 0), at(2027, 0), at(2025, 0)]);
		expect(period.request.buckets).toEqual(Array.from({ length: 12 }, (_, m) => at(2026, m)));
		expect(period.labels[0]).toBe('Jan');
		expect(period.title).toBe('2026');
		expect(period.keys[11]).toBe('2026-12');
	});

	it('all time runs by month from the first recorded month, with nothing before it', () => {
		const period = usagePeriod('all', new Date(2026, 9, 21), at(2026, 6, 15) + 3600);
		expect(period.request.from).toBe(at(2026, 6));
		expect(period.request.to).toBe(at(2026, 10));
		expect(period.request.previous_from).toBe(period.request.from);
		expect(period.request.buckets).toEqual([at(2026, 6), at(2026, 7), at(2026, 8), at(2026, 9)]);
		expect(period.labels[0]).toBe('Jul 2026');
		expect(period.title).toBe('All time');
		// No usage yet: this month only.
		expect(usagePeriod('all', new Date(2026, 9, 21), null).request.buckets).toEqual([at(2026, 9)]);
		// Many years go by year.
		const long = usagePeriod('all', new Date(2026, 9, 21), at(2020, 3));
		expect(long.request.buckets[0]).toBe(at(2020, 0));
		expect(long.labels).toEqual(['2020', '2021', '2022', '2023', '2024', '2025', '2026']);
	});

	it('steps back and forward, but not past the current period', () => {
		const now = new Date(2026, 9, 21);
		const back = shiftAnchor('month', now, -1);
		expect([back.getFullYear(), back.getMonth()]).toEqual([2026, 8]);
		expect(shiftAnchor('year', now, -1).getFullYear()).toBe(2025);
		// The 31st of a month steps to the month before, not two months back.
		const end = shiftAnchor('month', new Date(2026, 2, 31), -1);
		expect([end.getFullYear(), end.getMonth()]).toEqual([2026, 1]);
		expect(canGoForward('month', back, now)).toBe(true);
		expect(canGoForward('month', now, now)).toBe(false);
		expect(canGoForward('year', new Date(2025, 0, 1), now)).toBe(true);
		expect(canGoForward('all', now, now)).toBe(false);
	});
});

describe('statistics values', () => {
	it('sizes read in bytes, KB, MB and GB', () => {
		expect([formatBytes(0), formatBytes(512), formatBytes(2048), formatBytes(5.5 * 1024 * 1024), formatBytes(3 * 1024 ** 3)]).toEqual([
			'0 B',
			'512 B',
			'2 KB',
			'5.5 MB',
			'3 GB',
		]);
	});

	it('export by day and by model as CSV', () => {
		const totals = (cost: number, calls: number): UsageTotals => ({
			cost_usd: cost,
			reported_usd: cost,
			estimated_usd: 0,
			unpriced_calls: 0,
			llm_calls: calls,
			prompt_tokens: calls * 100,
			completion_tokens: calls * 10,
			reasoning_tokens: 0,
			cached_tokens: 0,
			cache_write_tokens: 0,
			searches: 1,
			downloads: 2,
			web_bytes: 300,
			model_bytes_sent: 400,
			model_bytes_received: 50,
			runs: 1,
		});
		const report = {
			totals: totals(0.3, 3),
			previous: totals(0, 0),
			series: [
				{ start: at(2026, 9, 1), totals: totals(0.1, 1) },
				{ start: at(2026, 9, 2), totals: totals(0.2, 2) },
			],
			by_model: [{ provider: 'openrouter', model: 'a, "quoted"', calls: 3, prompt_tokens: 300, completion_tokens: 30, cached_tokens: 0, cost_usd: 0.3, unpriced_calls: 1 }],
			by_search: [],
			by_mode: [],
			top_runs: [],
		} satisfies UsageReport;
		const keys = ['2026-10-01', '2026-10-02'];
		expect(usageCsv(report, keys, 'days').split('\n')).toEqual([
			'period,cost_usd,reported_usd,estimated_usd,calls_without_price,model_calls,input_tokens,output_tokens,reasoning_tokens,cached_tokens,searches,downloads,web_bytes,model_bytes_sent,model_bytes_received',
			'2026-10-01,0.1,0.1,0,0,1,100,10,0,0,1,2,300,400,50',
			'2026-10-02,0.2,0.2,0,0,2,200,20,0,0,1,2,300,400,50',
		]);
		expect(usageCsv(report, keys, 'models').split('\n')).toEqual([
			'provider,model,calls,input_tokens,output_tokens,cached_tokens,cost_usd,calls_without_price',
			'openrouter,"a, ""quoted""",3,300,30,0,0.3,1',
		]);
	});
});
