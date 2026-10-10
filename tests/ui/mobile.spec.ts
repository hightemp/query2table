import { test, expect } from './fixtures';
import type { Page } from '@playwright/test';

// A typical phone held upright.
test.use({ viewport: { width: 390, height: 844 } });

/** Elements wider than the screen, which would make the page scroll sideways. */
async function overflowing(page: Page) {
	return page.evaluate(() => {
		const width = document.documentElement.clientWidth;
		return [...document.querySelectorAll('body *')]
			.filter((element) => {
				const box = element.getBoundingClientRect();
				if (!box.width || !box.height) return false;
				// Content inside its own scroller (tables, query lists) may be wider.
				for (let parent = element.parentElement; parent; parent = parent.parentElement) {
					const style = getComputedStyle(parent);
					if (['auto', 'scroll', 'hidden'].includes(style.overflowX) && parent.getBoundingClientRect().right <= width + 1)
						return false;
				}
				return box.right > width + 1 || box.left < -1;
			})
			.map((element) => `${element.tagName.toLowerCase()}.${[...element.classList].join('.')}`);
	});
}

test('On a phone the pages switch from a bottom tab bar', async ({ page }) => {
	await page.goto('/');
	const tabs = page.getByRole('navigation', { name: 'Main' });
	const box = (await tabs.boundingBox())!;
	expect(box.y + box.height).toBeGreaterThan(844 - 2);
	expect(box.width).toBeGreaterThan(380);
	// The bar keeps only what fits: no title, collapse button or theme switch (that is in Settings).
	await expect(page.getByRole('button', { name: 'Toggle sidebar' })).toBeHidden();
	await expect(page.getByRole('group', { name: 'Theme' })).toBeHidden();
	for (const [name, path] of [
		['History', '/history'],
		['Settings', '/settings'],
		['Query', '/'],
	]) {
		await tabs.getByRole('link', { name }).click();
		await expect(page).toHaveURL(new RegExp(`${path}$`));
		await expect(tabs.getByRole('link', { name })).toHaveAttribute('aria-current', 'page');
	}
});

test('The narrow window gets the same bar on the desktop', async ({ page }) => {
	await page.setViewportSize({ width: 600, height: 800 });
	await page.goto('/');
	const box = (await page.getByRole('navigation', { name: 'Main' }).boundingBox())!;
	expect(box.y).toBeGreaterThan(700);
	await page.setViewportSize({ width: 1000, height: 800 });
	const side = (await page.getByRole('navigation', { name: 'Main' }).boundingBox())!;
	expect(side.y).toBeLessThan(200);
});

for (const path of ['/', '/history', '/settings']) {
	test(`${path} fits the phone screen`, async ({ page }) => {
		await page.goto(path);
		await page.waitForTimeout(200);
		expect(await overflowing(page)).toEqual([]);
	});
}

test('The query box is wide enough to type in', async ({ page }) => {
	await page.goto('/');
	const box = (await page.getByRole('textbox', { name: /What would you like to find/ }).boundingBox())!;
	expect(box.width).toBeGreaterThan(320);
	// Modes sit two to a line, not one per line.
	const table = (await page.getByRole('button', { name: 'Table', exact: true }).boundingBox())!;
	const images = (await page.getByRole('button', { name: 'Images', exact: true }).boundingBox())!;
	expect(Math.abs(table.y - images.y)).toBeLessThan(4);
});
