import { test, expect, emit } from './fixtures';

const answer = [
	'Mass and energy: \\(E = mc^2\\). Plans cost $5 to $10 a month.',
	'',
	'$$',
	'\\int_0^1 x^2\\,dx = \\frac{1}{3}' + ' + \\frac{1}{3}'.repeat(40),
	'$$',
	'',
	'Broken: $\\frac{a$.',
].join('\n');

test('Research answers draw formulas and keep prices as text', async ({ page }) => {
	await page.setViewportSize({ width: 900, height: 800 });
	await page.goto('/');
	await page.getByRole('button', { name: 'Research', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('Formulas');
	await page.getByRole('button', { name: 'Start Research' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await emit(page, 'run:research_answer', { run_id: 'live', turn_index: 0, markdown: answer, follow_ups: [] });
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });

	const panel = page.getByRole('tabpanel', { name: 'Answer' });
	await expect(panel.getByRole('img', { name: 'E = mc^2' }).locator('svg')).toBeVisible();
	const display = panel.locator('div.math-display');
	await expect(display.locator('svg').first()).toBeVisible();
	await expect(panel).toContainText('Plans cost $5 to $10 a month.');
	await expect(panel.locator('.math-error')).toHaveText('$\\frac{a$');

	// A long formula scrolls inside its own box; the page keeps its width.
	expect(await display.evaluate((el) => el.scrollWidth > el.clientWidth)).toBe(true);
	expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(900);

	await page.getByRole('button', { name: 'Copy answer' }).click();
	await page.getByRole('menuitem', { name: 'Copy Markdown' }).click();
	expect(await page.evaluate(() => (window as any).__uiFixture.clipboard)).toBe(answer);
	await page.getByRole('button', { name: 'Copy answer' }).click();
	await page.getByRole('menuitem', { name: 'Copy as text' }).click();
	expect(await page.evaluate(() => (window as any).__uiFixture.clipboard)).toContain('\\(E = mc^2\\). Plans cost $5 to $10');
});
