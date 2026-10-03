import { test, expect, viewRun, emit } from './fixtures';
import type { Page } from '@playwright/test';

const answer = [
	'# Малазийские прокси',
	'',
	'Ниже — подборка проверенных вариантов. ' + 'Цены и наличие могут меняться, уточняйте на сайте. '.repeat(6),
	'',
	'## Платные провайдеры',
	'',
	'| Провайдер | Типы прокси | Ссылка |',
	'| --- | --- | --- |',
	'| Proxy-Seller | IPv4, IPv6 | [proxy-seller.me](https://proxy-seller.me/malaysia) |',
	'| Proxys.io | IPv4 | [proxys.io](https://proxys.io/en) |',
	'',
	'## Бесплатные списки',
	'',
	'Списки: https://spys.one/en/ ' + 'Проверяйте работоспособность. '.repeat(40),
	'',
	'## Советы',
	'',
	'Берите пробный период. '.repeat(60),
].join('\n');

const steps = [
	{ id: 's1', step_index: 0, step_type: 'search', content: 'малазийские прокси купить', url: null },
	{
		id: 's2',
		step_index: 1,
		step_type: 'think',
		content: 'The search results give many providers. ' + 'I should compare their prices and types. '.repeat(8),
		url: null,
	},
	{ id: 's3', step_index: 2, step_type: 'fetch', content: 'Read page (8000 characters of content).', url: 'https://proxy-seller.me/malaysia' },
	{
		id: 's4',
		step_index: 3,
		step_type: 'fetch',
		content: 'Froxy — mobile proxies\n\ndocument.documentElement.className="client-js";RLCONF={"wgBreakFrames":false}',
		url: 'https://froxy.com/',
	},
	{ id: 's5', step_index: 4, step_type: 'error', content: 'Failed to fetch https://broken.example: HTTP 404', url: null },
];

const calls = (page: Page, command: string) =>
	page.evaluate(
		(command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command),
		command
	);

async function openResearch(page: Page, width = 1400, height = 900) {
	await page.addInitScript(
		({ answer, steps }) => {
			const internals = (window as any).__TAURI_INTERNALS__;
			const original = internals.invoke;
			internals.invoke = async (command: string, args: any = {}) => {
				if (command === 'get_research_result') {
					(window as any).__uiFixture.calls.push({ command, args });
					return { answer_markdown: answer, steps };
				}
				return original(command, args);
			};
		},
		{ answer, steps }
	);
	await page.setViewportSize({ width, height });
	await viewRun(page, 3);
	await expect(page.getByRole('tab', { name: 'Answer' })).toBeVisible();
}

test('Answer, Activity and Sources are tabs above the answer', async ({ page }) => {
	await openResearch(page);
	const tabs = page.getByRole('tablist', { name: 'Research results' });
	await expect(tabs.getByRole('tab')).toHaveText(['Answer', /Activity\s*5/, /Sources\s*4/]);
	await expect(page.getByRole('tab', { name: 'Answer' })).toHaveAttribute('aria-selected', 'true');
	await expect(page.getByRole('tabpanel', { name: 'Answer' })).toContainText('Малазийские прокси');

	await page.getByRole('tab', { name: /Activity/ }).click();
	const activity = page.getByRole('tabpanel', { name: /Activity/ });
	await expect(activity.locator('.step-number')).toHaveText(['1', '2', '3', '4', '5']);
	// Numbers are drawn by the step itself, so the native list marker cannot overlap them.
	expect(await activity.locator('ol').evaluate((ol) => getComputedStyle(ol).listStyleType)).toBe('none');
	await expect(activity.locator('.step').nth(2)).toContainText('Read page');
	await expect(activity.locator('.step').nth(2)).toContainText('proxy-seller.me › malaysia');
	await expect(activity.locator('.step').nth(2)).not.toContainText('8000 characters');

	const legacy = activity.locator('.step').nth(3);
	await expect(legacy).toContainText('Froxy — mobile proxies');
	await expect(legacy).not.toContainText('RLCONF');
	await legacy.getByRole('button', { name: 'Show page text' }).click();
	await expect(legacy.locator('pre')).toContainText('RLCONF');
	await expect(activity.locator('.step').nth(4)).toContainText('Request issue');
	// "Show more" only where the one-line summary is actually cut.
	await expect(activity.locator('.step').nth(0).getByRole('button', { name: 'Show more' })).toHaveCount(0);
	await activity.locator('.step').nth(1).getByRole('button', { name: 'Show more' }).click();
	await expect(activity.locator('.step').nth(1).getByRole('button', { name: 'Show less' })).toBeVisible();

	// Keyboard: arrows move between tabs.
	await page.getByRole('tab', { name: /Activity/ }).focus();
	await page.keyboard.press('ArrowRight');
	await expect(page.getByRole('tab', { name: /Sources/ })).toHaveAttribute('aria-selected', 'true');
});

