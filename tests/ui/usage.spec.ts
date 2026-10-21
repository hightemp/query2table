import { test, expect } from './fixtures';
import type { Page } from '@playwright/test';

const calls = (page: Page, command: string) =>
	page.evaluate((command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command), command);
const monthTitle = (date: Date) => date.toLocaleString('en-US', { month: 'long', year: 'numeric' });
const monthStart = (page: Page, back = 0) =>
	page.evaluate((back) => {
		const now = new Date();
		return new Date(now.getFullYear(), now.getMonth() - back, 1).getTime() / 1000;
	}, back);

test('Statistics show the period’s totals, chart and breakdowns', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'Statistics' }).click();
	await expect(page).toHaveURL(/\/usage$/);
	await expect(page.getByRole('heading', { name: 'Statistics', level: 1 })).toBeVisible();
	await expect(page.getByRole('button', { name: 'Month', exact: true })).toHaveAttribute('aria-pressed', 'true');
	await expect(page.getByRole('heading', { name: monthTitle(new Date()) })).toBeVisible();
	const [request] = await calls(page, 'get_usage');
	expect(request.args.request.from).toBe(await monthStart(page));
	expect(request.args.request.previous_from).toBe(await monthStart(page, 1));

	const cost = page.getByRole('group', { name: 'Cost' });
	await expect(cost).toContainText('$1.25');
	await expect(cost).toContainText('+25% vs previous period');
	await expect(cost).toContainText('2 calls without a price');
	await expect(page.getByRole('group', { name: 'Tokens' })).toContainText('12,000');
	await expect(page.getByRole('group', { name: 'Searches' })).toContainText('Brave 3 · Serper 1');
	await expect(page.getByRole('group', { name: 'Downloaded from the web' })).toContainText('2 MB');
	await expect(page.getByRole('group', { name: 'Exchanged with models' })).toContainText('1.5 MB sent · 512 KB received');

	// Calls without a price get one from today's rates.
	await cost.getByRole('button', { name: 'Recalculate' }).click();
	await expect(page.getByText('Priced 2 calls')).toBeVisible();
	expect(await calls(page, 'reprice_usage')).toHaveLength(1);
	expect(await calls(page, 'get_usage')).toHaveLength(2);

	const chart = page.getByRole('img', { name: /^Cost by day/ });
	await expect(chart.locator('svg').first()).toBeVisible();
	await page.getByRole('combobox', { name: 'Chart' }).click();
	await page.getByRole('option', { name: 'Tokens' }).click();
	await expect(page.getByRole('img', { name: /^Tokens by day/ }).locator('svg').first()).toBeVisible();

	const models = page.getByRole('table', { name: 'By model' });
	await expect(models.getByRole('row', { name: /openai\/gpt-4\.1-mini/ })).toContainText('$1.00');
	await expect(models.getByRole('row', { name: /claude-sonnet-5-5/ })).toContainText('Anthropic');
	await expect(page.getByRole('table', { name: 'By mode' }).getByRole('row', { name: /Research/ })).toContainText('$0.25');
	const runs = page.getByRole('table', { name: 'Most expensive runs' });
	await expect(runs.getByRole('link', { name: 'Robot channels' })).toHaveAttribute('href', '/history/table');
	await expect(runs.getByRole('row', { name: /Old idea/ })).toContainText('deleted');
	await expect(runs.getByRole('row', { name: /Old idea/ }).getByRole('link')).toHaveCount(0);

	const disk = page.getByRole('region', { name: 'Disk' });
	await expect(disk).toContainText('Database');
	await expect(disk).toContainText('5 MB');
	await disk.getByRole('button', { name: 'Open data folder' }).click();
	expect((await calls(page, 'open_app_folder')).at(-1).args).toEqual({ location: 'data' });
});

