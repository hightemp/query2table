import { test, expect, emit, viewRun } from './fixtures';

const calls = (page: import('@playwright/test').Page, command: string) =>
	page.evaluate((command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command), command);

test('A table can be built from the attached files only', async ({ page }) => {
	await page.goto('/');
	await page.getByLabel('What would you like to find?').fill('Make a table of the companies in these files');
	await page.getByRole('button', { name: 'Attach files' }).click();
	await expect(page.getByRole('list', { name: 'Attached files' }).getByRole('listitem')).toHaveCount(2);
	await page.getByRole('checkbox', { name: 'Also search the web' }).uncheck();
	await page.getByRole('button', { name: 'Build Table' }).click();
	const [start] = await calls(page, 'start_run');
	expect(start.args).toMatchObject({ runType: 'table', attachments: ['att-report.pdf', 'att-prices.xlsx'], sourceMode: 'files' });
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await expect(page.getByRole('list', { name: 'Attached files' }).getByRole('button')).toHaveText(['report.pdf', 'prices.xlsx']);
});

test('Links and images use the files as context only', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('button', { name: 'Links', exact: true }).click();
	await page.getByRole('button', { name: 'Attach files' }).click();
	await expect(page.getByRole('list', { name: 'Attached files' }).getByRole('listitem')).toHaveCount(2);
	await expect(page.getByRole('checkbox', { name: 'Also search the web' })).toHaveCount(0);
});

test('Rows cite places in files and open the passage', async ({ page }) => {
	await page.addInitScript(() => {
		const fixture = (window as any).__uiFixture;
		fixture.rowSources = [
			{ id: 'file-source', row_id: 'r', url: 'attachment://att-companies.csv?rows=2-3', title: 'companies.csv, rows 2–3', snippet: null },
		];
		fixture.runAttachments = [
			{
				turn_index: 0,
				attachment: {
					id: 'att-companies.csv', file_name: 'companies.csv', kind: 'spreadsheet', mime: 'text/csv', size: 120,
					status: 'ready', page_count: null, sheet_count: 1, char_count: 120, scanned_pages: 0, thumbnail: null, created_at: 0,
				},
			},
		];
	});
	await viewRun(page, 0);
	await expect(page.getByRole('list', { name: 'Attached files' }).getByRole('button')).toHaveText(['companies.csv']);
	await page.getByRole('button', { name: 'Open row 1 details', exact: true }).click();
	const source = page.getByRole('button', { name: 'companies.csv, rows 2–3' });
	await expect(source).toBeVisible();
	await source.click();
	await expect(page.getByRole('dialog', { name: 'companies.csv' })).toContainText('Passage of companies.csv');
});
