import { test, expect, emit } from './fixtures';
import type { Page } from '@playwright/test';

const calls = (page: Page, command: string) =>
	page.evaluate(
		(command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command),
		command
	);

test('Missing provider keys block the run with a link to the right settings', async ({ page }) => {
	// Runs after the fixture script, so it edits the fixture's saved settings.
	await page.addInitScript(() => {
		const values = (window as any).__uiFixture.values;
		values.ollama_cloud_api_key = '';
		values.brave_api_key = '';
	});
	await page.goto('/');
	await page.getByLabel('What would you like to find?').fill('Find robots');
	const problems = page.getByRole('alert').filter({ hasText: 'Finish setup' });
	await expect(problems).toContainText('Ollama Cloud needs API key.');
	await expect(problems).toContainText('Brave Search needs an API key.');
	await expect(page.getByRole('button', { name: 'Build Table' })).toBeDisabled();
	await page.keyboard.press('Control+Enter');
	expect(await calls(page, 'start_run')).toHaveLength(0);
	await problems.getByRole('link', { name: 'Open settings' }).last().click();
	await expect(page).toHaveURL(/\/settings#settings-search$/);
	await expect(page.locator('#settings-search')).toBeInViewport();
});

test('Stop conditions are validated, sent explicitly and remembered for the next run', async ({
	page,
}) => {
	await page.goto('/');
	await page.getByLabel('What would you like to find?').fill('Find robots');
	await expect(page.getByRole('button', { name: /Stop conditions/ })).toContainText(
		'50 rows · $1.00 · 10 min'
	);
	await page.getByRole('button', { name: /Stop conditions/ }).click();
	await expect(page.getByText(/whichever limit it reaches first/)).toBeVisible();

	await page.getByLabel('Max cost (USD)').fill('');
	await page.getByLabel('Max duration (min)').fill('0');
	await expect(page.getByLabel('Max cost (USD)')).toHaveAttribute('aria-invalid', 'true');
	await expect(page.getByText('Enter an amount of at least $0.01.')).toBeVisible();
	await expect(page.getByRole('button', { name: /Stop conditions/ })).toContainText(
		'Check the values'
	);
	await page.getByRole('button', { name: 'Build Table' }).click();
	expect(await calls(page, 'start_run')).toHaveLength(0);

	await page.getByLabel('Target rows').fill('25');
	await page.getByLabel('Max cost (USD)').fill('2.5');
	await page.getByLabel('Max duration (min)').fill('15');
	await expect(page.getByRole('button', { name: /Stop conditions/ })).toContainText(
		'25 rows · $2.50 · 15 min'
	);
	await page.getByRole('button', { name: 'Build Table' }).click();
	const [start] = await calls(page, 'start_run');
	expect(start.args.stopConditions).toEqual({
		target_row_count: 25,
		max_budget_usd: 2.5,
		max_duration_seconds: 900,
	});
	await expect
		.poll(async () => (await calls(page, 'update_setting')).map((c: any) => [c.args.key, c.args.value]))
		.toEqual([
			['target_row_count', '25'],
			['max_budget_usd', '2.5'],
			['max_duration_seconds', '900'],
		]);

	// The remembered values are used after starting over.
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });
	await page.getByRole('button', { name: 'New query' }).click();
	await expect(page.getByLabel('What would you like to find?')).toBeFocused();
	await expect(page.getByRole('button', { name: /Stop conditions/ })).toContainText(
		'25 rows · $2.50 · 15 min'
	);
});

test('Examples, recent queries and Edit query refill the form', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('button', { name: 'Images', exact: true }).click();
	await page.getByRole('group', { name: 'Example queries' }).getByRole('button').first().click();
	await expect(page.getByLabel('What would you like to find?')).toHaveValue(
		'Brutalist libraries built after 1960'
	);
	await expect(page.getByRole('button', { name: 'Search Images' })).toBeEnabled();

	await page.getByRole('button', { name: 'Saved links research' }).click();
	await expect(page.getByLabel('What would you like to find?')).toHaveValue('Saved links research');
	await expect(page.getByRole('button', { name: 'Links', exact: true })).toHaveAttribute(
		'aria-pressed',
		'true'
	);

	await page.getByRole('button', { name: 'Find Links' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });
	await page.getByRole('button', { name: 'Edit query' }).click();
	await expect(page.getByLabel('What would you like to find?')).toHaveValue('Saved links research');
	await expect(page.getByRole('button', { name: 'Links', exact: true })).toHaveAttribute(
		'aria-pressed',
		'true'
	);
});

test('Cancel asks for confirmation and the run summary stays compact', async ({ page }) => {
	await page.setViewportSize({ width: 1200, height: 800 });
	await page.goto('/');
	await page.getByLabel('What would you like to find?').fill('Find robots');
	await page.getByRole('button', { name: 'Build Table' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await emit(page, 'run:progress_update', {
		run_id: 'live',
		stats: {
			rows_found: 20,
			pages_fetched: 4,
			pages_total: 10,
			queries_executed: 2,
			queries_total: 5,
			elapsed_secs: 90,
		},
	});
	await emit(page, 'run:log_entry', {
		run_id: 'live',
		level: 'INFO',
		role: 'fetcher',
		message: 'Fetching pages',
	});

	const progress = page.getByRole('progressbar', { name: 'Run progress' });
	await expect(progress).toHaveAttribute('aria-valuenow', '40');
	await expect(page.locator('.run-progress')).toContainText('40% of the rows target · 20 of 50 rows');
	await expect(page.locator('.progress-stats')).toContainText('Rows 20 / 50');

	// Status, statistics and cost share one line in a regular window.
	const operation = await page.locator('.current-operation').boundingBox();
	const cost = await page.locator('.cost-summary > summary').boundingBox();
	expect(Math.abs(operation!.y - cost!.y)).toBeLessThan(12);

	// Stages and the Logs panel tell what happens; there is no separate activity popover.
	await expect(page.getByText('Activity', { exact: true })).toHaveCount(0);

	await page.getByRole('button', { name: 'Cancel', exact: true }).click();
	const dialog = page.getByRole('dialog', { name: 'Cancel this run?' });
	await dialog.getByRole('button', { name: 'Keep running' }).click();
	expect(await calls(page, 'cancel_run')).toHaveLength(0);
	await page.getByRole('button', { name: 'Cancel', exact: true }).click();
	await dialog.getByRole('button', { name: 'Cancel run' }).click();
	await expect.poll(async () => (await calls(page, 'cancel_run')).length).toBe(1);
});