test('Sources list cited links first and mark what was read', async ({ page }) => {
	await openResearch(page);
	await page.getByRole('tab', { name: /Sources/ }).click();
	const items = page.getByRole('tabpanel', { name: /Sources/ }).locator('.source');
	await expect(items).toHaveCount(4);
	await expect(items.nth(0)).toContainText('proxy-seller.me');
	await expect(items.nth(0)).toContainText('Read');
	await expect(items.nth(0)).toContainText('Cited');
	await expect(items.nth(1)).toContainText('proxys.io');
	await expect(items.nth(1)).not.toContainText('Read');
	await expect(items.nth(3)).toContainText('Froxy — mobile proxies');
	await expect(items.nth(3)).not.toContainText('Cited');
});

test('The answer can be copied as Markdown, text or with sources', async ({ page }) => {
	await openResearch(page);
	const copied = async (name: string) => {
		await page.getByRole('button', { name: 'Copy answer' }).click();
		await page.getByRole('menuitem', { name }).click();
		return (await calls(page, 'copy_text')).at(-1).args.text as string;
	};
	expect(await copied('Copy Markdown')).toBe(answer);
	const text = await copied('Copy as text');
	expect(text).toContain('Провайдер\tТипы прокси\tСсылка');
	expect(text).not.toContain('|');
	const withSources = await copied('Copy with sources');
	expect(withSources.startsWith(answer.trimEnd())).toBe(true);
	expect(withSources).toContain('## Sources\n\n1. [proxy-seller.me](https://proxy-seller.me/malaysia)');
});

test('Long answers have a contents list, readable lines and a back-to-top button', async ({ page }) => {
	await openResearch(page);
	const toc = page.getByRole('navigation', { name: 'Contents' });
	await expect(toc.getByRole('link')).toHaveText([
		'Малазийские прокси',
		'Платные провайдеры',
		'Бесплатные списки',
		'Советы',
	]);
	await toc.getByRole('link', { name: 'Советы' }).click();
	await expect(page.getByRole('heading', { name: 'Советы' })).toBeInViewport();
	await expect(toc.getByRole('link', { name: 'Советы' })).toHaveAttribute('aria-current', 'true');
	await expect(toc).toBeInViewport();

	const paragraph = page.locator('.markdown > p').first();
	const fontSize = await paragraph.evaluate((p) => parseFloat(getComputedStyle(p).fontSize));
	expect((await paragraph.boundingBox())!.width).toBeLessThanOrEqual(fontSize * 46);

	await page.getByRole('button', { name: 'Back to top' }).click();
	await expect(page.getByRole('heading', { name: 'Малазийские прокси' })).toBeInViewport();
	await expect(page.getByRole('button', { name: 'Back to top' })).toHaveCount(0);
});

