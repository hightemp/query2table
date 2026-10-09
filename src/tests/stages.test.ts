import { describe, expect, it } from 'vitest';
import { stageViews, queriesSummary } from '$lib/utils/stages';
import type { SearchQueryInfo } from '$lib/types';

const query = (status: string, result_count = 0, error: string | null = null): SearchQueryInfo => ({
	id: status + result_count,
	query_text: 'q',
	language: 'en',
	status,
	result_count,
	error,
});
const progress = { rows_found: 4, pages_fetched: 30, pages_total: 120, queries_executed: 0, queries_total: 0, elapsed_secs: 5, spent_usd: 0 };

describe('run stages', () => {
	it('mark earlier stages done, the current one active and count searches and pages', () => {
		const queries = [query('completed', 4), query('failed', 0, 'HTTP 429'), query('running'), query('pending')];
		const views = stageViews('table', 'search', 'running', null, queries);
		expect(views.map((v) => [v.id, v.state])).toEqual([
			['interpret', 'done'],
			['schema', 'done'],
			['plan', 'done'],
			['search', 'active'],
			['read', 'waiting'],
			['dedup', 'waiting'],
		]);
		expect(views[3].label).toBe('Searching');
		expect(views[3].detail).toBe('2 of 4');

		const reading = stageViews('table', 'read', 'running', progress, queries);
		expect(reading.find((v) => v.id === 'read')).toMatchObject({ state: 'active', detail: '30 of 120 pages' });
		expect(reading.find((v) => v.id === 'search')?.detail).toBe('4 queries');
	});

	it('skip web stages in files-only runs and finish every stage when the run is done', () => {
		const filesOnly = stageViews('table', 'read', 'running', progress, []);
		expect(filesOnly.filter((v) => v.state === 'skipped').map((v) => v.id)).toEqual(['plan', 'search']);
		const done = stageViews('links', 'read', 'completed', progress, [query('completed', 3)]);
		expect(done.every((v) => v.state === 'done')).toBe(true);
		expect(done.map((v) => v.id)).toEqual(['plan', 'search', 'read']);
	});

	it('count images and pages from the stage progress', () => {
		const counts = { rank: { done: 45, total: 120 }, read: { done: 12, total: 40 } };
		const ranking = stageViews('images', 'rank', 'running', null, [], counts);
		expect(ranking.find((v) => v.id === 'rank')?.detail).toBe('45 of 120');
		const reading = stageViews('table', 'read', 'running', progress, [], counts);
		// Pages done (extracted or failed) beat pages merely fetched.
		expect(reading.find((v) => v.id === 'read')?.detail).toBe('12 of 40 pages');
	});

	it('use short names in the folded line', () => {
		expect(stageViews('table', 'read', 'running', null, [], {}, true).map((v) => v.label)).toEqual([
			'Query',
			'Columns',
			'Queries',
			'Search',
			'Reading',
			'Duplicates',
		]);
	});

	it('show comparing only for image runs that compare with a reference', () => {
		expect(stageViews('images', 'rank', 'running', null, []).map((v) => v.id)).toEqual(['plan', 'search', 'rank']);
		expect(stageViews('images', 'compare', 'running', null, []).map((v) => v.id)).toEqual(['plan', 'search', 'rank', 'compare']);
	});

	it('summarise the queries of a saved run', () => {
		expect(queriesSummary([query('completed', 4), query('completed', 6), query('failed', 0, 'x'), query('skipped')])).toBe(
			'4 queries · 10 results · 1 failed · 1 skipped'
		);
		expect(queriesSummary([])).toBe('');
	});
});
