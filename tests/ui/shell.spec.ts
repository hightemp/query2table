import { test, expect, viewRun, emit } from './fixtures';

const fixtureCalls = (page: import('@playwright/test').Page, command: string) =>
	page.evaluate(
		(command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command),
		command
	);

test('Saved theme is applied by a blocking script before the app renders', async ({ page }) => {
	const html = await (await page.request.get('/')).text();
	const themeScript = html.indexOf('theme-init.js');
	expect(themeScript).toBeGreaterThan(-1);
	expect(themeScript).toBeLessThan(html.indexOf('/_app/'));

	await page.goto('/');
	// Let the app finish applying the saved theme so it cannot race the checks below.
	await expect(page.getByRole('button', { name: 'Dark', exact: true })).toHaveAttribute(
		'aria-pressed',
		'true'
	);
	await page.emulateMedia({ colorScheme: 'dark' });
	const appliedFor = (saved: string | null) =>
		page.evaluate(async (saved) => {
			if (saved === null) localStorage.removeItem('q2t-theme');
			else localStorage.setItem('q2t-theme', saved);
			document.documentElement.classList.remove('dark');
			const script = document.createElement('script');
			script.src = `/theme-init.js?${Math.random()}`;
			await new Promise((resolve) => {
				script.onload = resolve;
				document.head.append(script);
			});
			return document.documentElement.classList.contains('dark');
		}, saved);
	expect(await appliedFor('light')).toBe(false);
	expect(await appliedFor('dark')).toBe(true);
	expect(await appliedFor('system')).toBe(true);
	expect(await appliedFor(null)).toBe(true);
});

test('Theme switch offers light, dark and system and follows the system setting', async ({
	page,
}) => {
	await page.goto('/');
	await expect(page.locator('html')).toHaveClass(/dark/);
	for (const name of ['Light', 'Dark', 'System'])
		expect(
			await page
				.getByRole('button', { name, exact: true })
				.evaluate((button) => button.scrollWidth <= button.clientWidth)
		).toBe(true);
	await page.getByRole('button', { name: 'Light', exact: true }).click();
	await expect(page.locator('html')).not.toHaveClass(/dark/);
	await expect(page.getByRole('button', { name: 'Light', exact: true })).toHaveAttribute(
		'aria-pressed',
		'true'
	);

	await page.getByRole('button', { name: 'System', exact: true }).click();
	await page.emulateMedia({ colorScheme: 'dark' });
	await expect(page.locator('html')).toHaveClass(/dark/);
	await page.emulateMedia({ colorScheme: 'light' });
	await expect(page.locator('html')).not.toHaveClass(/dark/);
	const saved = await fixtureCalls(page, 'update_setting');
	expect(saved.map((c: any) => c.args.value)).toEqual(['light', 'system']);

	// Collapsed sidebar: one button cycles the theme.
	await page.getByRole('button', { name: 'Toggle sidebar' }).click();
	await page.getByRole('button', { name: 'Theme: System. Switch to Light' }).click();
	await expect(page.getByRole('button', { name: 'Theme: Light. Switch to Dark' })).toBeVisible();
});

test('Theme save failure is reported in a toast and the theme is restored', async ({ page }) => {
	await page.goto('/');
	await page.evaluate(() => ((window as any).__uiFixture.failSave = true));
	await page.getByRole('button', { name: 'Light', exact: true }).click();
	await expect(page.getByRole('alert')).toContainText('Could not save the theme');
	await expect(page.locator('html')).toHaveClass(/dark/);
	await page.getByRole('button', { name: 'Dismiss' }).click();
	await expect(page.getByRole('alert')).toHaveCount(0);
});

