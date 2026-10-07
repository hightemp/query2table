import { test, expect } from './fixtures';
import type { Page } from '@playwright/test';

const calls = (page: Page, command: string) =>
	page.evaluate(
		(command) => (window as any).__uiFixture.calls.filter((c: any) => c.command === command),
		command
	);
const nav = (page: Page) => page.getByRole('navigation', { name: 'Settings sections' });

test('Sections are listed on the left and follow the scroll', async ({ page }) => {
	await page.setViewportSize({ width: 1300, height: 800 });
	await page.goto('/settings');
	await expect(nav(page).getByRole('link')).toHaveText(['LLM', 'Search', 'Runs', 'Network', 'Application']);
	await expect(nav(page).getByRole('link', { name: 'LLM' })).toHaveAttribute('aria-current', 'true');
	await nav(page).getByRole('link', { name: 'Network' }).click();
	await expect(page.getByRole('heading', { name: 'Network', level: 2 })).toBeInViewport();
	await expect(nav(page).getByRole('link', { name: 'Network' })).toHaveAttribute('aria-current', 'true');
	const box = (await nav(page).boundingBox())!;
	const content = (await page.locator('.settings-scroll').boundingBox())!;
	expect(box.x + box.width).toBeLessThanOrEqual(content.x);
});

test('A narrow window picks sections from a menu', async ({ page }) => {
	await page.setViewportSize({ width: 900, height: 700 });
	await page.goto('/settings');
	await page.getByRole('combobox', { name: 'Settings section' }).selectOption('runs');
	await expect(page.getByRole('heading', { name: 'Runs', level: 2 })).toBeInViewport();
});

test('Readiness tells what is missing and leads to the field', async ({ page }) => {
	await page.goto('/settings');
	const status = page.getByRole('status', { name: 'Setup status' });
	await expect(status).toContainText('Ready for new runs');
	await page.getByLabel('Brave Search API key', { exact: true }).fill('');
	await expect(status).toContainText('Brave Search needs an API key.');
	await status.getByRole('link', { name: 'Brave Search needs an API key.' }).click();
	await expect(page.getByLabel('Brave Search API key', { exact: true })).toBeFocused();
});

test('Ranges use a slider and a number, and invalid values block Save', async ({ page }) => {
	await page.goto('/settings');
	const temperature = page.getByRole('spinbutton', { name: 'Temperature', exact: true });
	await page.getByRole('slider', { name: 'Temperature' }).fill('0.4');
	await expect(temperature).toHaveValue('0.4');
	await temperature.fill('1.5');
	await expect(page.getByText('Enter a number from 0 to 1.')).toBeVisible();
	await expect(temperature).toHaveAttribute('aria-invalid', 'true');
	await expect(page.getByRole('button', { name: 'Save', exact: true })).toBeDisabled();
	await expect(nav(page).getByRole('link', { name: 'LLM' })).toHaveAttribute('data-state', 'error');
	await temperature.fill('0.3');
	await expect(nav(page).getByRole('link', { name: 'LLM' })).toHaveAttribute('data-state', 'changed');
	await expect(page.getByRole('button', { name: 'Save', exact: true })).toBeEnabled();
});

test('Changes are saved together in one request', async ({ page }) => {
	await page.goto('/settings');
	const backup = page.getByRole('switch', { name: 'Use the other provider as backup' });
	await expect(backup).toHaveAttribute('aria-checked', 'true');
	await backup.click();
	await expect(backup).toHaveAttribute('aria-checked', 'false');
	await page.getByRole('spinbutton', { name: 'Temperature', exact: true }).fill('0.4');
	await expect(page.getByText('2 unsaved changes')).toBeVisible();
	await page.getByRole('button', { name: 'Save', exact: true }).click();
	await expect(page.getByText('Changes saved', { exact: true })).toBeVisible();
	const saves = await calls(page, 'update_settings');
	expect(saves).toHaveLength(1);
	expect(saves[0].args.values).toEqual({ search_fallback_enabled: 'false', llm_temperature: '0.4' });
	expect(await calls(page, 'update_setting')).toHaveLength(0);
});

test('Rare settings wait under Advanced and the search finds them', async ({ page }) => {
	await page.goto('/settings');
	const timeout = page.getByRole('spinbutton', { name: 'Page load timeout', exact: true });
	await expect(timeout).toBeHidden();
	const runs = page.locator('#settings-runs');
	await runs.getByRole('button', { name: /Advanced/ }).click();
	await expect(timeout).toBeVisible();
	await runs.getByRole('button', { name: /Advanced/ }).click();
	await page.getByRole('searchbox', { name: 'Search settings' }).fill('timeout');
	await expect(timeout).toBeVisible();
	await expect(page.getByRole('spinbutton', { name: 'Temperature', exact: true })).toBeHidden();
	await page.getByRole('searchbox', { name: 'Search settings' }).fill('nothing matches this');
	await expect(page.getByText('No settings match “nothing matches this”.')).toBeVisible();
});

