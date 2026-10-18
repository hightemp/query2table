import { test, expect, emit } from './fixtures';
import type { Page } from '@playwright/test';

const answer = [
	'Red pandas live in mountain forests.',
	'',
	'![Red panda](https://img.example.com/panda.jpg)',
	'',
	'Some pictures could not be kept: ![Lost picture](https://img.example.com/lost.jpg)',
].join('\n');

async function answerWithImages(page: Page, width = 1000) {
	const web: string[] = [];
	page.on('request', (request) => {
		if (request.url().includes('img.example.com')) web.push(request.url());
	});
	await page.setViewportSize({ width, height: 800 });
	await page.goto('/');
	await page.getByRole('button', { name: 'Research', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('Show a red panda');
	await page.getByRole('button', { name: 'Start Research' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await emit(page, 'run:research_step', { run_id: 'live', step_id: 'a', turn_index: 0, step_index: 0, step_type: 'image_search', content: 'red panda', url: null });
	await emit(page, 'run:research_step', {
		run_id: 'live',
		step_id: 'b',
		turn_index: 0,
		step_index: 1,
		step_type: 'view_image',
		content: 'Red panda in a tree',
		url: 'https://img.example.com/panda.jpg',
	});
	await page.evaluate(() => {
		(window as any).__uiFixture.researchImages = [
			{ url: 'https://img.example.com/panda.jpg', source_url: 'https://zoo.example/pandas', title: 'Red panda in a tree', width: 40, height: 30 },
		];
	});
	await emit(page, 'run:research_answer', { run_id: 'live', turn_index: 0, markdown: answer, follow_ups: [] });
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });
	return web;
}

test('Research answers show kept pictures and never load them from the web', async ({ page }) => {
	const web = await answerWithImages(page);
	const panel = page.getByRole('tabpanel', { name: 'Answer' });
	const picture = panel.getByRole('img', { name: 'Red panda' });
	await expect(picture).toBeVisible();
	expect(await picture.getAttribute('src')).toMatch(/^data:image\//);
	const caption = panel.locator('.answer-image-caption').first();
	await expect(caption).toHaveText('Red panda · zoo.example');
	await expect(caption.getByRole('link', { name: 'zoo.example' })).toHaveAttribute('href', 'https://zoo.example/pandas');
	// A picture that was not kept is a link to its address, not a picture.
	await expect(panel.getByRole('link', { name: 'Lost picture' })).toHaveAttribute('href', 'https://img.example.com/lost.jpg');
	await expect(panel.locator('img')).toHaveCount(1);
	expect(web).toEqual([]);

	await panel.getByRole('button', { name: 'Open image: Red panda' }).click();
	const dialog = page.getByRole('dialog', { name: 'Red panda' });
	await expect(dialog.getByRole('img', { name: 'Red panda' })).toBeVisible();
	await expect(dialog).toContainText('zoo.example · 40 × 30');
	await expect(dialog.getByRole('link', { name: 'Source page' })).toHaveAttribute('href', 'https://zoo.example/pandas');
	await dialog.getByRole('button', { name: 'Close' }).click();
	await expect(dialog).toBeHidden();

	await page.getByRole('tab', { name: /Sources/ }).click();
	const sources = page.getByRole('tabpanel', { name: /Sources/ });
	await expect(sources.getByRole('listitem')).toHaveCount(1);
	await expect(sources.getByRole('listitem')).toContainText('zoo.example');
	await expect(sources.getByRole('listitem')).toContainText('1 image');

	await page.getByRole('tab', { name: /Activity/ }).click();
	const activity = page.getByRole('tabpanel', { name: /Activity/ });
	await expect(activity).toContainText('Image search');
	await expect(activity).toContainText('Looked at image');
	await expect(activity).toContainText('Red panda in a tree');

	await page.getByRole('tab', { name: 'Answer' }).click();
	await page.getByRole('button', { name: 'Copy answer' }).click();
	await page.getByRole('menuitem', { name: 'Copy Markdown' }).click();
	expect(await page.evaluate(() => (window as any).__uiFixture.clipboard)).toBe(answer);
	expect(web).toEqual([]);
});

test('Answer pictures fit a phone screen', async ({ page }) => {
	await answerWithImages(page, 390);
	await expect(page.getByRole('img', { name: 'Red panda' })).toBeVisible();
	expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(390);
});

test('Images in answers can be turned off in Settings', async ({ page }) => {
	await page.goto('/settings');
	await expect(page.getByRole('heading', { name: 'Research', level: 3 })).toBeAttached();
	const images = page.getByRole('switch', { name: 'Images in answers' });
	await expect(images).toBeChecked();
	await expect(page.getByRole('spinbutton', { name: 'Images per answer' })).toHaveValue('8');
	await images.click();
	await expect(images).not.toBeChecked();
});
