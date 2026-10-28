import { test, expect, emit } from './fixtures';
import type { Page } from '@playwright/test';

const VIDEO = 'https://www.youtube.com/watch?v=abc';
const THUMBNAIL = 'https://i.ytimg.com/vi/abc/hq.jpg';
const answer = [
	'The talk explains it well:',
	'',
	`[![Async Rust in depth](${THUMBNAIL})](${VIDEO})`,
	'',
	`The key point comes at [12:34](${VIDEO}&t=754s).`,
	'',
	'[![Not kept](https://i.ytimg.com/vi/other/hq.jpg)](https://vimeo.com/42)',
].join('\n');

async function answerWithVideos(page: Page, width = 1000) {
	const web: string[] = [];
	page.on('request', (request) => {
		if (request.url().includes('ytimg.com')) web.push(request.url());
	});
	await page.setViewportSize({ width, height: 900 });
	await page.goto('/');
	await page.getByRole('button', { name: 'Research', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('Explain async Rust');
	await page.getByRole('button', { name: 'Start Research' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await emit(page, 'run:research_step', { run_id: 'live', step_id: 'a', turn_index: 0, step_index: 0, step_type: 'video_search', content: 'async rust (youtube)', url: null });
	await emit(page, 'run:research_step', { run_id: 'live', step_id: 'b', turn_index: 0, step_index: 1, step_type: 'video', content: 'Async Rust in depth', url: VIDEO });
	await emit(page, 'run:research_step', { run_id: 'live', step_id: 'c', turn_index: 0, step_index: 2, step_type: 'video', content: 'Async Rust in depth · from 12:00', url: VIDEO });
	await page.evaluate(
		([url, video]) => {
			(window as any).__uiFixture.researchImages = [
				{ url, source_url: video, title: 'Async Rust in depth', width: 480, height: 270, video_url: video, channel: 'Rust Talks', duration: 3723, site: 'Youtube' },
			];
		},
		[THUMBNAIL, VIDEO]
	);
	await emit(page, 'run:research_answer', { run_id: 'live', turn_index: 0, markdown: answer, follow_ups: [] });
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });
	return web;
}

test('Research answers show videos as cards with kept thumbnails', async ({ page }) => {
	const web = await answerWithVideos(page);
	const panel = page.getByRole('tabpanel', { name: 'Answer' });

	const card = panel.getByRole('link', { name: 'Open video: Async Rust in depth' });
	await expect(card).toHaveAttribute('href', VIDEO);
	await expect(card).toContainText('Async Rust in depth');
	await expect(card).toContainText('Rust Talks · youtube.com');
	await expect(card).toContainText('1:02:03');
	const thumbnail = card.locator('img');
	await expect(thumbnail).toBeVisible();
	expect(await thumbnail.getAttribute('src')).toMatch(/^data:image\//);

	// A card whose thumbnail was not kept still links to its video, without a picture.
	const bare = panel.getByRole('link', { name: 'Open video: Not kept' });
	await expect(bare).toHaveAttribute('href', 'https://vimeo.com/42');
	await expect(bare.locator('img')).toHaveCount(0);
	await expect(bare).toContainText('vimeo.com');
	expect(web).toEqual([]);

	await page.getByRole('tab', { name: /Sources/ }).click();
	const sources = page.getByRole('tabpanel', { name: /Sources/ });
	await expect(sources.getByRole('heading', { name: 'Videos' })).toBeVisible();
	const items = sources.getByRole('listitem');
	await expect(items).toHaveCount(2);
	await expect(items.nth(0)).toContainText('Async Rust in depth');
	await expect(items.nth(0)).toContainText('Rust Talks · youtube.com');
	await expect(items.nth(0)).toContainText('1:02:03');
	await expect(items.nth(0)).toContainText('Cited');
	await expect(items.nth(0)).toContainText('Read');
	await expect(items.nth(0).locator('img')).toBeVisible();
	await expect(items.nth(1)).toContainText('Not kept');

	await page.getByRole('tab', { name: /Activity/ }).click();
	const activity = page.getByRole('tabpanel', { name: /Activity/ });
	await expect(activity).toContainText('Video search');
	await expect(activity).toContainText('async rust (youtube)');
	await expect(activity).toContainText('Watched video');
	await expect(activity).toContainText('Async Rust in depth · from 12:00');

	await page.getByRole('tab', { name: 'Answer' }).click();
	await page.getByRole('button', { name: 'Copy answer' }).click();
	await page.getByRole('menuitem', { name: 'Copy Markdown' }).click();
	expect(await page.evaluate(() => (window as any).__uiFixture.clipboard)).toContain(`[![Async Rust in depth](${THUMBNAIL})](${VIDEO})`);
});

test('Video cards fit a phone screen', async ({ page }) => {
	await answerWithVideos(page, 390);
	const card = page.getByRole('link', { name: 'Open video: Async Rust in depth' });
	await expect(card).toBeVisible();
	const box = await card.boundingBox();
	expect(box!.width).toBeLessThanOrEqual(390 - 16);
	expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(390);
});
