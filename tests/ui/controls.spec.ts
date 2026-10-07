import { test, expect, viewRun, emit } from './fixtures';

test('Lists are the app’s own: keyboard, search and theme colors', async ({ page }) => {
	await viewRun(page, 0);
	const column = page.getByRole('combobox', { name: 'Search in column' });
	await column.focus();
	await page.keyboard.press('ArrowDown');
	const list = page.getByRole('listbox');
	await expect(list).toBeVisible();
	// No native <select> is left anywhere in the app.
	await expect(page.locator('select')).toHaveCount(0);
	await page.keyboard.press('ArrowDown');
	await page.keyboard.press('Enter');
	await expect(list).toHaveCount(0);
	await expect(column).not.toHaveText('All columns');
	await expect(column).toBeFocused();
	await page.keyboard.press('Escape');

	await page.goto('/settings');
	const provider = page.getByRole('combobox', { name: 'Provider', exact: true });
	await provider.click();
	await expect(page.getByRole('option', { name: 'OpenRouter' })).toBeVisible();
	const background = await page.getByRole('listbox').evaluate((el) => getComputedStyle(el.parentElement!).backgroundColor);
	const panel = await page.evaluate(() => getComputedStyle(document.body).getPropertyValue('--app-panel'));
	expect(background).toBe(await page.evaluate((c) => {
		const probe = document.createElement('div');
		probe.style.color = c;
		document.body.append(probe);
		const value = getComputedStyle(probe).color;
		probe.remove();
		return value;
	}, panel.trim()));
	// A click outside closes the list without choosing.
	await page.getByRole('heading', { level: 1 }).click();
	await expect(page.getByRole('listbox')).toHaveCount(0);
});

test('Number fields step with their own buttons', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('button', { name: /Stop conditions/ }).click();
	const cost = page.getByLabel('Max cost (USD)');
	await cost.fill('1');
	const field = page.locator('.number-input', { has: cost });
	await field.getByRole('button', { name: 'Increase' }).click();
	await expect(cost).toHaveValue('1.01');
	await field.getByRole('button', { name: 'Decrease' }).click();
	await field.getByRole('button', { name: 'Decrease' }).click();
	await expect(cost).toHaveValue('0.99');
	await expect(page.getByRole('button', { name: /Stop conditions/ })).toContainText('$0.99');
});

test('Checkboxes, disclosures and tooltips are drawn by the app', async ({ page }) => {
	await viewRun(page, 0);
	await page.getByRole('button', { name: 'Columns' }).click();
	const option = page.getByRole('group', { name: 'Visible columns' }).locator('.checkbox').last();
	const input = option.getByRole('checkbox');
	await expect(input).toBeChecked();
	// The drawn box is what is seen and clicked; the real input stays invisible on top of it.
	expect(await input.evaluate((el) => getComputedStyle(el).opacity)).toBe('0');
	await option.locator('.box').click();
	await expect(input).not.toBeChecked();
	await page.keyboard.press('Escape');

	await page.getByRole('button', { name: 'Compact rows' }).hover();
	const tip = page.getByRole('tooltip');
	await expect(tip).toHaveText('Compact rows');
	await page.keyboard.press('Escape');
	await expect(tip).toBeHidden();

	const summary = page.locator('summary').first();
	expect(await summary.evaluate((el) => getComputedStyle(el).listStyleType)).toBe('none');
	expect(await summary.evaluate((el) => getComputedStyle(el, '::before').maskImage || getComputedStyle(el, '::before').webkitMaskImage)).toContain('svg');
});

test('Schema rows line up the type list and the Required checkbox with the fields', async ({ page }) => {
	await page.setViewportSize({ width: 1400, height: 800 });
	await page.goto('/');
	await page.getByLabel('What would you like to find?').fill('Find robots');
	await page.getByRole('button', { name: 'Build Table' }).click();
	await emit(page, 'run:schema_proposed', {
		run_id: 'live',
		columns: [{ name: 'name', type: 'text', description: 'Name', required: true }],
	});
	const name = await page.getByLabel('Column 1 name', { exact: true }).boundingBox();
	const type = await page.getByRole('combobox', { name: 'Column 1 type' }).boundingBox();
	const box = await page.locator('.column-row .checkbox .box').first().boundingBox();
	const center = (r: { y: number; height: number }) => r.y + r.height / 2;
	expect(Math.abs(type!.height - name!.height)).toBeLessThanOrEqual(1);
	expect(Math.abs(center(type!) - center(name!))).toBeLessThanOrEqual(1);
	expect(Math.abs(center(box!) - center(name!))).toBeLessThanOrEqual(2);
});
