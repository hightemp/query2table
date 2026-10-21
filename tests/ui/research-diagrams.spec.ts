import { test, expect, emit } from './fixtures';
import type { Page } from '@playwright/test';

const chart = (value: Record<string, unknown>) => ['```chart', JSON.stringify(value), '```'].join('\n');
const charts: Record<string, Record<string, unknown>> = {
	bar: { type: 'bar', title: 'Bar sales', x: ['2022', '2023'], series: [{ name: 'A', data: [1, 2] }, { name: 'B', data: [2, null] }], unit: 'M' },
	line: { type: 'line', title: 'Line sales', x: ['Jan', 'Feb', 'Mar'], series: [{ data: [1, 3, 2] }] },
	area: { type: 'area', title: 'Area sales', stacked: true, x: ['a', 'b'], series: [{ data: [1, 2] }, { data: [2, 1] }] },
	mixed: { type: 'bar', title: 'Mixed sales', x: ['Q1', 'Q2'], series: [{ name: 'Revenue', data: [10, 12] }, { name: 'Margin', type: 'line', axis: 'right', data: [20, 25] }], unit_right: '%' },
	pie: { type: 'pie', title: 'Pie share', x: ['Chrome', 'Safari'], series: [{ data: [70, 30] }] },
	donut: { type: 'donut', title: 'Donut share', x: ['Yes', 'No'], series: [{ data: [3, 1] }] },
	scatter: { type: 'scatter', title: 'Scatter cars', series: [{ data: [[1, 2], [2, 3]] }] },
	radar: { type: 'radar', title: 'Radar phones', x: ['Speed', 'Price', 'Range'], series: [{ name: 'A', data: [5, 3, 4] }] },
	heatmap: { type: 'heatmap', title: 'Heatmap visits', x: ['Mon', 'Tue'], y: ['AM', 'PM'], series: [{ data: [[1, 2], [3, null]] }] },
};
const flow = ['```mermaid', 'flowchart LR', '  A["Ask a question"] --> B["Read pages"] --> C["Answer"]', '```'].join('\n');
const answer = [
	'How it works:',
	'',
	flow,
	'',
	...Object.values(charts).flatMap((value) => [chart(value), '']),
	'Broken diagram:',
	'',
	'```mermaid',
	'flowchart LR',
	'  A --> ',
	'  ]]]',
	'```',
	'',
	'```chart',
	'{"type": "bar", "x": ["a"], "series": [{"data": [1, 2]}]}',
	'```',
	'',
	'Unsafe names stay text:',
	'',
	chart({ type: 'bar', title: 'Unsafe', x: ['<img src=x onerror="window.__hacked=1">'], series: [{ name: '<b onmouseover="window.__hacked=1">B</b>', data: [1] }] }),
	'',
	'```mermaid',
	'flowchart LR',
	'  A["<img src=x onerror=window.__hacked=1>"] --> B',
	'```',
].join('\n');