test('Switching tabs further down shows the tab from its start', async ({ page }) => {
	await openResearch(page, 1400, 700);
	const view = page.locator('.research-view');
	const tabBar = page.locator('.tab-bar');
	const expectTabsAtTop = async () => {
		const top = await view.evaluate((v) => v.getBoundingClientRect().top);
		await expect
			.poll(async () => Math.abs((await tabBar.boundingBox())!.y - top))
			.toBeLessThanOrEqual(2);
		const panel = (await page.getByRole('tabpanel').boundingBox())!;
		const bar = (await tabBar.boundingBox())!;
		expect(Math.abs(panel.y - (bar.y + bar.height))).toBeLessThanOrEqual(2);
	};
	for (const name of [/Sources/, /Activity/, /Answer/]) {
		await view.evaluate((v) => (v.scrollTop = v.scrollHeight));
		await page.getByRole('tab', { name }).click();
		await expectTabsAtTop();
	}
	// Above the tabs, switching keeps the reader where they are.
	await view.evaluate((v) => (v.scrollTop = 0));
	await page.getByRole('tab', { name: /Sources/ }).click();
	expect(await view.evaluate((v) => v.scrollTop)).toBe(0);
});

test('A short tab of the last answer still opens under the tabs at the top', async ({ page }) => {
	await page.addInitScript(
		({ answer, steps }) => {
			const internals = (window as any).__TAURI_INTERNALS__;
			const original = internals.invoke;
			const turn = (turn_index: number, question: string, answer_markdown: string) => ({
				turn_index,
				question,
				answer_markdown,
				follow_ups: [],
				status: 'completed',
				limits: {},
				accounting: null,
			});
			internals.invoke = async (command: string, args: any = {}) => {
				if (command === 'get_research_result')
					return {
						answer_markdown: answer,
						steps: steps.map((s: any) => ({ ...s, turn_index: 1 })),
						turns: [turn(0, 'Первый вопрос', answer), turn(1, 'Перескажи кратко', answer)],
					};
				return original(command, args);
			};
		},
		{ answer, steps }
	);
	await page.setViewportSize({ width: 1400, height: 800 });
	await viewRun(page, 3);
	const view = page.locator('.research-view');
	const last = page.locator('[data-turn="1"]');
	const tabBar = last.locator('.tab-bar');
	await view.evaluate((v) => (v.scrollTop = v.scrollHeight));
	await last.getByRole('tab', { name: /Sources/ }).click();
	const top = await view.evaluate((v) => v.getBoundingClientRect().top);
	await expect.poll(async () => Math.abs((await tabBar.boundingBox())!.y - top)).toBeLessThanOrEqual(2);
	await expect(last.getByRole('tabpanel')).toContainText('proxy-seller.me');
	// The earlier answer's tabs are out of sight, not peeking over the top.
	await expect(page.locator('[data-turn="0"] .tab-bar')).not.toBeInViewport();
});

test('Narrow windows get a contents menu and tables keep words whole', async ({ page }) => {
	await openResearch(page, 900, 600);
	await expect(page.getByRole('navigation', { name: 'Contents' })).toHaveCount(0);
	await page.getByRole('button', { name: 'Contents' }).click();
	await page.getByRole('menuitem', { name: 'Бесплатные списки' }).click();
	await expect(page.getByRole('heading', { name: 'Бесплатные списки' })).toBeInViewport();

	const header = page.locator('.markdown th').first();
	const lines = await header.evaluate((th) => {
		const range = document.createRange();
		range.selectNodeContents(th);
		return new Set([...range.getClientRects()].map((r) => Math.round(r.top))).size;
	});
	expect(lines).toBe(1);
	expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(900);
});

