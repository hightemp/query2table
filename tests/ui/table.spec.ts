import { test, expect, viewRun, emit } from './fixtures';
import type { Page } from '@playwright/test';

const calls = (page: Page, command: string) =>
	page.evaluate(
		(command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command),
		command
	);

const columns = [
	{ name: 'company_name', type: 'text', description: 'Legal name', required: true },
	{ name: 'funding_usd', type: 'number', description: 'Total funding', required: false },
	{ name: 'is_public', type: 'boolean', description: '', required: false },
	{ name: 'founded', type: 'date', description: '', required: false },
	{ name: 'ceo_name', type: 'text', description: '', required: false },
	{ name: 'hq_city', type: 'text', description: '', required: false },
	{ name: 'industry', type: 'text', description: '', required: false },
	{ name: 'website', type: 'url', description: '', required: false },
];

async function liveTable(page: Page) {
	await page.setViewportSize({ width: 1000, height: 700 });
	await page.goto('/');
	await page.getByLabel('What would you like to find?').fill('Find companies');
	await page.getByRole('button', { name: 'Build Table' }).click();
	await emit(page, 'run:schema_proposed', { run_id: 'live', columns });
	await page.getByRole('button', { name: 'Confirm Schema' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	const rows = [
		['Alpha', 500000000, true, '2015-04-02', 'Ann Lee', 0.95],
		['Beta', 'Unknown', 'no', '2019', 'Unknown', 0.4],
		['Gamma', '1,200', 'yes', 'Spring 2020', 'Bo Chen', 0.8],
	];
	for (const [i, [name, funding, pub, founded, ceo, confidence]] of rows.entries())
		await emit(page, 'run:row_added', {
			run_id: 'live',
			row_id: `r${i}`,
			data: {
				company_name: name,
				funding_usd: funding,
				is_public: pub,
				founded,
				ceo_name: ceo,
				hq_city: 'Berlin',
				industry: 'Software',
				website: `https://${String(name).toLowerCase()}.example`,
			},
			confidence,
		});
	await expect(page.locator('.data-row')).toHaveCount(3);
}

test('Cells are formatted by type and headers are readable', async ({ page }) => {
	await liveTable(page);
	await expect(page.getByRole('button', { name: 'CEO name', exact: true })).toBeVisible();
	await expect(page.getByRole('button', { name: 'Company name', exact: true })).toHaveAttribute(
		'title',
		'Legal name'
	);
	const alpha = page.locator('.data-row').nth(0);
	await expect(alpha.locator('td.numeric')).toHaveText((500000000).toLocaleString());
	expect(await alpha.locator('td.numeric').evaluate((td) => getComputedStyle(td).textAlign)).toBe(
		'right'
	);
	await expect(alpha.locator('.flag.yes')).toHaveText('Yes');
	const beta = page.locator('.data-row').nth(1);
	await expect(beta.locator('td.missing').first()).toHaveText('Unknown');
	await expect(beta.locator('.flag')).toHaveText('No');
	await expect(beta).toHaveClass(/low-confidence/);
	await expect(beta.locator('.evidence')).toContainText('40%');
	await expect(beta.locator('.evidence')).toContainText('1 source');
	await expect(page.locator('.data-row').nth(2).locator('td').nth(4)).toContainText('Yes');
	await expect(page.locator('.data-row').nth(2)).toContainText('Spring 2020');
});

test('Numeric sorting keeps missing values last in both directions', async ({ page }) => {
	await liveTable(page);
	const names = () => page.locator('.data-row td.first-column').allTextContents();
	await page.getByRole('button', { name: 'Funding USD', exact: true }).click();
	expect((await names()).map((t) => t.trim())).toEqual(['Gamma', 'Alpha', 'Beta']);
	await page.getByRole('button', { name: 'Funding USD', exact: true }).click();
	expect((await names()).map((t) => t.trim())).toEqual(['Alpha', 'Gamma', 'Beta']);
});

test('Details stay reachable in wide tables: sticky columns, row click and navigation', async ({
	page,
}) => {
	await liveTable(page);
	const scroll = page.locator('.table-scroll');
	await scroll.evaluate((e) => (e.scrollLeft = e.scrollWidth));
	const details = page.getByRole('button', { name: 'Open row 1 details', exact: true });
	await expect(details).toBeInViewport();
	const nameCell = page.locator('.data-row').first().locator('td.first-column');
	const box = (await nameCell.boundingBox())!;
	const wrap = (await page.locator('.results-table-wrap').boundingBox())!;
	expect(box.x).toBeLessThan(wrap.x + 60);

	// Clicking a cell opens the row; a link inside the row does not.
	await page.locator('.data-row').nth(1).locator('td').last().click({ position: { x: 4, y: 4 } });
	const dialog = page.getByRole('dialog', { name: 'Row Details' });
	await expect(dialog).toContainText('Row 2 of 3');
	await expect(dialog).toContainText('Beta');
	await page.keyboard.press('j');
	await expect(dialog).toContainText('Row 3 of 3');
	await expect(dialog).toContainText('Gamma');
	await expect(dialog.getByRole('button', { name: 'Next' })).toBeDisabled();
	await dialog.getByRole('button', { name: 'Previous' }).click();
	await page.keyboard.press('Alt+ArrowUp');
	await expect(dialog).toContainText('Row 1 of 3');
	await expect(dialog).toContainText('Alpha');
	await page.keyboard.press('Escape');

	await page.locator('.data-row').first().getByRole('link').click();
	await expect(dialog).toHaveCount(0);
});

test('Columns can be hidden, resized and shown compactly; rows copy as text', async ({ page }) => {
	await liveTable(page);
	await page.getByRole('button', { name: 'Columns' }).click();
	await page.getByRole('group', { name: 'Visible columns' }).getByLabel('Industry').uncheck();
	await expect(page.getByRole('button', { name: 'Industry', exact: true })).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Columns (7/8)' })).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(page.getByRole('group', { name: 'Visible columns' })).toHaveCount(0);

	const header = page.locator('th').filter({ hasText: 'CEO name' });
	const before = (await header.boundingBox())!.width;
	await page.getByRole('separator', { name: 'Resize CEO name column' }).focus();
	await page.keyboard.press('ArrowRight');
	await page.keyboard.press('ArrowRight');
	await expect.poll(async () => (await header.boundingBox())!.width).toBe(before + 40);

	const rowHeight = async () => (await page.locator('.data-row').first().boundingBox())!.height;
	expect(await rowHeight()).toBeGreaterThan(60);
	await page.getByRole('button', { name: 'Compact rows' }).click();
	await expect.poll(rowHeight).toBeLessThan(40);
	await page.reload();
	await expect(page.getByRole('button', { name: 'Compact rows' })).toHaveCount(0);

	await viewRun(page, 0);
	await expect(page.getByRole('button', { name: 'Compact rows' })).toHaveAttribute(
		'aria-pressed',
		'true'
	);
	await page.getByRole('button', { name: 'Open row 1 details', exact: true }).focus();
	await page.keyboard.press('Control+c');
	const copied = await calls(page, 'copy_text');
	expect(copied.at(-1).args.text.split('\t')[0]).toBe('Robot channel 0000');
	await expect(page.locator('.toast')).toContainText('Row copied');
});

test('Saved runs show source counts and search can target one column', async ({ page }) => {
	await viewRun(page, 0);
	await expect(page.locator('.data-row').first().locator('.evidence')).toContainText('1 source');
	await expect(page.locator('.data-row').nth(1).locator('.evidence')).toContainText('2 sources');
	await page.getByLabel('Search in column').selectOption('Count');
	await page.getByRole('searchbox', { name: 'Search results' }).fill('999');
	await expect(page.locator('.data-row')).toHaveCount(1);
	await page.getByLabel('Search in column').selectOption('Name');
	await expect(page.locator('.data-row')).toHaveCount(1);
	await page.getByRole('searchbox', { name: 'Search results' }).fill('channel 09');
	await expect(page.locator('.data-row').first()).toContainText('Robot channel 09');
});
