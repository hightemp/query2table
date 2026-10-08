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
	await expect(page.getByRole('note')).toContainText('compared with the attached picture: up to 30');
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
