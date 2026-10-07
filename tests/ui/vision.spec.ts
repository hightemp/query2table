import { test, expect, choose } from './fixtures';

const calls = (page: import('@playwright/test').Page, command: string) =>
	page.evaluate((command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command), command);

test('Settings tell whether the model sees images and offer a model for images', async ({ page }) => {
	await page.goto('/settings');
	const llm = page.locator('#settings-llm');
	await expect(llm.getByRole('heading', { name: 'Images' })).toBeVisible();
	const sees = llm.getByRole('combobox', { name: 'Model sees images' });
	await expect(sees).toHaveText('Auto');
	await expect(llm.getByText('deepseek-test cannot see images, according to Ollama Cloud.')).toBeVisible();
	await expect(llm.getByText('Choose a model for images to read pictures and scanned pages.')).toBeVisible();

	const reader = llm.getByRole('combobox', { name: 'Model for images', exact: true });
	await reader.fill('qwen-vl');
	await expect(llm.getByText('Pictures and scanned pages go to qwen-vl.')).toBeVisible();
	await page.getByRole('button', { name: 'Save' }).click();
	expect((await calls(page, 'update_settings')).at(-1).args.values).toMatchObject({ vision_model: 'qwen-vl' });

	// The switch overrides what the catalog says.
	await choose(sees, 'Yes');
	await expect(llm.getByText('deepseek-test reads pictures and scanned pages itself.')).toBeVisible();
	expect((await calls(page, 'get_vision_status')).at(-1).args.overrides).toMatchObject({ llm_vision: 'on' });
});

test('An unknown model can be marked as seeing images', async ({ page }) => {
	await page.addInitScript(() => ((window as any).__uiFixture.visionDetected = null));
	await page.goto('/settings');
	await expect(
		page.getByText('Ollama Cloud does not say whether deepseek-test sees images. Choose “Yes” if it does.')
	).toBeVisible();
});

test('Pictures and scans warn when no model can read them', async ({ page }) => {
	await page.addInitScript(() => ((window as any).__uiFixture.dialogFiles = ['/docs/photo.png', '/docs/scan.pdf']));
	await page.goto('/');
	await page.getByRole('button', { name: 'Attach files' }).click();
	const files = page.getByRole('list', { name: 'Attached files' });
	await expect(files.getByRole('listitem').nth(0)).toContainText('The model cannot see images');
	await files.getByRole('listitem').nth(1).locator('.warning').hover();
	await expect(page.getByRole('tooltip')).toContainText('No model for images is set up');
	await expect(page.getByRole('link', { name: 'Choose a model for images' })).toHaveAttribute('href', '/settings#settings-llm');
});

test('With a model for images, pictures attach without warnings', async ({ page }) => {
	await page.addInitScript(() => {
		const fixture = (window as any).__uiFixture;
		fixture.values.vision_model = 'qwen-vl';
		fixture.dialogFiles = ['/docs/photo.png'];
	});
	await page.goto('/');
	await page.getByRole('button', { name: 'Attach files' }).click();
	const files = page.getByRole('list', { name: 'Attached files' });
	await expect(files.getByRole('listitem')).toContainText('2.4 MB');
	await expect(files.getByRole('listitem')).not.toContainText('cannot see images');
	await expect(page.getByRole('link', { name: 'Choose a model for images' })).toHaveCount(0);
});
