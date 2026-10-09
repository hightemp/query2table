import { test, expect, emit, viewRun } from './fixtures';

const q = (id: string, text: string, status: string, result_count = 0, error: string | null = null) => ({
	id,
	query_text: text,
	language: 'ru',
	status,
	result_count,
	error,
});

test('A table run shows its stages and searches until rows arrive', async ({ page }) => {
	await page.goto('/');
	await page.getByLabel('What would you like to find?').fill('Найди немецкие компании');
	await page.getByRole('button', { name: 'Build Table' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await emit(page, 'run:stage', { run_id: 'live', stage: 'interpret' });
	const stages = page.getByRole('region', { name: 'Run stages' });
	await expect(stages.locator('[aria-current="step"]')).toContainText('Understanding the query');

	await emit(page, 'run:stage', { run_id: 'live', stage: 'search' });
	await emit(page, 'run:search_queries', {
		run_id: 'live',
		queries: [q('a', 'немецкие компании', 'pending'), q('b', 'German companies', 'pending'), q('c', 'Firmen Deutschland', 'pending')],
	});
	await emit(page, 'run:search_query', { run_id: 'live', query: q('a', 'немецкие компании', 'completed', 4) });
	await emit(page, 'run:search_query', { run_id: 'live', query: q('b', 'German companies', 'failed', 0, 'HTTP 429') });
	await emit(page, 'run:search_query', { run_id: 'live', query: q('c', 'Firmen Deutschland', 'running') });
	await expect(stages.locator('[aria-current="step"]')).toContainText('Searching');
	await expect(stages.locator('[aria-current="step"]')).toContainText('2 of 3');
	const list = stages.getByRole('list', { name: 'Search queries' });
	await expect(list.getByRole('listitem')).toHaveCount(3);
	await expect(list.getByRole('listitem').nth(0)).toContainText('4 new');
	await list.getByRole('listitem').nth(1).getByText('failed').hover();
	await expect(page.getByRole('tooltip')).toHaveText('HTTP 429');
	await expect(list.getByRole('listitem').nth(2)).toContainText('searching…');

	await emit(page, 'run:stage', { run_id: 'live', stage: 'read' });
	await emit(page, 'run:progress_update', {
		run_id: 'live',
		stats: { rows_found: 0, pages_fetched: 30, pages_total: 120, queries_executed: 3, queries_total: 3, elapsed_secs: 9, spent_usd: 0 },
	});
	await expect(stages.locator('[aria-current="step"]')).toContainText('30 of 120 pages');
	await expect(stages.getByRole('listitem').filter({ hasText: 'Searching' })).toContainText('3 queries');

	// Rows arrive: the panel folds into one line above the table.
	await emit(page, 'run:row_added', { run_id: 'live', row_id: 'r1', data: { name: 'SAP SE' }, confidence: 0.9 });
	await expect(page.locator('.result-workspace').getByRole('region', { name: 'Run stages' })).toHaveCount(0);
	const line = page.getByRole('region', { name: 'Run stages' });
	await expect(line).toBeVisible();
	await expect(line.getByRole('list', { name: 'Search queries' })).toHaveCount(0);
	await line.getByRole('button', { name: /Search queries/ }).click();
	await expect(line.getByRole('list', { name: 'Search queries' }).getByRole('listitem')).toHaveCount(3);
});

test('Image ranking and comparing count the images done', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('button', { name: 'Images', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('Red tractors');
	await page.getByRole('button', { name: 'Search Images' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await emit(page, 'run:stage', { run_id: 'live', stage: 'rank' });
	await emit(page, 'run:stage_progress', { run_id: 'live', stage: 'rank', done: 45, total: 120 });
	const stages = page.getByRole('region', { name: 'Run stages' });
	await expect(stages.locator('[aria-current="step"]')).toContainText('Ranking images');
	await expect(stages.locator('[aria-current="step"]')).toContainText('45 of 120');
	await emit(page, 'run:stage_progress', { run_id: 'live', stage: 'rank', done: 120, total: 120 });
	await emit(page, 'run:stage', { run_id: 'live', stage: 'compare' });
	await emit(page, 'run:stage_progress', { run_id: 'live', stage: 'compare', done: 8, total: 30 });
	await expect(stages.locator('[aria-current="step"]')).toContainText('Comparing with the picture');
	await expect(stages.locator('[aria-current="step"]')).toContainText('8 of 30');
	await expect(stages.getByRole('listitem').filter({ hasText: 'Ranking images' })).toContainText('120 of 120');
});

test('A saved run keeps a folded summary of its searches', async ({ page }) => {
	await page.addInitScript(() => {
		(window as any).__uiFixture.runQueries = [
			{ id: 'a', query_text: 'robots', language: 'en', status: 'completed', result_count: 4, error: null },
			{ id: 'b', query_text: 'роботы', language: 'ru', status: 'completed', result_count: 6, error: null },
			{ id: 'c', query_text: 'Roboter', language: 'de', status: 'failed', result_count: 0, error: 'HTTP 500' },
		];
	});
	await viewRun(page, 0);
	const summary = page.getByRole('button', { name: /Search queries/ });
	await expect(summary).toContainText('3 queries · 10 results · 1 failed');
	await expect(summary).toHaveAttribute('aria-expanded', 'false');
	await summary.click();
	await expect(page.getByRole('list', { name: 'Search queries' }).getByRole('listitem')).toHaveText([
		/robots.*4 new/,
		/роботы.*6 new/,
		/Roboter.*failed/,
	]);
});
