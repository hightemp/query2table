import { test, expect } from './fixtures';

test('Attaching a picture in Images mode warns about comparing costs', async ({ page }) => {
	await page.addInitScript(() => {
		const fixture = (window as any).__uiFixture;
		fixture.values.vision_model = 'qwen-vl';
		fixture.dialogFiles = ['/docs/photo.png'];
	});
	await page.goto('/');
	await page.getByRole('button', { name: 'Images', exact: true }).click();
	await page.getByRole('button', { name: 'Attach files' }).click();
	await expect(page.getByRole('note')).toContainText('The 30 best found images');
	// Other modes do not compare pictures.
	await page.getByRole('button', { name: 'Links', exact: true }).click();
	await expect(page.getByRole('note')).toHaveCount(0);
});

test('Without a model for images the comparison is said to be skipped', async ({ page }) => {
	await page.addInitScript(() => ((window as any).__uiFixture.dialogFiles = ['/docs/photo.png']));
	await page.goto('/');
	await page.getByRole('button', { name: 'Images', exact: true }).click();
	await page.getByRole('button', { name: 'Attach files' }).click();
	await expect(page.getByRole('note')).toContainText('will not be compared');
});

test('The number of compared images comes from Settings, and 0 turns comparing off', async ({ page }) => {
	await page.addInitScript(() => {
		const fixture = (window as any).__uiFixture;
		fixture.values.vision_model = 'qwen-vl';
		fixture.values.image_compare_max = '0';
		fixture.dialogFiles = ['/docs/photo.png'];
	});
	await page.goto('/');
	await page.getByRole('button', { name: 'Images', exact: true }).click();
	await page.getByRole('button', { name: 'Attach files' }).click();
	await expect(page.getByRole('note')).toContainText('turned off in Settings');

	await page.goto('/settings');
	const search = page.locator('#settings-search');
	await expect(search.getByRole('spinbutton', { name: 'Images compared with a reference' })).toHaveValue('0');
	const llm = page.locator('#settings-llm');
	await expect(llm.getByRole('combobox', { name: 'Model context size' })).toHaveText('Medium (about 32K tokens)');
});
