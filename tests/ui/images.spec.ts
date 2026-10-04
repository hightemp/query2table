import { test, expect, viewRun } from './fixtures';
import type { Page } from '@playwright/test';

const svg = (color: string, w: number, h: number) =>
	'data:image/svg+xml,' +
	encodeURIComponent(
		`<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}"><rect width="${w}" height="${h}" fill="${color}"/></svg>`
	);
const images = [
	{
		id: 'wide',
		image_url: 'https://cdn.example/wide.svg',
		thumbnail_url: svg('#3366aa', 320, 180),
		title: 'Wide tower',
		source_url: 'https://news.example/towers',
		width: 1600,
		height: 900,
		relevance_score: 0.9,
	},
	{
		// Saved before the Brave mapping fix: page in image_url, bare domain in source_url.
		id: 'legacy',
		image_url: 'https://www.forbes.ru/gallery/1',
		thumbnail_url: svg('#aa6633', 300, 200),
		title: '',
		source_url: 'forbes.ru',
		width: 608,
		height: 382,
		relevance_score: 0.9,
	},
	{
		id: 'portrait',
		image_url: 'https://cdn.example/portrait.svg',
		thumbnail_url: 'https://broken.example/thumb.jpg',
		title: 'Portrait',
		source_url: 'https://xn--80aswg.xn--p1ai/photo',
		width: 900,
		height: 2400,
		relevance_score: 0.6,
	},
];

const originals: Record<string, string> = {
	'https://cdn.example/wide.svg': svg('#3366aa', 1600, 900),
	'https://cdn.example/portrait.svg': svg('#33aa66', 900, 2400),
};

const calls = (page: Page, command: string) =>
	page.evaluate(
		(command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command),
		command
	);

async function openGallery(page: Page) {
	await page.route('https://broken.example/**', (route) => route.fulfill({ status: 404 }));
	await page.route('https://cdn.example/**', (route) =>
		route.fulfill({
			contentType: 'image/svg+xml',
			body: decodeURIComponent(originals[route.request().url()].split(',')[1]),
		})
	);
	await page.addInitScript(({ images, originals }) => {
		const internals = (window as any).__TAURI_INTERNALS__;
		const original = internals.invoke;
		internals.invoke = async (command: string, args: any = {}) => {
			const fixture = (window as any).__uiFixture;
			switch (command) {
				case 'get_image_results':
					fixture.calls.push({ command, args });
					return images;
				case 'proxy_image':
					fixture.calls.push({ command, args });
					// Like the backend: pages are not images.
					if (originals[args.url]) return originals[args.url];
					if (!String(args.url).startsWith('data:image')) throw 'Not an image: text/html';
					return args.url;
				case 'plugin:dialog|open':
					fixture.calls.push({ command, args });
					return '/tmp/q2t-images';
				case 'save_image':
					fixture.calls.push({ command, args });
					return `${args.path}`;
				case 'save_images':
					fixture.calls.push({ command, args });
					return { saved: args.items.slice(1).map((i: any) => `/tmp/q2t-images/${i.name}.png`), failed: [[args.items[0].name, 'HTTP 404']] };
			}
			return original(command, args);
		};
	}, { images, originals });
	await page.setViewportSize({ width: 1200, height: 800 });
	await viewRun(page, 1);
	await expect(page.locator('.tile')).toHaveCount(3);
}

