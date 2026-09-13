import { test, expect, viewRun, emit } from './fixtures';

for (const [width, height] of [
	[900, 600],
	[1200, 800],
	[1440, 900],
]) {
	for (const theme of ['dark', 'light']) {
		test(`Settings scroll boundaries ${width}x${height} ${theme}`, async ({ page }) => {
			await page.setViewportSize({ width, height });
			await page.goto('/settings');
			if (theme === 'light') await page.getByRole('button', { name: 'Toggle theme' }).click();
			for (const collapsed of [false, true]) {
				if (collapsed) await page.getByRole('button', { name: 'Toggle sidebar' }).click();
				for (const logs of [false, true]) {
					if (logs) await page.getByRole('button', { name: /Logs \(/ }).click();
					await page.getByRole('link', { name: 'Application files', exact: true }).click();
					await expect(page.getByRole('button', { name: 'Save', exact: true })).toBeVisible();
					await expect(page.getByLabel('Application logs', { exact: true })).toBeVisible();
					const metrics = await page.evaluate(() => {
						const scroll = document.querySelector('.settings-scroll')!;
						const r = scroll.getBoundingClientRect();
						const logs = document.querySelector('.log-panel')!.getBoundingClientRect();
						const cards = [...scroll.querySelectorAll('.settings-section,.app-files')];
						return {
							width: innerWidth,
							documentWidth: document.documentElement.scrollWidth,
							bottom: r.bottom,
							logTop: logs.top,
							gap: Math.min(...cards.map((e) => r.right - e.getBoundingClientRect().right)),
							scrollWidth: scroll.scrollWidth,
							clientWidth: scroll.clientWidth,
						};
					});
					expect(metrics.documentWidth).toBeLessThanOrEqual(width);
					expect(metrics.bottom).toBeLessThanOrEqual(metrics.logTop);
					expect(metrics.gap).toBeGreaterThanOrEqual(12);
					expect(metrics.scrollWidth).toBeLessThanOrEqual(metrics.clientWidth + 1);
					if (logs) await page.getByRole('button', { name: /Logs \(/ }).click();
				}
			}
			await page.screenshot({ path: `test-results/settings-${width}-${theme}.png` });
		});
	}
}

test('Settings guard keeps edits and offers save, discard and stay', async ({ page }) => {
	await page.goto('/settings');
	await page.getByLabel(/^Max output tokens /).fill('8192');
	await page.getByRole('link', { name: 'History', exact: true }).click();
	await expect(page.getByRole('dialog', { name: 'Unsaved changes' })).toBeVisible();
	await page.getByRole('button', { name: 'Stay', exact: true }).click();
	await expect(page.getByLabel(/^Max output tokens /)).toHaveValue('8192');
	await page.evaluate(() => ((window as any).__uiFixture.failSave = true));
	await page.getByRole('button', { name: 'Save', exact: true }).click();
	await expect(page.getByRole('alert')).toContainText('Local data');
	await page.evaluate(() => ((window as any).__uiFixture.failSave = false));
	await page.getByRole('link', { name: 'History', exact: true }).click();
	await page.getByRole('button', { name: 'Save and leave' }).click();
	await expect(page.getByRole('heading', { name: 'Run History' })).toBeVisible();
});

test('Model menu fits the viewport and keyboard filtering preserves the saved value', async ({
	page,
}) => {
	await page.setViewportSize({ width: 900, height: 600 });
	await page.goto('/settings');
	const model = page.getByRole('combobox', { name: /Ollama Cloud Model/ });
	await model.scrollIntoViewIfNeeded();
	await model.click();
	await expect(page.getByRole('option', { name: 'model-0', exact: true })).toBeVisible();
	const popup = await page.locator('.dropdown').boundingBox();
	expect(popup!.y).toBeGreaterThanOrEqual(0);
	expect(popup!.y + popup!.height).toBeLessThanOrEqual(600);
	await model.fill('model-19');
	await model.press('ArrowDown');
	await model.press('Enter');
	await expect(model).toHaveValue('model-19');
	await model.fill('unmatched');
	await model.press('Escape');
	await expect(model).toHaveValue('model-19');
});

test('Virtual table supports structured values, keyboard details, sources and filtering', async ({
	page,
}) => {
	await page.setViewportSize({ width: 1200, height: 800 });
	await viewRun(page, 0);
	await expect(page.locator('.data-row').first()).toBeVisible();
	expect(await page.locator('.data-row').count()).toBeLessThan(50);
	await expect(page.locator('.table-scroll')).not.toContainText('[object Object]');
	await expect(page.locator('.data-row').first()).toContainText('ROS 2 · Arduino');
	await page.getByRole('button', { name: 'Open row 1 details', exact: true }).focus();
	await page.keyboard.press('Enter');
	const dialog = page.getByRole('dialog', { name: 'Row Details' });
	await expect(dialog).toContainText('"enabled": false');
	await expect(dialog).toContainText('Evidence title');
	await page.getByRole('button', { name: 'Copy Count' }).click();
	expect(
		await page.evaluate(
			() =>
				(window as any).__uiFixture.calls.findLast((c: any) => c.command === 'copy_text').args.text
		)
	).toBe('0');
	await page.keyboard.press('Escape');
	await expect(page.getByRole('button', { name: 'Open row 1 details', exact: true })).toBeFocused();
	await page.getByRole('searchbox', { name: 'Search results' }).fill('0999');
	await expect(page.locator('.data-row')).toHaveCount(1);
	await expect(page.locator('.data-row')).toContainText('Robot channel 0999');
	await page.getByRole('searchbox').fill('no-such-row');
	await expect(page.getByText('No matching rows. Try another search.')).toBeVisible();
});

test('Live table keeps scroll, filter and sorting when results arrive; logs do not control activity', async ({
	page,
}) => {
	await page.goto('/');
	await page.getByLabel('What would you like to find?').fill('Find robots');
	await page.getByRole('button', { name: 'Start Research' }).click();
	await page.evaluate(() => {
		const f = (window as any).__uiFixture;
		f.emit('run:schema_proposed', { run_id: 'live', columns: f.schema });
	});
	await page.getByRole('button', { name: 'Confirm Schema' }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'running' });
	await page.evaluate(() => {
		const f = (window as any).__uiFixture;
		for (const row of f.rows)
			f.emit('run:row_added', {
				run_id: 'live',
				row_id: row.id,
				data: row.data,
				confidence: row.confidence,
			});
	});
	await page.getByRole('searchbox').fill('Robot');
	await page.getByRole('button', { name: 'Count', exact: true }).click();
	await page.locator('.table-scroll').evaluate((e) => (e.scrollTop = 20000));
	await expect
		.poll(() => page.locator('.data-row').first().getAttribute('aria-rowindex'))
		.not.toBe('2');
	const before = await page.locator('.table-scroll').evaluate((e) => e.scrollTop);
	await emit(page, 'run:row_added', {
		run_id: 'live',
		row_id: 'appended',
		data: { Name: 'Robot appended', Count: 1001 },
		confidence: 1,
	});
	expect(await page.locator('.table-scroll').evaluate((e) => e.scrollTop)).toBe(before);
	await expect(page.getByRole('searchbox')).toHaveValue('Robot');
	await expect(page.locator('th[aria-sort=ascending]')).toContainText('Count');
	await emit(page, 'run:log_entry', {
		run_id: 'live',
		level: 'INFO',
		role: 'query_expander',
		message: 'Expanding queries',
	});
	await expect(page.getByText('Expanding searches', { exact: true })).toBeVisible();
	await page.getByRole('button', { name: /Logs \(/ }).click();
	await expect(
		page.locator('.log-entry').filter({ hasText: '[query_expander] Expanding queries' })
	).toHaveCount(1);
	await page.getByRole('button', { name: 'Clear logs' }).click();
	await expect(page.getByText('Expanding searches', { exact: true })).toBeVisible();
	await emit(page, 'run:log_entry', {
		run_id: 'old-run',
		level: 'INFO',
		role: 'extractor',
		message: 'Unrelated run',
	});
	await expect(page.getByText('Expanding searches', { exact: true })).toBeVisible();
});

test('Research answer precedes collapsed activity and contains safe readable Markdown', async ({
	page,
}) => {
	await page.setViewportSize({ width: 900, height: 600 });
	await viewRun(page, 3);
	await expect(page.getByRole('region', { name: 'Research answer' })).toBeVisible();
	expect(await page.locator('.activity').getAttribute('open')).toBeNull();
	expect(
		await page
			.locator('.markdown ul')
			.first()
			.evaluate((e) => getComputedStyle(e).listStyleType)
	).toBe('disc');
	expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(900);
	expect(await page.evaluate(() => (window as any).unsafe)).toBeUndefined();
	await page.locator('.activity > summary').click();
	await page.locator('.activity li summary').first().click();
	await expect(page.locator('.step-content').first()).toContainText(
		'A full research step. '.repeat(30)
	);
});

test('Image preview ignores obsolete responses and displays decode failures', async ({ page }) => {
	await viewRun(page, 1);
	await page.evaluate(() => ((window as any).__uiFixture.delayImage = true));
	await page.getByRole('button', { name: 'Preview Robot image 0' }).click();
	await page.getByRole('button', { name: 'Next image' }).click();
	await expect
		.poll(() => page.evaluate(() => (window as any).__uiFixture.pendingImages.length))
		.toBe(2);
	await page.evaluate(() => {
		const f = (window as any).__uiFixture;
		f.pendingImages[1](f.images[1].thumbnail_url);
	});
	await expect(page.getByRole('dialog')).toHaveAccessibleName('Robot image 1');
	await page.evaluate(() => (window as any).__uiFixture.pendingImages[0]('data:bad'));
	await expect(page.locator('.preview-body img')).toHaveAttribute('src', /svg/);
	await page.locator('.preview-body img').dispatchEvent('error');
	await expect(page.getByRole('button', { name: 'Retry image' })).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(page.getByRole('dialog')).toHaveCount(0);
});

test('Export blocks every dismissal while saving and shows completion', async ({ page }) => {
	await viewRun(page, 0);
	await page.evaluate(() => ((window as any).__uiFixture.delayExport = true));
	await page.getByRole('button', { name: 'Export', exact: true }).click();
	const dialog = page.getByRole('dialog');
	await dialog.getByRole('button', { name: 'Export', exact: true }).click();
	await expect(dialog.getByRole('button', { name: 'Close', exact: true })).toBeDisabled();
	await page.keyboard.press('Escape');
	await expect(dialog).toBeVisible();
	await page.mouse.click(5, 5);
	await expect(dialog).toBeVisible();
	await page.evaluate(() => (window as any).__uiFixture.finishExport());
	await expect(dialog).toHaveAccessibleName('Export complete');
	await expect(dialog).toContainText('/tmp/query2table-fixture.csv');
	await dialog.getByRole('button', { name: 'Done' }).click();
});

test('History deletion supports cancellation and keeps the entry after failure', async ({
	page,
}) => {
	await page.goto('/history');
	await page.getByRole('button', { name: 'Delete', exact: true }).first().click();
	await page.getByRole('button', { name: 'Keep run' }).click();
	await expect(page.locator('.run-card')).toHaveCount(4);
	await page.evaluate(() => ((window as any).__uiFixture.failDelete = true));
	await page.getByRole('button', { name: 'Delete', exact: true }).first().click();
	await page.getByRole('button', { name: 'Delete run', exact: true }).click();
	await expect(page.getByRole('dialog')).toContainText('Local data');
	await expect(page.locator('.run-card')).toHaveCount(4);
});

test('Long links stay within the workspace in a small window', async ({ page }) => {
	await page.setViewportSize({ width: 900, height: 600 });
	await viewRun(page, 2);
	await page.getByRole('button', { name: /Logs \(/ }).click();
	await expect(page.locator('.link-card')).toContainText('Relevance 80%');
	expect(await page.locator('.link-list').evaluate((e) => e.scrollWidth <= e.clientWidth + 1)).toBe(
		true
	);
	await page.getByRole('button', { name: 'Copy link URL' }).click();
	await expect(page.getByRole('button', { name: 'Copied' })).toBeVisible();
	await page.getByRole('button', { name: 'Toggle theme' }).click();
	await expect(page.locator('html')).not.toHaveClass('dark');
});

test('Long schema scrolls independently, keeps actions accessible and preserves edits on pause', async ({
	page,
}) => {
	await page.setViewportSize({ width: 900, height: 600 });
	await page.goto('/');
	await page.getByLabel('What would you like to find?').fill('Find robots');
	await page.getByRole('button', { name: 'Start Research' }).click();
	const columns = Array.from({ length: 20 }, (_, i) => ({
		name: `Column ${i}`,
		type: 'text',
		description: 'Description',
		required: false,
	}));
	await emit(page, 'run:schema_proposed', { run_id: 'live', columns });
	await page.getByRole('button', { name: /Logs \(/ }).click();
	await page.getByLabel('Column 1 name', { exact: true }).fill('Edited name');
	await expect(page.getByRole('button', { name: 'Confirm Schema' })).toBeVisible();
	const button = await page.getByRole('button', { name: 'Confirm Schema' }).boundingBox();
	const dock = await page.locator('.log-panel').boundingBox();
	expect(button!.y + button!.height).toBeLessThanOrEqual(dock!.y);
	await page.getByRole('button', { name: 'Pause', exact: true }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'paused' });
	await expect(page.locator('.activity-dot.running')).toHaveCount(0);
	await page.getByRole('button', { name: 'Resume', exact: true }).click();
	await emit(page, 'run:status_changed', { run_id: 'live', status: 'schema_review' });
	await expect(page.getByLabel('Column 1 name', { exact: true })).toHaveValue('Edited name');
	await page.getByRole('button', { name: 'Confirm Schema' }).click();
	await expect(page.getByRole('button', { name: 'Confirming…' })).toBeDisabled();
});

test('Empty history results and image close during loading leave no stale dialog', async ({
	page,
}) => {
	await page.goto('/history');
	await page.evaluate(() => ((window as any).__uiFixture.rows = []));
	await page.getByRole('button', { name: 'View', exact: true }).first().click();
	await expect(page.getByText('No results found for this run.')).toBeVisible();
	await page.getByRole('button', { name: 'Back to History' }).click();
	await page.getByRole('button', { name: 'View', exact: true }).nth(1).click();
	await page.evaluate(() => ((window as any).__uiFixture.delayImage = true));
	await page.getByRole('button', { name: 'Preview Robot image 0' }).click();
	await expect
		.poll(() => page.evaluate(() => (window as any).__uiFixture.pendingImages.length))
		.toBe(1);
	await page.keyboard.press('Escape');
	await page.evaluate(() => (window as any).__uiFixture.pendingImages[0]('data:invalid'));
	await expect(page.getByRole('dialog')).toHaveCount(0);
});