test('Periods step back and forward, by year and over all time', async ({ page }) => {
	await page.goto('/usage');
	const next = page.getByRole('button', { name: 'Next period' });
	await expect(next).toBeDisabled();
	await page.getByRole('button', { name: 'Previous period' }).click();
	const now = new Date();
	await expect(page.getByRole('heading', { name: monthTitle(new Date(now.getFullYear(), now.getMonth() - 1, 1)) })).toBeVisible();
	expect((await calls(page, 'get_usage')).at(-1).args.request.from).toBe(await monthStart(page, 1));
	await expect(next).toBeEnabled();
	await next.click();
	await expect(page.getByRole('heading', { name: monthTitle(now) })).toBeVisible();

	await page.getByRole('button', { name: 'Year', exact: true }).click();
	await expect(page.getByRole('heading', { name: String(now.getFullYear()) })).toBeVisible();
	expect((await calls(page, 'get_usage')).at(-1).args.request.buckets).toHaveLength(12);
	await expect(page.getByRole('img', { name: /^Cost by month/ })).toBeVisible();

	await page.getByRole('button', { name: 'All time', exact: true }).click();
	await expect(page.getByRole('heading', { name: 'All time' })).toBeVisible();
	await expect(page.getByRole('button', { name: 'Previous period' })).toHaveCount(0);
	const all = (await calls(page, 'get_usage')).at(-1).args.request;
	// The fixture's first call was in January of last year.
	expect(all.from).toBe(await page.evaluate(() => new Date(new Date().getFullYear() - 1, 0, 1).getTime() / 1000));
	await expect(page.getByRole('group', { name: 'Cost' })).not.toContainText('previous period');
});

test('The monthly budget shows its progress and warns on the query page', async ({ page }) => {
	await page.addInitScript(() => Object.assign((window as any).__uiFixture.budget, { limit_usd: 10, spent_usd: 12 }));
	await page.goto('/usage');
	const budget = page.getByRole('region', { name: 'Monthly budget' });
	await expect(budget).toContainText('$12.00 of $10.00');
	await expect(budget.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '100');

	await page.goto('/');
	const notice = page.getByRole('alert').filter({ hasText: 'This month’s budget is exceeded' });
	await expect(notice).toContainText('$12.00 of $10.00');
	await notice.getByRole('link', { name: 'Statistics' }).click();
	await expect(page).toHaveURL(/\/usage$/);
});

test('The monthly budget is a setting; without it the page says how to set one', async ({ page }) => {
	await page.goto('/usage');
	await expect(page.getByRole('region', { name: 'Monthly budget' })).toContainText('No monthly budget');
	await page.goto('/settings');
	const field = page.getByLabel('Monthly budget');
	await field.fill('25');
	await page.getByRole('button', { name: 'Save', exact: true }).click();
	await expect(page.getByText('Changes saved', { exact: true })).toBeVisible();
	const [saved] = await calls(page, 'update_settings');
	expect(saved.args.values.monthly_budget_usd).toBe('25');
});

test('Statistics export as CSV and can be cleared', async ({ page }) => {
	await page.goto('/usage');
	await page.getByRole('button', { name: 'Export' }).click();
	await page.getByRole('menuitem', { name: 'Totals by date (CSV)' }).click();
	await expect(page.getByText(/^Saved /)).toBeVisible();
	const [dialog] = await calls(page, 'plugin:dialog|save');
	const now = new Date();
	expect(dialog.args.options.defaultPath).toBe(`statistics-${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}.csv`);
	const [saved] = await calls(page, 'save_file_data');
	const text = atob(saved.args.data);
	expect(text.split('\n')[0]).toMatch(/^period,cost_usd,/);
	expect(text.split('\n')[1]).toMatch(new RegExp(`^${now.getFullYear()}-\\d\\d-01,0.1,`));

	await page.getByRole('button', { name: 'Export' }).click();
	await page.getByRole('menuitem', { name: 'By model (CSV)' }).click();
	await expect.poll(async () => (await calls(page, 'save_file_data')).length).toBe(2);
	expect(atob((await calls(page, 'save_file_data'))[1].args.data)).toContain('openrouter,openai/gpt-4.1-mini,');

	await page.getByRole('button', { name: 'Clear statistics' }).click();
	const confirm = page.getByRole('dialog', { name: 'Clear statistics?' });
	await expect(confirm).toContainText('Runs stay in History.');
	await confirm.getByRole('button', { name: 'Clear', exact: true }).click();
	await expect(page.getByText('Statistics cleared')).toBeVisible();
	expect(await calls(page, 'clear_usage')).toHaveLength(1);
	await expect(page.getByText('No usage in this period.')).toBeVisible();
});

test('Statistics fit a phone screen', async ({ page }) => {
	await page.setViewportSize({ width: 390, height: 800 });
	await page.goto('/usage');
	await expect(page.getByRole('group', { name: 'Cost' })).toBeVisible();
	await expect(page.getByRole('navigation', { name: 'Main' }).getByRole('link', { name: 'Statistics' })).toBeVisible();
	expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(390);
});
