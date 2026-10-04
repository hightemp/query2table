import { test, expect, viewRun } from './fixtures';
import type { Page } from '@playwright/test';

const calls = (page: Page, command: string) =>
	page.evaluate(
		(command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command),
		command
	);

/** Gives the fixture runs stats, a running run and a run saved with its limits. */
async function richHistory(page: Page) {
	// A fixed afternoon, so day groups do not depend on when the test runs.
	const clock = new Date(2026, 9, 3, 15, 0, 0);
	await page.clock.setFixedTime(clock);
	await page.addInitScript((now: number) => {
		const fixture = (window as any).__uiFixture;
		const accounting = (spent: number, unpriced = 0) => ({
			spent_usd: spent,
			unpriced_calls: unpriced,
			breakdown: [{ provider: 'ollama_cloud', model: 'deepseek-v4.1-flash', calls: 3, unpriced_calls: unpriced }],
		});
		const patch: Record<string, any> = {
			table: {
				created_at: now - 2 * 3600,
				stats: JSON.stringify({ rows_found: 48, elapsed_secs: 109, accounting: accounting(0.031) }),
				config: JSON.stringify({ mode: 'table', stop: { target_row_count: 7, max_budget_usd: 2, max_duration_seconds: 300 } }),
			},
			images: { created_at: now - 26 * 3600, status: 'failed', stats: JSON.stringify({ image_count: 3 }) },
			links: { created_at: now - 40 * 86400, stats: JSON.stringify({ link_count: 46, accounting: accounting(0, 5) }) },
			research: { created_at: now - 600, turn_count: 2, stats: JSON.stringify({ steps: 0 }) },
		};
		for (const run of fixture.runs) Object.assign(run, patch[run.id]);
	}, Math.floor(clock.getTime() / 1000));
}

test('History lists runs as compact rows grouped by day', async ({ page }) => {
	await richHistory(page);
	await page.goto('/history');
	const groups = page.locator('.history-group h2');
	await expect(groups).toHaveText(['Today', 'Yesterday', /\w+ \d{4}/]);
	const table = page.locator('.history-row').filter({ hasText: 'Saved table research' });
	await expect(table).toContainText('48 rows');
	await expect(table).toContainText('1 m 49 s');
	await expect(table).toContainText('$0.03');
	await expect(table).toContainText('deepseek-v4.1-flash');
	await expect(table).toContainText('2 h ago');
	// Completed is the normal case and gets no badge; failures do.
	await expect(table.locator('.badge')).toHaveCount(0);
	await expect(page.locator('.history-row').filter({ hasText: 'Saved images research' })).toContainText('Failed');
	await expect(page.locator('.history-row').filter({ hasText: 'Saved links research' })).toContainText('cost unknown');
	await expect(page.locator('.history-row').filter({ hasText: 'Saved research research' })).toContainText('2 questions');
	expect((await table.boundingBox())!.height).toBeLessThan(72);
});

test('History searches, filters by type and status, and sorts', async ({ page }) => {
	await page.goto('/history');
	const types = page.getByRole('group', { name: 'Run type' });
	await expect(types.getByRole('button')).toHaveText(['All 4', 'Table 1', 'Images 1', 'Links 1', 'Research 1']);
	await types.getByRole('button', { name: /Links/ }).click();
	await expect(page.locator('.history-row')).toHaveCount(1);
	await expect(types.getByRole('button', { name: /Links/ })).toHaveAttribute('aria-pressed', 'true');
	await types.getByRole('button', { name: /All/ }).click();

	await page.getByRole('searchbox', { name: 'Search runs' }).fill('image');
	await expect(page.locator('.history-row')).toHaveCount(1);
	const last = (await calls(page, 'list_history')).at(-1);
	expect(last.args.filter.search).toBe('image');
	await page.getByRole('searchbox', { name: 'Search runs' }).fill('nothing like this');
	await expect(page.getByText('No runs match these filters.')).toBeVisible();
	await page.getByRole('button', { name: 'Clear filters' }).click();
	await expect(page.locator('.history-row')).toHaveCount(4);

	await page.getByLabel('Status').selectOption('failed');
	expect((await calls(page, 'list_history')).at(-1).args.filter.status).toBe('failed');
	await page.getByLabel('Status').selectOption('any');
	await page.getByLabel('Sort').selectOption('oldest');
	await expect(page.locator('.history-row').first()).toContainText('Saved table research');
});

