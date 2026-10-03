import { test, expect, viewRun, emit } from './fixtures';
import type { Page } from '@playwright/test';

const links = Array.from({ length: 13 }, (_, i) => ({
	id: `l${i}`,
	url: i < 8 ? `https://github.com/org/repo-${i}` : `https://blog.example/post-${i}`,
	title: `Resource ${i}`,
	description: `Description of resource ${i}. `.repeat(i === 0 ? 12 : 1),
	reason: i === 0 ? 'Lists the tools the query asks for' : '',
	relevance_score: i === 12 ? 0.2 : 0.95 - i * 0.02,
	low_relevance: i === 12,
	hidden: false,
	visited_at: i === 1 ? 1700000000 : null,
	created_at: 100 + i,
}));

const calls = (page: Page, command: string) =>
	page.evaluate(
		(command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command),
		command
	);

async function openLinks(page: Page) {
	await page.addInitScript((links) => {
		const internals = (window as any).__TAURI_INTERNALS__;
		const original = internals.invoke;
		internals.invoke = async (command: string, args: any = {}) => {
			if (command === 'get_link_results') {
				(window as any).__uiFixture.calls.push({ command, args });
				return links;
			}
			return original(command, args);
		};
	}, links);
	await page.setViewportSize({ width: 1200, height: 800 });
	await viewRun(page, 2);
	await expect(page.locator('.link-card')).toHaveCount(12);
}

test('Links show site, path, match tier and expandable descriptions', async ({ page }) => {
	await openLinks(page);
	const first = page.locator('.link-card').first();
	await expect(first).toContainText('github.com › org/repo-0');
	await expect(first.locator('.tier')).toHaveText('Best match');
	await expect(first.locator('.tier')).toHaveAttribute('title', '95% match — Lists the tools the query asks for');
	await expect(page.locator('.link-card').nth(1)).toContainText('Visited');
	const description = first.getByRole('button', { name: /Description of resource 0/ });
	await expect(description).toHaveAttribute('aria-expanded', 'false');
	await description.click();
	await expect(first).toContainText('Why it matches: Lists the tools the query asks for');
	await expect(page.getByRole('status').filter({ hasText: 'relevant links' })).toHaveText('12 relevant links');

	// The weak link is kept but collapsed.
	await page.getByRole('button', { name: 'Show 1 less relevant link' }).click();
	await expect(page.getByRole('region', { name: 'Less relevant links' })).toContainText('Resource 12');
	await expect(page.getByRole('region', { name: 'Less relevant links' }).locator('.tier')).toHaveText('Partial match');
});

test('Opening a link marks it visited, and links can be hidden with undo', async ({ page }) => {
	await openLinks(page);
	await page.getByRole('link', { name: 'Resource 2' }).click();
	await expect(page.locator('.link-card').nth(2)).toContainText('Visited');
	expect((await calls(page, 'set_link_visited')).at(-1).args).toEqual({ linkId: 'l2', visited: true });

	await page.locator('.link-card').nth(3).click({ button: 'right' });
	const menu = page.getByRole('menu');
	await expect(menu.getByRole('menuitem')).toHaveText([
		'Open link',
		'Copy link',
		'Copy as Markdown',
		'Mark as visited',
		'Select',
		'Hide link',
	]);
	await menu.getByRole('menuitem', { name: 'Hide link' }).click();
	await expect(page.locator('.link-card')).toHaveCount(11);
	expect((await calls(page, 'set_link_hidden')).at(-1).args).toEqual({ linkId: 'l3', hidden: true });
	await page.locator('.toast').getByRole('button', { name: 'Undo' }).click();
	await expect(page.locator('.link-card')).toHaveCount(12);
	expect((await calls(page, 'set_link_hidden')).at(-1).args).toEqual({ linkId: 'l3', hidden: false });

	await page.locator('.link-card').nth(3).click({ button: 'right' });
	await menu.getByRole('menuitem', { name: 'Hide link' }).click();
	await page.getByLabel('Show hidden (1)').check();
	await expect(page.locator('.link-card.hidden-link')).toContainText('Hidden from results and exports');
	await page.locator('.link-card.hidden-link').getByRole('button', { name: 'Show again' }).click();
	await expect(page.getByLabel(/Show hidden/)).toHaveCount(0);

	await page.locator('.link-card').nth(1).click({ button: 'right' });
	await menu.getByRole('menuitem', { name: 'Mark as not visited' }).click();
	await expect(page.locator('.link-card').nth(1)).not.toContainText('Visited');
});

