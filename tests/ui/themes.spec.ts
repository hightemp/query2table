import { test, expect } from './fixtures';

const saved = (page: import('@playwright/test').Page) =>
	page.evaluate(() => (window as any).__uiFixture.values);

test('Color themes are chosen for the light and dark appearance', async ({ page }) => {
	await page.goto('/settings');
	const html = page.locator('html');
	const dark = page.getByRole('radiogroup', { name: 'Dark theme' });
	await expect(dark.getByRole('radio')).toHaveCount(6);
	await expect(dark.getByRole('radio', { name: 'Default' })).toBeChecked();

	await dark.getByRole('radio', { name: 'Solarized Dark' }).check();
	await expect(html).toHaveAttribute('data-theme', 'solarized-dark');
	await expect(page.locator('body')).toHaveCSS('background-color', 'rgb(0, 43, 54)');
	expect((await saved(page)).theme_dark).toBe('solarized-dark');

	// Choosing a light theme while Dark is on shows it at once.
	const light = page.getByRole('radiogroup', { name: 'Light theme' });
	await light.getByRole('radio', { name: 'Gruvbox Light' }).check();
	await expect(html).not.toHaveClass(/dark/);
	await expect(html).toHaveAttribute('data-theme', 'gruvbox-light');
	await expect(page.getByRole('radiogroup', { name: 'Theme', exact: true }).getByRole('radio', { name: 'Light' })).toBeChecked();

	// System follows the pair.
	await page.emulateMedia({ colorScheme: 'dark' });
	await page.getByRole('radiogroup', { name: 'Theme', exact: true }).getByRole('radio', { name: 'System' }).check();
	await expect(html).toHaveAttribute('data-theme', 'solarized-dark');
	await page.emulateMedia({ colorScheme: 'light' });
	await expect(html).toHaveAttribute('data-theme', 'gruvbox-light');

	// The choice is restored (the fixture forgets saved values on reload, so seed them).
	await page.addInitScript(() =>
		Object.assign((window as any).__uiFixture.values, {
			theme: 'system',
			theme_light: 'gruvbox-light',
			theme_dark: 'solarized-dark',
		})
	);
	await page.reload();
	await expect(html).toHaveAttribute('data-theme', 'gruvbox-light');
	await expect(light.getByRole('radio', { name: 'Gruvbox Light' })).toBeChecked();
});

test('The interface size scales the text', async ({ page }) => {
	await page.goto('/settings');
	const heading = page.getByRole('heading', { level: 1 });
	const size = async () => parseFloat(await heading.evaluate((el) => getComputedStyle(el).fontSize));
	const normal = await size();
	const scale = page.getByRole('radiogroup', { name: 'Interface size' });
	await expect(scale.getByRole('radio', { name: 'Normal' })).toBeChecked();
	await scale.getByRole('radio', { name: 'Large' }).check();
	await expect(page.locator('html')).toHaveAttribute('data-scale', 'large');
	expect(await size()).toBeGreaterThan(normal);
	expect((await saved(page)).ui_scale).toBe('large');
	await scale.getByRole('radio', { name: 'Compact' }).check();
	expect(await size()).toBeLessThan(normal);
	await page.addInitScript(() => ((window as any).__uiFixture.values.ui_scale = 'compact'));
	await page.reload();
	await expect(page.locator('html')).toHaveAttribute('data-scale', 'compact');
});

test('A theme that cannot be saved is reported and undone', async ({ page }) => {
	await page.goto('/settings');
	await page.evaluate(() => ((window as any).__uiFixture.failSave = true));
	await page.getByRole('radiogroup', { name: 'Dark theme' }).getByRole('radio', { name: 'Dracula' }).click();
	await expect(page.getByRole('alert')).toContainText('Could not save the theme');
	await expect(page.locator('html')).toHaveAttribute('data-theme', 'default-dark');
});
