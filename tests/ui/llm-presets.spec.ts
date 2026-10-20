import { test, expect, choose } from './fixtures';
import type { Page } from '@playwright/test';

const calls = (page: Page, command: string) =>
	page.evaluate(
		(command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command),
		command
	);
const provider = (page: Page) => page.getByRole('combobox', { name: 'Provider', exact: true });
const llm = (page: Page) => page.locator('#settings-llm');

test('A preset is a provider with its own key and model, kept when switching', async ({ page }) => {
	await page.goto('/settings');
	await choose(provider(page), 'DeepSeek');
	await expect(llm(page)).toContainText('Create one at platform.deepseek.com/api_keys.');
	// The address of a cloud service is fixed.
	await expect(llm(page).getByLabel('Server address')).toHaveCount(0);
	await llm(page).getByLabel('API key', { exact: true }).fill('ds-key');

	const model = page.getByRole('combobox', { name: 'Model', exact: true });
	await model.click();
	await page.getByRole('option', { name: 'local-model' }).click();
	await expect(model).toHaveValue('local-model');
	expect((await calls(page, 'list_openai_models')).at(-1).args).toEqual({
		baseUrl: 'https://api.deepseek.com/v1',
		apiKey: 'ds-key',
		provider: 'deepseek',
	});

	// Manual rates are scoped to the preset.
	await page.getByRole('checkbox', { name: 'Use custom rates for this model and endpoint' }).click();
	await expect(llm(page)).toContainText('local-model · DeepSeek');
	await page.getByRole('checkbox', { name: 'Use custom rates for this model and endpoint' }).click();

	// A local server has an address and needs no key.
	await choose(provider(page), 'LM Studio (local)');
	await expect(llm(page).getByLabel('Server address')).toHaveValue('http://localhost:1234/v1');
	await expect(llm(page)).toContainText('Leave empty if the server does not need one.');

	await choose(provider(page), 'DeepSeek');
	await expect(llm(page).getByLabel('API key', { exact: true })).toHaveValue('ds-key');
	await expect(model).toHaveValue('local-model');

	await page.getByRole('button', { name: 'Save', exact: true }).click();
	await expect(page.getByText('Changes saved', { exact: true })).toBeVisible();
	const [saved] = await calls(page, 'update_settings');
	expect(saved.args.values).toMatchObject({ llm_provider: 'deepseek', deepseek_api_key: 'ds-key', deepseek_model: 'local-model' });
	expect(saved.args.values).not.toHaveProperty('openai_api_key');
});

test('Readiness asks a cloud preset for its key', async ({ page }) => {
	await page.goto('/settings');
	await choose(provider(page), 'xAI');
	const status = page.getByRole('status', { name: 'Setup status' });
	await expect(status).toContainText('xAI needs API key and model.');
	await status.getByRole('link', { name: 'xAI needs API key and model.' }).click();
	await expect(llm(page).getByLabel('API key', { exact: true })).toBeFocused();
});

test('The custom address keeps the OpenAI-compatible server settings', async ({ page }) => {
	await page.goto('/settings');
	await choose(provider(page), 'Custom address (OpenAI-compatible)');
	await expect(llm(page).getByLabel('API base URL')).toBeVisible();
	await page.getByRole('combobox', { name: 'Model', exact: true }).click();
	await expect(page.getByRole('option', { name: 'local-model' })).toBeVisible();
	expect((await calls(page, 'list_openai_models')).at(-1).args.provider).toBeNull();
});