test('Preview opens reliably and explains thumbnails of old results', async ({ page }) => {
	await openGallery(page);
	await page.getByRole('button', { name: 'Preview Wide tower' }).click();
	const dialog = page.getByRole('dialog', { name: 'Wide tower' });
	await expect(dialog.locator('.preview-body img')).toHaveAttribute('src', /3366aa/);
	await expect(dialog).not.toContainText('could not be displayed');
	await expect(dialog.getByRole('link', { name: 'Open original' })).toBeVisible();

	await page.keyboard.press('ArrowRight');
	const legacy = page.getByRole('dialog', { name: 'forbes.ru' });
	await expect(legacy.locator('.preview-body img')).toHaveAttribute('src', /aa6633/);
	await expect(legacy).toContainText('the original file is not known');
	await expect(legacy.getByRole('link', { name: 'Source page' })).toHaveAttribute(
		'href',
		'https://www.forbes.ru/gallery/1'
	);
	await expect(legacy.getByRole('link', { name: 'Open original' })).toHaveCount(0);
	// The page URL is never sent to the image proxy for old results.
	expect((await calls(page, 'proxy_image')).map((c: any) => c.args.url)).not.toContain(
		'https://www.forbes.ru/gallery/1'
	);

	await legacy.getByRole('button', { name: 'Save image…' }).click();
	const [saved] = await calls(page, 'save_image');
	expect(saved.args.url).toContain('aa6633');
	expect(saved.args.fallbackUrl).toContain('aa6633');
	expect(saved.args.path).toBe('/tmp/query2table-fixture.csv');
	await expect(page.locator('.toast')).toContainText('Saved');
});

test('Rows keep image proportions and broken thumbnails fall back to the original', async ({
	page,
}) => {
	await openGallery(page);
	const box = async (name: string) =>
		(await page.getByRole('button', { name: `Preview ${name}` }).boundingBox())!;
	const wide = await box('Wide tower');
	expect(wide.width / wide.height).toBeCloseTo(1600 / 900, 1);
	const portrait = page.getByRole('button', { name: 'Preview Portrait' });
	await expect(portrait.locator('img')).toHaveAttribute('src', 'https://cdn.example/portrait.svg');
	await expect
		.poll(() => portrait.locator('img').evaluate((img: HTMLImageElement) => img.naturalWidth))
		.toBeGreaterThan(0);
	const tall = await box('Portrait');
	expect(tall.width / tall.height).toBeCloseTo(0.4, 1);
	await expect(page.locator('.tile').filter({ hasText: 'Portrait' })).toContainText('сайт.рф');
});

test('Images can be searched, filtered by site and size, and sorted', async ({ page }) => {
	await openGallery(page);
	await page.getByLabel('Minimum size').selectOption('1000');
	await expect(page.locator('.tile')).toHaveCount(2);
	await expect(page.getByRole('status').filter({ hasText: 'of 3 images' })).toHaveText('2 of 3 images');
	await page.getByLabel('Source site').selectOption('news.example');
	await expect(page.locator('.tile')).toHaveCount(1);
	await page.getByLabel('Search images').fill('nothing');
	await expect(page.getByText('No images match these filters.')).toBeVisible();
	await page.getByRole('button', { name: 'Reset filters' }).click();
	await expect(page.locator('.tile')).toHaveCount(3);
	await page.getByLabel('Sort images').selectOption('size');
	await expect(page.locator('.tile').first()).toContainText('Portrait');
	await page.getByLabel('Sort images').selectOption('found');
	await expect(page.locator('.tile').first()).toContainText('Wide tower');
	// Scores differ, so they are shown.
	await expect(page.locator('.tile').first()).toContainText('90% match');
});

test('Several images can be selected, copied and saved to a folder', async ({ page }) => {
	await openGallery(page);
	await page.getByRole('checkbox', { name: 'Select Wide tower' }).check({ force: true });
	await expect(page.getByRole('region', { name: 'Selected images' })).toContainText('1 selected');
	await page.getByRole('button', { name: 'Select all shown' }).click();
	await expect(page.getByRole('region', { name: 'Selected images' })).toContainText('3 selected');

	await page.getByRole('button', { name: 'Copy links' }).click();
	const copied = (await calls(page, 'copy_text')).at(-1).args.text.split('\n');
	expect(copied).toHaveLength(3);
	expect(copied).toContain('https://www.forbes.ru/gallery/1');

	await page.getByRole('button', { name: 'Save to folder…' }).click();
	const [batch] = await calls(page, 'save_images');
	expect(batch.args.directory).toBe('/tmp/q2t-images');
	expect(batch.args.items.map((i: any) => i.name)).toEqual(['Wide tower', 'forbes.ru', 'Portrait']);
	expect(batch.args.items[2].fallback_url).toBe('https://broken.example/thumb.jpg');
	await expect(page.getByRole('alert')).toContainText('Saved 2 of 3 images. Not saved: Wide tower');

	await page.getByRole('button', { name: 'List view' }).click();
	await expect(page.locator('.image-list tbody tr')).toHaveCount(3);
	await expect(page.locator('.image-list')).toContainText('1600 × 900');
	// The run is its own page, so a reload stays on it.
	await page.reload();
	await expect(page.locator('.image-list')).toBeVisible();
	await page.getByRole('button', { name: 'Clear selection' }).waitFor({ state: 'detached' });
});