async function showAnswer(page: Page, markdown = answer) {
	await page.goto('/');
	await page.getByRole('button', { name: 'Research', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('Diagrams');
	await page.getByRole('button', { name: 'Start Research' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await emit(page, 'run:research_answer', { run_id: 'live', turn_index: 0, markdown, follow_ups: [] });
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });
	return page.getByRole('tabpanel', { name: 'Answer' });
}

const calls = (page: Page, command: string) =>
	page.evaluate((command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command), command);

test('Research answers draw Mermaid diagrams and every kind of chart', async ({ page }) => {
	await page.setViewportSize({ width: 1000, height: 800 });
	const panel = await showAnswer(page);

	const diagram = panel.getByRole('img', { name: /Ask a question/ });
	await expect(diagram.locator('.diagram-view svg').first()).toBeVisible();
	await expect(diagram).toContainText('Read pages');
	for (const value of Object.values(charts)) {
		const figure = panel.getByRole('img', { name: new RegExp(`^${value.title}`) });
		await expect(figure.locator('.diagram-view svg').first(), String(value.title)).toBeVisible();
	}
	// Bars carry their labels and units; the stacked area and the second axis are drawn.
	await expect(panel.getByRole('img', { name: /^Bar sales/ })).toContainText('2023');
	await expect(panel.getByRole('img', { name: /^Mixed sales/ })).toContainText('%');

	// Broken blocks keep their source with a note.
	const broken = panel.locator('.diagram-error');
	await expect(broken).toHaveCount(2);
	await expect(broken.first()).toContainText('Could not draw this diagram');
	await expect(broken.first()).toContainText(']]]');
	await expect(broken.nth(1)).toContainText('series 1 has 2 values for 1 label');

	// Names from the answer never become markup.
	await panel.getByRole('img', { name: /^Unsafe/ }).hover();
	await expect(panel.locator('img[src="x"]')).toHaveCount(0);
	expect(await page.evaluate(() => (window as any).__hacked)).toBeUndefined();

	// The page keeps its width; wide diagrams scroll inside their box.
	expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(1000);
});

test('Copies keep the diagram and chart sources', async ({ page }) => {
	const markdown = ['Intro', '', flow, '', chart(charts.pie)].join('\n');
	await showAnswer(page, markdown);
	await page.getByRole('button', { name: 'Copy answer' }).click();
	await page.getByRole('menuitem', { name: 'Copy Markdown' }).click();
	expect(await page.evaluate(() => (window as any).__uiFixture.clipboard)).toBe(markdown);
	await page.getByRole('button', { name: 'Copy answer' }).click();
	await page.getByRole('menuitem', { name: 'Copy as text' }).click();
	const text = await page.evaluate(() => (window as any).__uiFixture.clipboard);
	expect(text).toContain('flowchart LR');
	expect(text).toContain('"type":"pie"');
});

test('A diagram is saved as SVG or PNG, its source copied, and opened full screen', async ({ page }) => {
	const panel = await showAnswer(page, ['Intro', '', chart(charts.bar), '', flow].join('\n'));
	const bars = panel.getByRole('img', { name: /^Bar sales/ });
	await expect(bars.locator('.diagram-view svg').first()).toBeVisible();

	const actions = panel.getByRole('button', { name: 'Diagram actions' }).first();
	await actions.click();
	await page.getByRole('menuitem', { name: 'Save as SVG' }).click();
	await expect(page.getByText(/^Saved /)).toBeVisible();
	let [saved] = await calls(page, 'save_picture');
	expect(saved.args.path).toBe('/tmp/query2table-fixture.csv');
	expect(atob(saved.args.data)).toMatch(/^<svg[\s\S]*Bar sales[\s\S]*<\/svg>$/);
	const [dialog] = await calls(page, 'plugin:dialog|save');
	expect(dialog.args.options.defaultPath).toBe('Bar sales.svg');

	await actions.click();
	await page.getByRole('menuitem', { name: 'Save as PNG' }).click();
	await expect.poll(async () => (await calls(page, 'save_picture')).length).toBe(2);
	saved = (await calls(page, 'save_picture'))[1];
	expect(atob(saved.args.data).slice(1, 4)).toBe('PNG');

	// Right-click opens the same menu.
	await panel.getByRole('img', { name: /Ask a question/ }).click({ button: 'right' });
	await page.getByRole('menuitem', { name: 'Copy source' }).click();
	expect(await page.evaluate(() => (window as any).__uiFixture.clipboard)).toContain('flowchart LR');

	await actions.click();
	await page.getByRole('menuitem', { name: 'Open full screen' }).click();
	const viewer = page.getByRole('dialog', { name: 'Bar sales' });
	await expect(viewer.locator('.picture svg').first()).toBeVisible();
	const zoom = viewer.getByRole('status');
	await expect(zoom).toHaveText('100%');
	await viewer.getByRole('button', { name: 'Zoom in' }).click();
	await expect(zoom).toHaveText('125%');
	await viewer.getByRole('button', { name: 'Fit' }).click();
	await expect(zoom).toHaveText('100%');
	await page.keyboard.press('Escape');
	await expect(viewer).toHaveCount(0);
});

test('Diagrams and charts follow the theme', async ({ page }) => {
	await page.addInitScript(() => Object.assign((window as any).__uiFixture.values, { theme: 'system' }));
	await page.emulateMedia({ colorScheme: 'dark' });
	const panel = await showAnswer(page, ['Intro', '', chart(charts.line), '', flow].join('\n'));
	const title = panel.getByRole('img', { name: /^Line sales/ }).locator('svg text', { hasText: 'Line sales' });
	await expect(title).toHaveAttribute('fill', /#e6ebf3/i);
	await page.emulateMedia({ colorScheme: 'light' });
	await expect(title).toHaveAttribute('fill', /#182338/i);
	await expect(panel.getByRole('img', { name: /Ask a question/ }).locator('.diagram-view svg').first()).toBeVisible();
});

test('Diagrams fit a phone screen', async ({ page }) => {
	await page.setViewportSize({ width: 390, height: 800 });
	const wide = ['```mermaid', 'flowchart LR', ...'ABCDEFGHIJ'.split('').map((c, i) => `  ${c}${i}["Step number ${i}"] --> ${c}${i + 1}["Next step ${i}"]`), '```'].join('\n');
	const panel = await showAnswer(page, ['Intro', '', wide, '', chart(charts.bar)].join('\n'));
	await expect(panel.getByRole('img', { name: /Step number 0/ }).locator('.diagram-view svg').first()).toBeVisible();
	await expect(panel.getByRole('img', { name: /^Bar sales/ }).locator('.diagram-view svg').first()).toBeVisible();
	expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(390);
});