test('Opening a run is a page of its own that Back and reload keep', async ({ page }) => {
	await page.setViewportSize({ width: 1200, height: 400 });
	await page.goto('/history');
	await page.locator('.history-list').evaluate((list) => (list.scrollTop = 60));
	await page.getByRole('link', { name: 'Saved links research', exact: true }).click();
	await expect(page).toHaveURL(/\/history\/links$/);
	await expect(page.getByRole('heading', { level: 1, name: 'Saved links research' })).toBeVisible();
	await page.reload();
	await expect(page.getByRole('heading', { level: 1, name: 'Saved links research' })).toBeVisible();
	await page.goBack();
	await page.goForward();
	await page.getByRole('button', { name: 'Back to History' }).click();
	await expect(page).toHaveURL(/\/history$/);
	await expect(page.locator('.history-row')).toHaveCount(4);
	await page.goto('/history/missing');
	await expect(page.getByText('This run no longer exists.')).toBeVisible();
});

test('The scroll position survives a visit to a run', async ({ page }) => {
	await page.addInitScript(() => {
		const fixture = (window as any).__uiFixture;
		const base = fixture.runs[0];
		fixture.runs = Array.from({ length: 30 }, (_, i) => ({ ...base, id: `t${i}`, query: `Run ${i}`, created_at: 5000 - i }));
	});
	await page.setViewportSize({ width: 1200, height: 500 });
	await page.goto('/history');
	const list = page.locator('.history-list');
	await list.evaluate((el) => (el.scrollTop = 600));
	await page.getByRole('link', { name: 'Run 20', exact: true }).click();
	await page.getByRole('button', { name: 'Back to History' }).click();
	await expect.poll(() => list.evaluate((el) => el.scrollTop)).toBeGreaterThan(500);
});

test('More runs load while scrolling', async ({ page }) => {
	await page.addInitScript(() => {
		const fixture = (window as any).__uiFixture;
		const base = fixture.runs[0];
		fixture.runs = Array.from({ length: 70 }, (_, i) => ({ ...base, id: `t${i}`, query: `Run ${i}`, created_at: 9000 - i }));
	});
	await page.goto('/history');
	await expect(page.locator('.history-row')).toHaveCount(50);
	await page.locator('.history-list').evaluate((el) => (el.scrollTop = el.scrollHeight));
	await expect(page.locator('.history-row')).toHaveCount(70);
});

test('Deleting a run can be undone', async ({ page }) => {
	await page.goto('/history');
	await page.getByRole('button', { name: 'Actions for Saved images research' }).click();
	await page.getByRole('menuitem', { name: 'Delete' }).click();
	await expect(page.locator('.history-row')).toHaveCount(3);
	await page.getByRole('button', { name: 'Undo' }).click();
	await expect(page.locator('.history-row')).toHaveCount(4);
	expect((await calls(page, 'restore_runs'))[0].args).toEqual({ runIds: ['images'] });
});

test('Selected runs can be deleted or exported together', async ({ page }) => {
	await page.goto('/history');
	await page.getByRole('checkbox', { name: 'Select Saved table research' }).check();
	await page.getByRole('checkbox', { name: 'Select Saved research research' }).check();
	const bar = page.getByRole('toolbar', { name: 'Selected runs' });
	await expect(bar).toContainText('2 selected');
	await bar.getByRole('button', { name: 'Export…' }).click();
	const dialog = page.getByRole('dialog', { name: 'Export runs' });
	await expect(dialog).toContainText('Research conversations are saved as Markdown.');
	await dialog.getByLabel('Format for tables, links and images').selectOption('xlsx');
	await dialog.getByRole('button', { name: 'Choose folder and export' }).click();
	await expect(dialog).toContainText('Exported 2 files to /tmp/exports');
	const [exported] = await calls(page, 'export_runs');
	expect(exported.args).toEqual({ runIds: ['research', 'table'], dir: '/tmp/exports', format: 'xlsx' });
	await dialog.getByRole('button', { name: 'Done' }).click();

	await bar.getByRole('button', { name: 'Delete' }).click();
	await expect(page.locator('.history-row')).toHaveCount(2);
	await expect(page.getByText('2 runs deleted')).toBeVisible();
	await expect(bar).toBeHidden();
});