test('Images have their own context menu instead of the webview one', async ({ page }) => {
	await openGallery(page);
	await page.getByRole('button', { name: 'Preview Wide tower' }).click({ button: 'right' });
	const menu = page.getByRole('menu', { name: 'Actions for Wide tower' });
	await expect(menu.getByRole('menuitem')).toHaveText([
		'Preview',
		'Open original',
		'Open source page',
		'Copy image link',
		'Copy page link',
		'Save image…',
		'Select',
	]);
	await expect(menu.getByRole('menuitem', { name: 'Preview' })).toBeFocused();
	await page.keyboard.press('ArrowDown');
	await page.keyboard.press('ArrowDown');
	await page.keyboard.press('ArrowDown');
	await expect(menu.getByRole('menuitem', { name: 'Copy image link' })).toBeFocused();
	await page.keyboard.press('Enter');
	await expect(menu).toHaveCount(0);
	expect((await calls(page, 'copy_text')).at(-1).args.text).toBe('https://cdn.example/wide.svg');

	await page.getByRole('button', { name: 'Preview Wide tower' }).click({ button: 'right' });
	await menu.getByRole('menuitem', { name: 'Open source page' }).click();
	expect((await calls(page, 'plugin:opener|open_url')).at(-1).args.url).toBe(
		'https://news.example/towers'
	);

	// Old records have no known image file, so only page actions are offered.
	const legacy = page.getByRole('button', { name: 'Preview forbes.ru' });
	await legacy.focus();
	await page.keyboard.press('Shift+F10');
	const legacyMenu = page.getByRole('menu', { name: 'Actions for forbes.ru' });
	await expect(legacyMenu.getByRole('menuitem')).toHaveText([
		'Preview',
		'Open source page',
		'Copy page link',
		'Save image…',
		'Select',
	]);
	await page.keyboard.press('Escape');
	await expect(legacyMenu).toHaveCount(0);
	await expect(legacy).toBeFocused();

	await legacy.click({ button: 'right' });
	await legacyMenu.getByRole('menuitem', { name: 'Select' }).click();
	await expect(page.getByRole('region', { name: 'Selected images' })).toContainText('1 selected');

	await page.getByRole('button', { name: 'List view' }).click();
	await page.locator('.image-list tbody tr').first().click({ button: 'right', position: { x: 400, y: 10 } });
	await expect(page.getByRole('menu')).toBeVisible();
	await page.mouse.click(5, 5);
	await expect(page.getByRole('menu')).toHaveCount(0);
});

test('The webview menu stays only in text fields and on selected text', async ({ page }) => {
	await openGallery(page);
	const prevented = (selector: string) =>
		page.locator(selector).first().evaluate((element) => {
			const event = new MouseEvent('contextmenu', { bubbles: true, cancelable: true });
			element.dispatchEvent(event);
			return event.defaultPrevented;
		});
	expect(await prevented('.sidebar')).toBe(true);
	expect(await prevented('h1')).toBe(true);
	expect(await prevented('input[type="search"]')).toBe(false);
	await page.locator('h1').selectText();
	expect(await prevented('h1')).toBe(false);
});