test('Selected links can be copied in several formats, opened and hidden', async ({ page }) => {
	await openLinks(page);
	await page.getByRole('checkbox', { name: 'Select Resource 0' }).check({ force: true });
	await page.getByRole('checkbox', { name: 'Select Resource 4' }).check({ force: true });
	await page.getByRole('button', { name: /Copy selected/ }).click();
	await page.getByRole('menuitem', { name: 'Markdown list' }).click();
	expect((await calls(page, 'copy_text')).at(-1).args.text).toBe(
		'- [Resource 0](https://github.com/org/repo-0)\n- [Resource 4](https://github.com/org/repo-4)'
	);
	await page.getByRole('button', { name: 'Open in browser' }).click();
	await expect.poll(async () => (await calls(page, 'plugin:opener|open_url')).length).toBe(2);

	await page.getByRole('button', { name: 'Select all shown' }).click();
	await expect(page.getByRole('region', { name: 'Selected links' })).toContainText('12 selected');
	await page.getByRole('button', { name: 'Open in browser' }).click();
	const dialog = page.getByRole('dialog', { name: 'Open 12 links?' });
	await dialog.getByRole('button', { name: 'Cancel' }).click();
	expect(await calls(page, 'plugin:opener|open_url')).toHaveLength(2);

	await page.getByRole('button', { name: 'Hide', exact: true }).click();
	await expect(page.locator('.toast').filter({ hasText: 'links hidden' })).toContainText('12 links hidden.');
	await expect(page.getByText('No links passed the relevance threshold.')).toBeVisible();
});

test('Links can be filtered, grouped by site and sorted', async ({ page }) => {
	await openLinks(page);
	await page.getByLabel('Site', { exact: true }).selectOption('blog.example');
	await expect(page.locator('.link-card')).toHaveCount(4);
	await page.getByLabel('Site', { exact: true }).selectOption('');
	await page.getByLabel('Group by site').check();
	await expect(page.getByRole('region', { name: 'github.com' }).locator('.link-card')).toHaveCount(8);
	await expect(page.locator('.site-group h3').first()).toContainText('github.com8');
	await page.getByLabel('Group by site').uncheck();
	await page.getByLabel('Search links').fill('resource 7');
	await expect(page.locator('.link-card')).toHaveCount(1);
	await page.getByLabel('Search links').fill('');
	await page.getByLabel('Sort links').selectOption('site');
	await expect(page.locator('.link-card').first()).toContainText('blog.example');
});

test('Live links appear as they are scored and keep the reading position', async ({ page }) => {
	await page.setViewportSize({ width: 1200, height: 700 });
	await page.goto('/');
	await page.getByRole('button', { name: 'Links', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('Find links');
	await page.getByRole('button', { name: 'Find Links' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	for (let i = 0; i < 10; i++)
		await emit(page, 'run:link_added', {
			run_id: 'live',
			link_id: `live${i}`,
			url: `https://site${i}.example/page`,
			title: `Live ${i}`,
			description: 'Found while the run is going.',
			reason: 'Matches',
			relevance_score: 0.7 + i * 0.01,
			low_relevance: false,
		});
	await expect(page.locator('.link-card').first()).toContainText('Live 9');
	await expect(page.locator('.link-card.fresh').first()).toBeVisible();

	const list = page.locator('.link-list');
	await list.evaluate((e) => (e.scrollTop = 300));
	const reading = page.locator('[data-link-id="live5"]');
	const before = (await reading.boundingBox())!.y;
	await emit(page, 'run:link_added', {
		run_id: 'live',
		link_id: 'top',
		url: 'https://top.example/',
		title: 'New best link',
		description: 'Sorted in above.',
		reason: '',
		relevance_score: 0.99,
		low_relevance: false,
	});
	await expect(page.locator('.link-card').first()).toContainText('New best link');
	await expect.poll(async () => Math.round((await reading.boundingBox())!.y)).toBe(Math.round(before));
});

test('Every external link offers the app menu instead of the webview one', async ({ page }) => {
	await viewRun(page, 3);
	const answerLink = page.locator('.markdown a[href^="https://"]').first();
	await answerLink.click({ button: 'right' });
	await expect(page.getByRole('menu').getByRole('menuitem')).toHaveText([
		'Open link',
		'Copy link',
		'Copy as Markdown',
	]);
	await page.getByRole('menuitem', { name: 'Copy link' }).click();
	expect((await calls(page, 'copy_text')).at(-1).args.text).toBe('https://example.com/');

	await viewRun(page, 0);
	await page.locator('.data-row').first().getByRole('link').click({ button: 'right' });
	await expect(page.getByRole('menu')).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(page.getByRole('dialog')).toHaveCount(0);
});
