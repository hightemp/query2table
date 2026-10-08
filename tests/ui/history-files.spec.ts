import { test, expect } from './fixtures';

const calls = (page: import('@playwright/test').Page, command: string) =>
	page.evaluate((command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command), command);

const report = {
	id: 'att-report.pdf', file_name: 'report.pdf', kind: 'document', mime: 'application/pdf', size: 1000,
	status: 'ready', page_count: 3, sheet_count: null, char_count: 900, scanned_pages: 0, thumbnail: null, created_at: 0,
};

test.beforeEach(async ({ page }) => {
	await page.addInitScript((report) => {
		const fixture = (window as any).__uiFixture;
		const run = fixture.runs.find((r: any) => r.id === 'research');
		run.attachment_count = 1;
		run.config = JSON.stringify({ mode: 'research', source_mode: 'files', stop: { target_row_count: 12, max_budget_usd: 0.5, max_duration_seconds: 300 } });
		fixture.runAttachments = [{ turn_index: 0, attachment: report }];
	}, report);
});

test('History shows which runs have files', async ({ page }) => {
	await page.goto('/history');
	const row = page.locator('.history-row', { hasText: 'Saved research research' });
	await expect(row.getByLabel('1 attached file')).toBeVisible();
	await expect(page.locator('.history-row', { hasText: 'Saved table research' }).getByLabel(/attached file/)).toHaveCount(0);
});

test('Run again repeats the files and where the run looked', async ({ page }) => {
	await page.goto('/history');
	await page.getByRole('button', { name: 'Actions for Saved research research' }).click();
	await page.getByRole('menuitem', { name: 'Run again' }).click();
	await expect(page).toHaveURL(/\/$/);
	const [start] = await calls(page, 'start_run');
	expect(start.args).toMatchObject({ runType: 'research', attachments: ['att-report.pdf'], sourceMode: 'files' });
});

test('Edit and run puts the files back in the form', async ({ page }) => {
	await page.goto('/history');
	await page.getByRole('button', { name: 'Actions for Saved research research' }).click();
	await page.getByRole('menuitem', { name: 'Edit and run' }).click();
	await expect(page).toHaveURL(/\/$/);
	await expect(page.getByRole('list', { name: 'Attached files' }).getByRole('listitem')).toContainText('report.pdf');
	await expect(page.getByRole('checkbox', { name: 'Also search the web' })).not.toBeChecked();
	expect(await calls(page, 'start_run')).toHaveLength(0);
});
