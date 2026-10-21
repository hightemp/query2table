import { test, expect } from './fixtures';
import type { Page } from '@playwright/test';

const LOCALES = ['en', 'ru', 'de', 'fr', 'es', 'it', 'pt', 'pl', 'uk', 'tr', 'zh', 'ja', 'ko'];
const PAGES = ['/', '/history', '/usage', '/settings'];

/** Controls whose text does not fit (ellipsis on purpose is fine). */
const overflowing = (page: Page) =>
	page.evaluate(() =>
		[...document.querySelectorAll<HTMLElement>('button, [role="tab"], a.nav-item, .chip, label')]
			.filter((el) => el.offsetParent !== null && el.clientWidth > 0)
			.filter((el) => getComputedStyle(el).textOverflow !== 'ellipsis' && !el.closest('[data-allow-overflow]'))
			.filter((el) => el.scrollWidth > el.clientWidth + 1)
			.map((el) => `${el.tagName.toLowerCase()} "${el.textContent?.trim().slice(0, 40)}" ${el.scrollWidth}>${el.clientWidth}`)
	);

for (const locale of LOCALES) {
	test(`The ${locale} interface fits the screen`, async ({ page }) => {
		await page.addInitScript((locale) => {
			localStorage.setItem('q2t-language', locale);
			Object.assign((window as any).__uiFixture.values, { ui_language: locale });
		}, locale);
		for (const width of [1280, 390]) {
			await page.setViewportSize({ width, height: 900 });
			for (const path of PAGES) {
				await page.goto(path);
				await expect(page.locator('html')).toHaveAttribute('lang', locale);
				await expect(page.getByRole('heading', { level: 1 }).first()).toBeVisible();
				expect(await page.evaluate(() => document.documentElement.scrollWidth), `${path} at ${width}px`).toBe(width);
				expect(await overflowing(page), `${path} at ${width}px`).toEqual([]);
			}
		}
	});
}
