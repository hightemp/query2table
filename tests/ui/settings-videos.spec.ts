import { test, expect, choose } from './fixtures';

test('Components show the video tools, update and remove them', async ({ page }) => {
	await page.goto('/settings');
	const block = page.locator('.components');
	await block.scrollIntoViewIfNeeded();
	await expect(block).toContainText('yt-dlp');
	await expect(block).toContainText('Not installed: downloaded the first time a video is read');
	await expect(block).toContainText('Version v2.9.8');

	await block.getByRole('button', { name: 'Check tool updates' }).click();
	await expect(block).toContainText('version v2.9.9 is available');

	// yt-dlp is downloaded with its progress shown.
	const ytdlp = block.getByRole('listitem').filter({ hasText: 'yt-dlp' });
	await ytdlp.getByRole('button', { name: 'Download' }).click();
	await expect(ytdlp).toContainText('Downloading… 37%');
	await expect(ytdlp.getByRole('progressbar')).toHaveAttribute('aria-valuenow', '37');
	await expect(ytdlp).toContainText('Version 2026.10.26');
	await expect(ytdlp.getByRole('progressbar')).toHaveCount(0);

	const deno = block.getByRole('listitem').filter({ hasText: 'Deno' });
	await deno.getByRole('button', { name: 'Remove' }).click();
	await expect(deno).toContainText('Not installed');
	await expect(deno.getByRole('button', { name: 'Download' })).toBeVisible();
});

test('A failed download says why', async ({ page }) => {
	await page.addInitScript(() => {
		addEventListener('DOMContentLoaded', () => ((window as any).__uiFixture.failInstall = 'network error: HTTP 404 for https://github.com/x'));
	});
	await page.goto('/settings');
	const block = page.locator('.components');
	await block.getByRole('listitem').filter({ hasText: 'yt-dlp' }).getByRole('button', { name: 'Download' }).click();
	await expect(block.getByRole('alert')).toHaveText('Failed: network error: HTTP 404 for https://github.com/x');
});

test('Videos can be turned off, and cookies come from a browser or a file with instructions', async ({ page }) => {
	await page.goto('/settings');
	await expect(page.getByRole('switch', { name: 'Videos' })).toBeChecked();
	const cookies = page.getByRole('combobox', { name: 'Browser cookies for videos' });
	await expect(cookies).toContainText('Off');
	await expect(page.getByLabel('cookies.txt file', { exact: true })).toHaveCount(0);

	await choose(cookies, 'cookies.txt file');
	await expect(page.getByLabel('cookies.txt file', { exact: true })).toBeVisible();

	const help = page.getByText('How to set up cookies');
	await help.click();
	await expect(page.getByText('Firefox works most reliably.', { exact: false })).toBeVisible();
	await expect(page.getByText('Get cookies.txt LOCALLY', { exact: false })).toBeVisible();
});

test('Phones have no browser cookies or own yt-dlp', async ({ page }) => {
	await page.addInitScript(() => Object.defineProperty(navigator, 'userAgent', { get: () => 'Mozilla/5.0 (Linux; Android 14) Mobile' }));
	await page.setViewportSize({ width: 390, height: 900 });
	await page.goto('/settings');
	await expect(page.getByRole('switch', { name: 'Videos' })).toBeAttached();
	await expect(page.getByRole('combobox', { name: 'Browser cookies for videos' })).toHaveCount(0);
	await expect(page.getByText('How to set up cookies')).toHaveCount(0);
});
