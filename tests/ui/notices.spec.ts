import { test, expect, viewRun, emit } from './fixtures';
import type { Page } from '@playwright/test';

const issue = (patch: Record<string, unknown>) => ({
	run_id: 'research',
	code: 'invalid_response',
	provider: 'ollama_cloud',
	model: 'deepseek-v4.1-flash',
	stage: 'research',
	message: 'The model response did not contain a valid research action',
	max_tokens: 40960,
	prompt_tokens: 23079,
	completion_tokens: 30,
	reasoning_tokens: null,
	retry_after_ms: null,
	attempt: 1,
	max_attempts: 1,
	will_retry: false,
	...patch,
});
const answer = ['# Heat pumps', '', 'Text. '.repeat(400), '', '## More', '', 'Text. '.repeat(400)].join('\n');
const turn = (turn_index: number, question: string) => ({
	turn_index,
	question,
	answer_markdown: answer,
	follow_ups: [],
	status: 'completed',
	limits: {},
	accounting: null,
});

async function openConversationWithNotices(page: Page) {
	await page.addInitScript(
		({ answer, turns, issues }) => {
			const internals = (window as any).__TAURI_INTERNALS__;
			const original = internals.invoke;
			let dismissed: string | null = null;
			internals.invoke = async (command: string, args: any = {}) => {
				if (command === 'get_research_result')
					return {
						answer_markdown: answer,
						steps: [
							{
								id: 'e1',
								turn_index: 1,
								step_index: 0,
								step_type: 'error',
								content: 'The model produced an invalid response: Parse error: expected value at line 1 column 1',
								url: null,
							},
						],
						turns,
					};
				if (command === 'get_run_issues') return args.runId === 'research' ? issues : [];
				if (command === 'dismiss_run_notices') {
					(window as any).__uiFixture.calls.push({ command, args });
					dismissed = JSON.stringify(args.dismissed);
					return;
				}
				const result = await original(command, args);
				if (command === 'list_runs')
					return result.map((run: any) =>
						run.id === 'research' ? { ...run, dismissed_notices: dismissed } : run
					);
				return result;
			};
		},
		{
			answer,
			turns: [turn(0, 'How do heat pumps perform?'), turn(1, 'Перескажи кратко')],
			issues: [
				issue({ turn_index: 1, outcome: 'continued', call_id: 'a' }),
				issue({ code: 'timeout', stage: 'research', turn_index: 0, outcome: 'continued', call_id: 'b' }),
			],
		}
	);
	await page.setViewportSize({ width: 1400, height: 800 });
	await viewRun(page, 3);
	await expect(page.getByRole('tab', { name: 'Answer' }).first()).toBeVisible();
}

test('Notices take one line above the result and open without hiding it', async ({ page }) => {
	await openConversationWithNotices(page);
	const strip = page.getByRole('region', { name: 'Run notices' });
	const toggle = strip.getByRole('button', { name: /2 notices/ });
	await expect(toggle).toHaveAttribute('aria-expanded', 'false');
	expect((await strip.boundingBox())!.height).toBeLessThan(48);
	// Both failures were worked around, so the strip is calm.
	await expect(strip).toHaveAttribute('data-level', 'info');

	await toggle.click();
	const card = strip.getByRole('article').filter({ hasText: 'Question 2' });
	await expect(card).toContainText('The agent asked the model again and continued. The answer is not affected.');
	await expect(card.getByText('deepseek-v4.1-flash', { exact: false })).toBeHidden();
	await card.getByText('Technical details').click();
	await expect(card).toContainText('Reported input');

	// The open list stays within its share of the window and above the answer's tabs.
	const box = (await strip.boundingBox())!;
	expect(box.height).toBeLessThanOrEqual(800 * 0.4 + 60);
	const tabs = (await page.locator('.tab-bar').first().boundingBox())!;
	expect(box.y + box.height).toBeLessThanOrEqual(tabs.y + 1);
	await page.locator('.research-view').evaluate((view) => (view.scrollTop = 600));
	const stuck = (await page.locator('.tab-bar').first().boundingBox())!;
	expect(stuck.y).toBeGreaterThanOrEqual(box.y + box.height - 1);
});

test('Dismissed notices stay quiet when the run is opened again', async ({ page }) => {
	await openConversationWithNotices(page);
	const strip = page.getByRole('region', { name: 'Run notices' });
	await strip.getByRole('button', { name: 'Dismiss notices' }).click();
	await expect(strip).toHaveAttribute('data-dismissed', 'true');
	await expect(strip).toContainText('dismissed');
	const calls = await page.evaluate(() =>
		(window as any).__uiFixture.calls.filter((c: any) => c.command === 'dismiss_run_notices')
	);
	expect(calls).toHaveLength(1);
	expect(calls[0].args.runId).toBe('research');

	await page.getByRole('button', { name: 'Back to History' }).click();
	await page.getByRole('button', { name: 'View', exact: true }).nth(3).click();
	await expect(page.getByRole('region', { name: 'Run notices' })).toHaveAttribute('data-dismissed', 'true');
});

test('Activity explains an unreadable model reply', async ({ page }) => {
	await openConversationWithNotices(page);
	const second = page.locator('[data-turn="1"]');
	await second.getByRole('tab', { name: /Activity/ }).click();
	const step = second.locator('.step').first();
	await expect(step).toContainText('Unreadable reply');
	await expect(step).toContainText('The agent asked again and continued.');
	await expect(step).not.toContainText('Parse error');
	await step.getByRole('button', { name: 'Show details' }).click();
	await expect(step).toContainText('Parse error');
});

test('A new notice during a run updates the strip without opening it', async ({ page }) => {
	await page.setViewportSize({ width: 1200, height: 800 });
	await page.goto('/');
	await page.getByRole('button', { name: 'Research', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('Find robot channels');
	await page.getByRole('button', { name: 'Start Research' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await emit(page, 'run:llm_issue', {
		...issue({ code: 'rate_limit', stage: 'extractor', will_retry: true, attempt: 1, max_attempts: 4, outcome: 'retrying', call_id: 'x' }),
		run_id: 'live',
	});
	const strip = page.getByRole('region', { name: 'Run notices' });
	await expect(strip.getByRole('button', { name: /1 notice/ })).toHaveAttribute('aria-expanded', 'false');
	await expect(strip).toHaveAttribute('data-level', 'info');
	await emit(page, 'run:llm_issue', {
		...issue({ code: 'timeout', stage: 'extractor', outcome: 'skipped', call_id: 'y' }),
		run_id: 'live',
	});
	await expect(strip).toHaveAttribute('data-level', 'warning');
	await expect(strip.getByRole('button', { name: /2 notices/ })).toHaveAttribute('aria-expanded', 'false');
	await expect(strip.getByRole('status')).toHaveText('The request timed out');
});