test('Fields and sections can be reset to their defaults', async ({ page }) => {
	await page.goto('/settings');
	const temperature = page.getByRole('spinbutton', { name: 'Temperature', exact: true });
	await temperature.fill('0.5');
	await page.getByRole('button', { name: 'Reset Temperature to 0.7' }).click();
	await expect(temperature).toHaveValue('0.7');
	const runs = page.locator('#settings-runs');
	await page.getByRole('spinbutton', { name: 'Parallel page loads', exact: true }).fill('3');
	await runs.getByRole('button', { name: 'Reset section' }).click();
	await expect(page.getByRole('spinbutton', { name: 'Parallel page loads', exact: true })).toHaveValue('8');
	// Keys are never reset.
	await page.locator('#settings-search').getByRole('button', { name: 'Reset section' }).click();
	await expect(page.getByLabel('Brave Search API key', { exact: true })).toHaveValue('fixture-key');
});

test('Connections can be tested with the values in the form', async ({ page }) => {
	await page.goto('/settings');
	await page.locator('#settings-llm').getByRole('button', { name: 'Test connection' }).click();
	await expect(page.locator('#settings-llm')).toContainText('“deepseek-test” is available');
	const [llm] = await calls(page, 'test_llm_connection');
	expect(llm.args.settings.llm_provider).toBe('ollama_cloud');
	const search = page.locator('#settings-search');
	await expect(search).toContainText('Uses one search request.');
	await page.getByLabel('Brave Search API key', { exact: true }).fill('new-key');
	await search.getByRole('button', { name: 'Test connection' }).click();
	await expect(search).toContainText('Brave Search answered with 1 result.');
	const [checked] = await calls(page, 'test_search_connection');
	expect(checked.args.settings.brave_api_key).toBe('new-key');
});

test('Local Ollama and OpenAI-compatible servers list their models', async ({ page }) => {
	await page.goto('/settings');
	await page.getByRole('combobox', { name: 'Provider', exact: true }).selectOption('ollama');
	const model = page.getByRole('combobox', { name: 'Model', exact: true });
	await model.click();
	await page.getByRole('option', { name: 'qwen3:8b' }).click();
	await expect(model).toHaveValue('qwen3:8b');
	expect((await calls(page, 'list_ollama_models'))[0].args).toEqual({ baseUrl: 'http://localhost:11434' });
	await page.getByRole('combobox', { name: 'Provider', exact: true }).selectOption('openai_compatible');
	await page.getByRole('combobox', { name: 'Model', exact: true }).click();
	await expect(page.getByRole('option', { name: 'local-model' })).toBeVisible();
});

test('Passwords say what they show and proxies hide their passwords', async ({ page }) => {
	await page.goto('/settings');
	const key = page.getByLabel('Brave Search API key', { exact: true });
	await expect(key).toHaveAttribute('type', 'password');
	const reveal = page.getByRole('button', { name: 'Show Brave Search API key' });
	await reveal.click();
	await expect(key).toHaveAttribute('type', 'text');
	await expect(page.getByRole('button', { name: 'Hide Brave Search API key' })).toHaveAttribute('aria-pressed', 'true');

	await page.getByRole('button', { name: 'Add proxy' }).click();
	const url = page.getByRole('textbox', { name: 'Proxy URL' });
	await url.fill('http://user:secret@proxy.example:8080');
	await url.blur();
	await expect(url).toHaveValue('http://user:••••@proxy.example:8080');
	await url.focus();
	await expect(url).toHaveValue('http://user:secret@proxy.example:8080');
	await page.getByRole('button', { name: 'Test proxy' }).click();
	await expect(page.locator('#settings-network')).toContainText('The proxy works');
	expect((await calls(page, 'test_proxy'))[0].args).toEqual({ url: 'http://user:secret@proxy.example:8080' });
});

test('Application settings include the theme and files show full paths', async ({ page }) => {
	await page.goto('/settings');
	const theme = page.locator('#settings-app').getByRole('radiogroup', { name: 'Theme', exact: true });
	await theme.getByRole('radio', { name: 'Light' }).check();
	await expect(page.locator('html')).not.toHaveClass(/dark/);
	await theme.getByRole('radio', { name: 'Dark' }).check();
	await expect(page.locator('html')).toHaveClass(/dark/);
	const data = page.getByLabel('Application data', { exact: true });
	await expect(data).toHaveAttribute('title', /long-folder\/long-folder/);
});

test('Settings can be exported and imported as a file', async ({ page }) => {
	await page.goto('/settings');
	await page.getByRole('button', { name: 'Export settings…' }).click();
	expect((await calls(page, 'export_settings'))[0].args.path).toBe('/tmp/query2table-fixture.csv');
	await page.getByRole('button', { name: 'Import settings…' }).click();
	await expect(page.getByText('Imported 2 settings. Review them and save.')).toBeVisible();
	await expect(page.getByRole('spinbutton', { name: 'Temperature', exact: true })).toHaveValue('0.3');
	await expect(page.getByText('2 unsaved changes')).toBeVisible();
	expect(await calls(page, 'update_settings')).toHaveLength(0);
});

test('A saved value outside the range is flagged but does not block other changes', async ({ page }) => {
	await page.addInitScript(() => ((window as any).__uiFixture.values.max_parallel_fetches = '40'));
	await page.goto('/settings');
	await expect(page.getByText('Enter a whole number from 1 to 20.')).toBeVisible();
	await page.getByRole('spinbutton', { name: 'Temperature', exact: true }).fill('0.4');
	await expect(page.getByRole('button', { name: 'Save', exact: true })).toBeEnabled();
	// Changing it to another invalid value blocks Save until it is fixed.
	await page.getByRole('spinbutton', { name: 'Parallel page loads', exact: true }).fill('50');
	await expect(page.getByRole('button', { name: 'Save', exact: true })).toBeDisabled();
});