test('Runs can be renamed and pinned', async ({ page }) => {
	await page.goto('/history');
	await page.getByRole('button', { name: 'Actions for Saved links research' }).click();
	await page.getByRole('menuitem', { name: 'Rename' }).click();
	const name = page.getByRole('textbox', { name: 'Run name' });
	await name.fill('Rust books');
	await name.press('Enter');
	await expect(page.getByRole('link', { name: 'Rust books', exact: true })).toBeVisible();
	// The query stays visible under the new name.
	await expect(page.locator('.history-row').filter({ hasText: 'Rust books' })).toContainText('Saved links research');

	await page.getByRole('button', { name: 'Actions for Rust books' }).click();
	await page.getByRole('menuitem', { name: 'Pin' }).click();
	await expect(page.locator('.history-group h2').first()).toHaveText('Pinned');
	await expect(page.locator('.history-group').first()).toContainText('Rust books');
});

test('Run again repeats the query with its limits and earlier schema', async ({ page }) => {
	await richHistory(page);
	await page.goto('/history');
	await page.getByRole('button', { name: 'Actions for Saved table research' }).click();
	await page.getByRole('menuitem', { name: 'Run again' }).click();
	await expect(page).toHaveURL(/\/$/);
	const [start] = await calls(page, 'start_run');
	expect(start.args).toMatchObject({
		query: 'Saved table research',
		runType: 'table',
		stopConditions: { target_row_count: 7, max_budget_usd: 2, max_duration_seconds: 300 },
	});
	expect(start.args.schema.map((c: any) => c.name)).toEqual(['Name', 'Details', 'Topics', 'Website', 'Count']);
});

test('Edit and run fills the query form', async ({ page }) => {
	await page.goto('/history');
	await page.getByRole('button', { name: 'Actions for Saved links research' }).click();
	await page.getByRole('menuitem', { name: 'Edit and run' }).click();
	await expect(page).toHaveURL(/\/$/);
	await expect(page.getByLabel('What would you like to find?')).toHaveValue('Saved links research');
	await expect(page.getByRole('button', { name: 'Links', exact: true })).toHaveAttribute('aria-pressed', 'true');
	expect(await calls(page, 'start_run')).toHaveLength(0);
});

test('Copy query copies the text', async ({ page }) => {
	await page.goto('/history');
	await page.getByRole('button', { name: 'Actions for Saved links research' }).click();
	await page.getByRole('menuitem', { name: 'Copy query' }).click();
	expect((await calls(page, 'copy_text')).at(-1).args).toMatchObject({ text: 'Saved links research' });
});

test('The run being answered opens live on the query page', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('button', { name: 'Research', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('Live question');
	await page.getByRole('button', { name: 'Start Research' }).click();
	await page.evaluate(() => {
		const fixture = (window as any).__uiFixture;
		fixture.runs.push({ ...fixture.runs[3], id: 'live', query: 'Live question', status: 'running', created_at: 9999 });
		fixture.emit('run:status_changed', { run_id: 'live', status: 'running' });
	});
	await page.getByRole('link', { name: 'History' }).click();
	const row = page.locator('.history-row').filter({ hasText: 'Live question' });
	await expect(row).toContainText('Running');
	await row.getByRole('link', { name: 'Live question', exact: true }).click();
	await expect(page).toHaveURL(/\/$/);
	await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible();
});

test('An empty history points to a new query', async ({ page }) => {
	await page.addInitScript(() => ((window as any).__uiFixture.runs = []));
	await page.goto('/history');
	await expect(page.getByText('No runs yet.')).toBeVisible();
	await page.getByRole('link', { name: 'New query' }).click();
	await expect(page).toHaveURL(/\/$/);
});

test('A saved run shares the live run header', async ({ page }) => {
	await viewRun(page, 0);
	const header = page.locator('.run-header');
	await expect(header.locator('.eyebrow')).toHaveText('Table');
	await expect(header.getByRole('heading', { level: 1 })).toHaveText('Saved table research');
	await expect(header.getByRole('button', { name: 'Export' })).toBeVisible();
	await header.getByRole('button', { name: 'More actions' }).click();
	await expect(page.getByRole('menuitem')).toHaveText(['Run again', 'Edit and run', 'Copy query', 'Rename', 'Pin', 'Delete']);
});