test('Sidebar state survives a restart and keyboard shortcuts toggle sidebar and logs', async ({
	page,
}) => {
	await page.goto('/');
	await page.getByLabel('What would you like to find?').waitFor();
	await page.keyboard.press('Control+b');
	await expect(page.locator('.sidebar')).toHaveClass(/collapsed/);
	await page.reload();
	await expect(page.locator('.sidebar')).toHaveClass(/collapsed/);
	await page.getByLabel('What would you like to find?').waitFor();
	await page.keyboard.press('Control+b');
	await expect(page.locator('.sidebar')).not.toHaveClass(/collapsed/);

	await page.keyboard.press('Control+j');
	await expect(page.getByRole('button', { name: /Logs \(/ })).toHaveAttribute(
		'aria-expanded',
		'true'
	);
	await page.keyboard.press('Control+j');
	await expect(page.getByRole('button', { name: /Logs \(/ })).toHaveAttribute(
		'aria-expanded',
		'false'
	);
});

test('Logs show local time, problem counts, filters, copy and resizing', async ({ page }) => {
	await page.setViewportSize({ width: 1200, height: 800 });
	await page.goto('/');
	await page.getByLabel('What would you like to find?').fill('Find robots');
	await page.keyboard.press('Control+Enter');
	expect(await fixtureCalls(page, 'start_run')).toHaveLength(1);

	const entries = [
		{ run_id: 'live', level: 'INFO', role: 'planner', message: 'Planned 3 searches' },
		{ run_id: 'live', level: 'WARN', role: 'fetcher', message: 'Slow page' },
		{ run_id: 'live', level: 'ERROR', role: 'fetcher', message: 'Timeout fetching page' },
		{ run_id: 'old', level: 'ERROR', role: 'fetcher', message: 'Earlier failure' },
	];
	for (const entry of entries) await emit(page, 'run:log_entry', entry);

	// Problem counts are visible while the panel is closed.
	await expect(page.locator('.log-header')).toContainText('2 errors');
	await expect(page.locator('.log-header')).toContainText('1 warning');

	await page.getByRole('button', { name: /Logs \(/ }).click();
	await expect(page.locator('.log-entry')).toHaveCount(4);
	const time = await page.locator('.log-time').first().textContent();
	expect(time).toMatch(/^\d{2}:\d{2}:\d{2}$/);

	await page.getByLabel('Filter logs').fill('page');
	await expect(page.locator('.log-entry')).toHaveCount(2);
	await page.getByLabel('Current run').check();
	await page.getByLabel('Filter logs').fill('');
	await expect(page.locator('.log-entry')).toHaveCount(3);

	await page.getByRole('button', { name: 'Copy shown logs' }).click();
	const copied = await fixtureCalls(page, 'copy_text');
	expect(copied.at(-1).args.text.split('\n')).toHaveLength(3);
	await expect(page.locator('.toast')).toContainText('Copied 3 log entries');

	await page.getByRole('button', { name: 'Open log folder' }).click();
	expect((await fixtureCalls(page, 'open_app_folder')).at(-1).args.location).toBe('logs');

	const panel = page.locator('.log-panel');
	const before = (await panel.boundingBox())!.height;
	await page.getByRole('separator', { name: 'Resize logs' }).focus();
	await page.keyboard.press('ArrowUp');
	await page.keyboard.press('ArrowUp');
	await expect.poll(async () => (await panel.boundingBox())!.height).toBe(before + 48);
	await page.reload();
	await page.getByRole('button', { name: /Logs \(/ }).click();
	await expect.poll(async () => (await panel.boundingBox())!.height).toBe(before + 48);
	await page.getByRole('separator', { name: 'Resize logs' }).focus();
	await page.keyboard.press('Home');
	await expect.poll(async () => (await panel.boundingBox())!.height).toBe(before);
});

test('Copy failures appear as toasts instead of shifting the layout', async ({ page }) => {
	await viewRun(page, 2);
	await page.evaluate(() => ((window as any).__uiFixture.failCopy = true));
	await page.getByRole('button', { name: /^Copy link of / }).click();
	await expect(page.getByRole('alert')).toContainText('Could not copy');
	await expect(page.locator('.link-card [role=alert]')).toHaveCount(0);
});

test('Ctrl+F focuses the table search and Ctrl+S saves settings', async ({ page }) => {
	await viewRun(page, 0);
	await page.locator('.data-row').first().waitFor();
	await page.keyboard.press('Control+f');
	await expect(page.getByRole('searchbox', { name: 'Search results' })).toBeFocused();

	await page.goto('/settings');
	await page.getByRole('spinbutton', { name: 'Temperature', exact: true }).fill('0.4');
	await page.keyboard.press('Control+s');
	await expect(page.getByText('Changes saved', { exact: true })).toBeVisible();
	const [saved] = await fixtureCalls(page, 'update_settings');
	expect(saved.args.values).toEqual({ llm_temperature: '0.4' });
});
