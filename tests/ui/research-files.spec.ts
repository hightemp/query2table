import { test, expect, emit } from './fixtures';

const calls = (page: import('@playwright/test').Page, command: string) =>
	page.evaluate((command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command), command);

async function startWithFiles(page: import('@playwright/test').Page, searchWeb: boolean) {
	await page.goto('/');
	await page.getByRole('button', { name: 'Research', exact: true }).click();
	await page.getByLabel('What would you like to find?').fill('How did revenue change?');
	await page.getByRole('button', { name: 'Attach files' }).click();
	await expect(page.getByRole('list', { name: 'Attached files' }).getByRole('listitem')).toHaveCount(2);
	const web = page.getByRole('checkbox', { name: 'Also search the web' });
	await expect(web).toBeChecked();
	if (!searchWeb) await web.uncheck();
	await page.getByRole('button', { name: 'Start Research' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
}

test('Research can work from the attached files only and shows them with the question', async ({ page }) => {
	await startWithFiles(page, false);
	const [start] = await calls(page, 'start_run');
	expect(start.args.attachments).toEqual(['att-report.pdf', 'att-prices.xlsx']);
	expect(start.args.sourceMode).toBe('files');
	const turn = page.locator('[data-turn="0"]');
	await expect(turn.getByRole('list', { name: 'Attached files' }).getByRole('button')).toHaveText(['report.pdf', 'prices.xlsx']);
	// The next question keeps working from the files unless asked otherwise.
	await emit(page, 'run:research_answer', { run_id: 'live', turn_index: 0, markdown: 'Done.', follow_ups: [] });
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });
	const followUp = page.getByRole('region', { name: 'Follow-up question' });
	await expect(followUp.getByRole('checkbox', { name: 'Also search the web' })).not.toBeChecked();
});

test('Read steps, answer links and sources open the passage of the file', async ({ page }) => {
	await startWithFiles(page, true);
	expect((await calls(page, 'start_run'))[0].args.sourceMode).toBe('web');
	await emit(page, 'run:research_step', {
		run_id: 'live',
		step_id: 'r1',
		turn_index: 0,
		step_index: 0,
		step_type: 'read',
		content: 'report.pdf, page 3',
		url: 'attachment://att-report.pdf?page=3',
	});
	await page.getByRole('tab', { name: /Activity/ }).click();
	const step = page.locator('.step.read');
	await expect(step).toContainText('Read file');
	await expect(step).toContainText('report.pdf, p. 3');
	await step.getByRole('button', { name: 'Show passage' }).click();
	const dialog = page.getByRole('dialog', { name: 'report.pdf, p. 3' });
	await expect(dialog).toContainText('revenue grew to 12 million');
	await dialog.getByRole('button', { name: 'Open file' }).click();
	expect((await calls(page, 'open_attachment'))[0].args).toEqual({ id: 'att-report.pdf' });
	await expect(dialog).toHaveCount(0);

	await emit(page, 'run:research_answer', {
		run_id: 'live',
		turn_index: 0,
		markdown: 'Revenue grew [report.pdf, p. 3](attachment://att-report.pdf?page=3).',
		follow_ups: [],
	});
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });
	await page.getByRole('tab', { name: 'Answer' }).click();
	const answer = page.getByRole('tabpanel', { name: 'Answer' });
	await answer.getByRole('link', { name: 'report.pdf, p. 3' }).click();
	await expect(page.getByRole('dialog', { name: 'report.pdf, p. 3' })).toBeVisible();
	await page.getByRole('dialog').getByRole('button', { name: 'Close' }).first().click();

	await page.getByRole('tab', { name: /Sources/ }).click();
	const source = page.getByRole('tabpanel', { name: /Sources/ }).getByRole('listitem').first();
	await expect(source).toContainText('report.pdf, p. 3');
	await expect(source).toContainText('Cited');
	await expect(source).toContainText('Read');
});

test('A follow-up question can bring its own files', async ({ page }) => {
	await startWithFiles(page, true);
	await emit(page, 'run:research_answer', { run_id: 'live', turn_index: 0, markdown: 'Done.', follow_ups: [] });
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'completed' });
	await page.evaluate(() => ((window as any).__uiFixture.dialogFiles = ['/docs/q3.pdf']));
	const followUp = page.getByRole('region', { name: 'Follow-up question' });
	await followUp.getByRole('button', { name: 'Attach files' }).click();
	await expect(followUp.getByRole('list', { name: 'Attached files' }).getByRole('listitem')).toHaveCount(1);
	await followUp.getByRole('checkbox', { name: 'Also search the web' }).uncheck();
	await followUp.getByRole('textbox', { name: 'Ask a follow-up' }).fill('And in Q3?');
	await followUp.getByRole('button', { name: 'Ask' }).click();
	const [ask] = await calls(page, 'ask_follow_up');
	expect(ask.args).toMatchObject({ attachments: ['att-q3.pdf'], sourceMode: 'files' });
	await expect(page.locator('[data-turn="1"]').getByRole('list', { name: 'Attached files' })).toContainText('q3.pdf');
	await expect(followUp.getByRole('list', { name: 'Attached files' })).toHaveCount(0);
});
