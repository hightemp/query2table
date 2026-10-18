import { test, expect, viewRun } from './fixtures';
import type { Page } from '@playwright/test';

// On Android the file dialog returns content:// addresses instead of paths.
test.use({ userAgent: 'Mozilla/5.0 (Linux; Android 15; Pixel 8) AppleWebKit/537.36 Chrome/130 Mobile Safari/537.36' });

const calls = (page: Page, command: string) =>
	page.evaluate((command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command), command);

/** Answers the save dialog with an Android content address. */
async function saveDialogReturns(page: Page, address: string) {
	await page.addInitScript((address) => {
		const internals = (window as any).__TAURI_INTERNALS__;
		const invoke = internals.invoke;
		internals.invoke = (command: string, args: any, options: any) =>
			command === 'plugin:dialog|save' ? Promise.resolve(address) : invoke(command, args, options);
	}, address);
}

test('Files are attached from the phone with their names', async ({ page }) => {
	await page.goto('/');
	const chooser = page.waitForEvent('filechooser');
	await page.getByRole('button', { name: 'Attach files' }).click();
	const picker = await chooser;
	expect(picker.isMultiple()).toBe(true);
	// Playwright runs in Node, which has Buffer; the app's type check does not include Node types.
	const bytes = (text: string) => (globalThis as any).Buffer.from(text);
	await picker.setFiles([
		{ name: 'report.pdf', mimeType: 'application/pdf', buffer: bytes('%PDF-1.4') },
		{ name: 'prices.xlsx', mimeType: 'application/octet-stream', buffer: bytes('PK') },
	]);
	const files = page.getByRole('list', { name: 'Attached files' });
	await expect(files.getByRole('listitem')).toHaveCount(2);
	await expect(files.getByRole('listitem').nth(0)).toContainText('report.pdf');
	// Android's IPC carries JSON only: the content goes in base64 with the name.
	const sent = await calls(page, 'add_attachment_data');
	expect(sent.map((c: any) => c.args.name).sort()).toEqual(['prices.xlsx', 'report.pdf']);
	expect(sent.find((c: any) => c.args.name === 'report.pdf').args.data).toBe('JVBERi0xLjQ=');
	// The file dialog is not used: it cannot tell the names of files.
	expect(await calls(page, 'plugin:dialog|open')).toHaveLength(0);
});

test('An export saved through the phone dialog shows where it went', async ({ page }) => {
	await saveDialogReturns(
		page,
		'content://com.android.externalstorage.documents/document/primary%3ADownload%2Fquery2table-export.csv'
	);
	await viewRun(page, 0);
	await page.getByRole('button', { name: 'Export', exact: true }).click();
	const dialog = page.getByRole('dialog');
	await dialog.getByRole('button', { name: 'Export', exact: true }).click();
	await expect(dialog).toHaveAccessibleName('Export complete');
	await expect(dialog).toContainText('Download/query2table-export.csv');
	await expect(dialog).not.toContainText('content://');
});

test('An address that does not name the file is described in words', async ({ page }) => {
	await saveDialogReturns(page, 'content://com.android.providers.downloads.documents/document/msf%3A1000000033');
	await viewRun(page, 0);
	await page.getByRole('button', { name: 'Export', exact: true }).click();
	const dialog = page.getByRole('dialog');
	await dialog.getByRole('button', { name: 'Export', exact: true }).click();
	await expect(dialog).toContainText('the file you chose');
	await expect(dialog).not.toContainText('content://');
});