test('A live run shows activity first and switches to the answer when it arrives', async ({ page }) => {
	await page.setViewportSize({ width: 1200, height: 800 });
	await page.goto('/');
	await page.getByRole('button', { name: 'Research', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('Найди малазийские прокси');
	await page.getByRole('button', { name: 'Start Research' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	for (const [i, s] of steps.slice(0, 3).entries())
		await emit(page, 'run:research_step', { run_id: 'live', step_id: s.id, ...s, step_index: i });
	await expect(page.getByRole('tab', { name: /Activity/ })).toHaveAttribute('aria-selected', 'true');
	await expect(page.getByRole('tab', { name: 'Answer' })).toBeDisabled();
	await expect(page.locator('.current-step')).toContainText('Reading proxy-seller.me');

	await emit(page, 'run:research_answer', { run_id: 'live', markdown: answer });
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });
	await expect(page.getByRole('tab', { name: 'Answer' })).toHaveAttribute('aria-selected', 'true');
	await expect(page.getByRole('tabpanel', { name: 'Answer' })).toContainText('Платные провайдеры');
});

const limits = { target_row_count: 16, max_budget_usd: 1, max_duration_seconds: 600 };

async function finishedLiveResearch(page: Page) {
	await page.setViewportSize({ width: 1200, height: 800 });
	await page.goto('/');
	await page.getByRole('button', { name: 'Research', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('Найди малазийские прокси');
	await page.getByRole('button', { name: 'Start Research' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await emit(page, 'run:research_step', { run_id: 'live', step_id: 'a', turn_index: 0, ...steps[0] });
	await emit(page, 'run:research_answer', {
		run_id: 'live',
		turn_index: 0,
		markdown: 'Use Proxy-Seller.',
		follow_ups: ['Какие из них дешевле?', 'Есть ли бесплатные?'],
	});
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });
}

test('Follow-up questions continue the conversation with its history', async ({ page }) => {
	await finishedLiveResearch(page);
	const box = page.getByRole('textbox', { name: 'Ask a follow-up' });
	await expect(box).toBeEnabled();
	await expect(page.getByRole('group', { name: 'Suggested questions' }).getByRole('button')).toHaveText([
		'Какие из них дешевле?',
		'Есть ли бесплатные?',
	]);
	await page.getByRole('button', { name: 'Какие из них дешевле?' }).click();
	await expect(box).toHaveValue('Какие из них дешевле?');
	await expect(page.getByRole('button', { name: /Follow-up limits/ })).toContainText('16 steps · $1.00 · 10 min');
	await box.press('Control+Enter');

	const [ask] = await calls(page, 'ask_follow_up');
	expect(ask.args).toEqual({ runId: 'live', question: 'Какие из них дешевле?', stopConditions: limits });
	const second = page.getByRole('region', { name: 'Какие из них дешевле?' });
	await expect(second.getByRole('heading', { name: 'Какие из них дешевле?' })).toBeVisible();
	await expect(second.getByRole('tab', { name: /Activity/ })).toHaveAttribute('aria-selected', 'true');
	await expect(box).toBeDisabled();

	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await emit(page, 'run:research_step', { run_id: 'live', step_id: 'b', turn_index: 1, ...steps[2] });
	await emit(page, 'run:research_answer', { run_id: 'live', turn_index: 1, markdown: 'Proxy5 is cheapest.', follow_ups: [] });
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });
	await expect(second.getByRole('tabpanel', { name: 'Answer' })).toContainText('Proxy5 is cheapest.');
	await expect(page.getByRole('region', { name: 'Research answer' }).first()).toContainText('Use Proxy-Seller.');
	await expect(second.getByRole('tab', { name: /Activity/ })).toContainText('1');
	await expect(box).toBeEnabled();
	await expect(box).toHaveValue('');
});

test('Cancelling a follow-up keeps the earlier answers', async ({ page }) => {
	await finishedLiveResearch(page);
	await page.getByRole('textbox', { name: 'Ask a follow-up' }).fill('Есть ли бесплатные?');
	await page.getByRole('button', { name: 'Ask', exact: true }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await page.getByRole('button', { name: 'Cancel', exact: true }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Cancel run' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'cancelled' });
	const second = page.getByRole('region', { name: 'Есть ли бесплатные?' });
	await expect(second).toContainText('This question was cancelled');
	await expect(page.getByRole('region', { name: 'Research answer' }).first()).toContainText('Use Proxy-Seller.');
	await expect(page.getByRole('textbox', { name: 'Ask a follow-up' })).toBeEnabled();
});

test('A saved conversation can be continued from History', async ({ page }) => {
	await page.addInitScript(() => {
		const internals = (window as any).__TAURI_INTERNALS__;
		const original = internals.invoke;
		internals.invoke = async (command: string, args: any = {}) => {
			if (command === 'get_research_result') {
				(window as any).__uiFixture.calls.push({ command, args });
				return {
					answer_markdown: 'Second answer.',
					steps: [{ id: 'x', turn_index: 1, step_index: 0, step_type: 'search', content: 'q', url: null }],
					turns: [
						{ turn_index: 0, question: 'Saved research research', answer_markdown: 'First answer.', follow_ups: [], status: 'completed', limits: { target_row_count: 12, max_budget_usd: 0.5, max_duration_seconds: 300 }, accounting: { spent_usd: 0.25 } },
						{ turn_index: 1, question: 'And then?', answer_markdown: 'Second answer.', follow_ups: ['What next?'], status: 'completed', limits: { target_row_count: 12, max_budget_usd: 0.5, max_duration_seconds: 300 }, accounting: { spent_usd: 0.1 } },
					],
				};
			}
			return original(command, args);
		};
	});
	await page.setViewportSize({ width: 1200, height: 800 });
	await viewRun(page, 3);
	await expect(page.getByRole('region', { name: 'And then?' })).toContainText('Second answer.');
	await expect(page.getByText('Conversation cost: $0.35')).toBeVisible();
	await page.getByRole('textbox', { name: 'Ask a follow-up' }).fill('What about prices?');
	await page.getByRole('button', { name: 'Ask', exact: true }).click();
	await expect(page).toHaveURL(/\/$/);
	const [ask] = await calls(page, 'ask_follow_up');
	expect(ask.args).toEqual({
		runId: 'research',
		question: 'What about prices?',
		stopConditions: { target_row_count: 12, max_budget_usd: 0.5, max_duration_seconds: 300 },
	});
	await expect(page.getByRole('region', { name: 'What about prices?' })).toBeVisible();
	await expect(page.getByRole('region', { name: 'And then?' })).toContainText('Second answer.');
});

test('Research has its own step limit and exports the conversation or one turn', async ({ page }) => {
	await page.setViewportSize({ width: 1200, height: 800 });
	await page.goto('/');
	await page.getByRole('button', { name: 'Research', exact: true }).click();
	await expect(page.getByRole('button', { name: /Stop conditions/ })).toContainText('16 steps');
	await page.getByRole('button', { name: 'Table', exact: true }).click();
	await expect(page.getByRole('button', { name: /Stop conditions/ })).toContainText('50 rows');
	await page.getByRole('button', { name: 'Research', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('Find proxies');
	await page.getByRole('button', { name: 'Start Research' }).click();
	expect((await calls(page, 'start_run'))[0].args.stopConditions.target_row_count).toBe(16);

	await viewRun(page, 3);
	await page.getByRole('button', { name: 'Export' }).click();
	const dialog = page.getByRole('dialog', { name: 'Export Results' });
	await dialog.getByLabel('Export').selectOption('Whole conversation');
	await dialog.getByRole('button', { name: 'Export' }).click();
	const [whole] = await calls(page, 'export_run');
	expect(whole.args.request.turn_index ?? null).toBeNull();
	await page.getByRole('dialog', { name: 'Export complete' }).getByRole('button', { name: 'Done' }).click();
	await page.getByRole('button', { name: 'Export' }).click();
	await dialog.getByLabel('Export').selectOption({ index: 1 });
	await expect(dialog.getByLabel('Export')).toHaveValue('0');
	await dialog.getByRole('button', { name: 'Export' }).click();
	expect((await calls(page, 'export_run')).at(-1).args.request.turn_index).toBe(0);
});
