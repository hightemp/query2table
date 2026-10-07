import { test, expect, emit } from './fixtures';

const calls = (page: import('@playwright/test').Page, command: string) =>
	page.evaluate((command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command), command);

test('Files are attached with the button, described, and sent with the run', async ({ page }) => {
	await page.goto('/');
	await page.getByLabel('What would you like to find?').fill('Compare the plans in these files');
	await page.getByRole('button', { name: 'Attach files' }).click();
	const files = page.getByRole('list', { name: 'Attached files' });
	await expect(files.getByRole('listitem')).toHaveCount(2);
	await expect(files.getByRole('listitem').nth(0)).toContainText('report.pdf');
	await expect(files.getByRole('listitem').nth(0)).toContainText('12 pages · 48K characters');
	await expect(files.getByRole('listitem').nth(1)).toContainText('2 sheets');
	await expect(page.getByText('File contents will be sent to Ollama Cloud.')).toBeVisible();
	const [dialog] = await calls(page, 'plugin:dialog|open');
	expect(dialog.args.options.multiple).toBe(true);
	expect(dialog.args.options.filters[0].extensions).toContain('pdf');

	await files.getByRole('button', { name: 'Remove prices.xlsx' }).click();
	await expect(files.getByRole('listitem')).toHaveCount(1);
	expect((await calls(page, 'remove_attachment'))[0].args).toEqual({ id: 'att-prices.xlsx' });

	await page.getByRole('button', { name: 'Build Table' }).click();
	const [start] = await calls(page, 'start_run');
	expect(start.args.attachments).toEqual(['att-report.pdf']);
});

test('Refused files and scans explain themselves', async ({ page }) => {
	await page.addInitScript(() => {
		const fixture = (window as any).__uiFixture;
		fixture.dialogFiles = ['/docs/broken.docx', '/docs/scan.pdf'];
		fixture.values.vision_model = 'qwen-vl';
	});
	await page.goto('/');
	await page.getByRole('button', { name: 'Attach files' }).click();
	const files = page.getByRole('list', { name: 'Attached files' });
	await expect(files.getByRole('listitem').nth(0)).toContainText('The file is damaged and cannot be read.');
	await expect(files.getByRole('listitem').nth(1)).toContainText('scan');
	await files.getByRole('listitem').nth(1).locator('.warning').hover();
	await expect(page.getByRole('tooltip')).toContainText('model that sees images');
	await page.getByLabel('What would you like to find?').fill('Summarise');
	await page.getByRole('button', { name: 'Build Table' }).click();
	expect((await calls(page, 'start_run'))[0].args.attachments).toEqual(['att-scan.pdf']);
});

test('Files can be dropped on the window and pasted into the query', async ({ page }) => {
	await page.goto('/');
	const form = page.locator('.query-form');
	// The window starts listening for dropped files once the form is shown.
	await page.waitForFunction(() =>
		(window as any).__uiFixture.calls.some((c: any) => c.args?.event === 'tauri://drag-drop')
	);
	await emit(page, 'tauri://drag-enter', { paths: ['/docs/notes.pdf'], position: { x: 10, y: 10 } });
	await expect(page.getByText('Drop files to attach them')).toBeVisible();
	await emit(page, 'tauri://drag-drop', { paths: ['/docs/notes.pdf'], position: { x: 10, y: 10 } });
	await expect(page.getByText('Drop files to attach them')).toBeHidden();
	const files = page.getByRole('list', { name: 'Attached files' });
	await expect(files.getByRole('listitem')).toHaveCount(1);
	await expect(form).toContainText('notes.pdf');

	await page.getByLabel('What would you like to find?').evaluate((textarea) => {
		const data = new DataTransfer();
		data.items.add(new File([new Uint8Array([137, 80, 78, 71])], 'screenshot.png', { type: 'image/png' }));
		textarea.dispatchEvent(new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true }));
	});
	await expect(files.getByRole('listitem')).toHaveCount(2);
	await expect(files.getByRole('img', { name: 'screenshot.png' })).toBeVisible();
	const [pasted] = await calls(page, 'add_attachment_data');
	expect(pasted.options?.headers?.['x-file-name'] ?? 'screenshot.png').toBe('screenshot.png');
});

test('A draft keeps its files after a restart', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('button', { name: 'Attach files' }).click();
	await expect(page.getByRole('list', { name: 'Attached files' }).getByRole('listitem')).toHaveCount(2);
	await page.reload();
	await expect(page.getByRole('list', { name: 'Attached files' }).getByRole('listitem')).toHaveCount(2);
	await page.getByRole('list', { name: 'Attached files' }).getByRole('button', { name: 'Open report.pdf' }).click();
	expect((await calls(page, 'open_attachment'))[0].args).toEqual({ id: 'att-report.pdf' });
});
