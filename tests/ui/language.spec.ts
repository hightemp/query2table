import { test, expect, viewRun, choose } from './fixtures';

test('The language is chosen in Settings and applies at once', async ({ page }) => {
	await page.goto('/settings');
	await page.getByRole('spinbutton', { name: 'Temperature', exact: true }).fill('0.4');
	await choose(page.getByLabel('Language', { exact: true }), 'Русский');
	const nav = page.getByRole('navigation').first();
	await expect(nav.getByRole('link')).toHaveText(['Запрос', 'История', 'Настройки']);
	await expect(page.getByRole('heading', { level: 1 })).toHaveText('Настройки');
	await expect(page.locator('html')).toHaveAttribute('lang', 'ru');
	// Unsaved edits survive the switch, now described in Russian.
	await expect(page.getByRole('spinbutton', { name: 'Температура', exact: true })).toHaveValue('0.4');
	await expect(page.getByText('1 несохранённое изменение')).toBeVisible();
	const saved = await page.evaluate(() =>
		(window as any).__uiFixture.calls.filter((c: any) => c.command === 'update_setting')
	);
	expect(saved.at(-1).args).toEqual({ key: 'ui_language', value: 'ru' });

	await page.getByRole('button', { name: 'Отменить изменения' }).click();
	await page.reload();
	await expect(page.getByRole('heading', { level: 1 })).toHaveText('Настройки');
	await choose(page.getByLabel('Язык', { exact: true }), 'English');
	await expect(page.getByRole('heading', { level: 1 })).toHaveText('Settings');
});

test('Russian text uses plural forms and local dates', async ({ page }) => {
	await page.addInitScript(() => localStorage.setItem('q2t-language', 'ru'));
	await page.goto('/history');
	await expect(page.getByRole('heading', { level: 1 })).toHaveText('История прогонов');
	await expect(page.getByRole('group', { name: 'Тип прогона' }).getByRole('button')).toHaveText([
		'Все 4',
		'Таблица 1',
		'Картинки 1',
		'Ссылки 1',
		'Исследование 1',
	]);
	await page.getByRole('checkbox', { name: 'Выбрать Saved table research' }).check();
	await page.getByRole('checkbox', { name: 'Выбрать Saved links research' }).check();
	await expect(page.getByRole('toolbar', { name: 'Выбранные прогоны' })).toContainText('Выбрано: 2');
	await page.getByRole('toolbar', { name: 'Выбранные прогоны' }).getByRole('button', { name: 'Удалить' }).click();
	await expect(page.getByText('2 прогона удалено')).toBeVisible();

	await viewRun(page, 3);
	await expect(page.getByRole('tab', { name: /Ответ/ })).toBeVisible();
	await expect(page.getByRole('tab', { name: /Источники/ })).toBeVisible();
});

test.describe('with a Russian system', () => {
	test.use({ locale: 'ru-RU' });

	test('the system language is used until another is chosen', async ({ page }) => {
		await page.goto('/');
		await expect(page.getByRole('heading', { level: 1 })).toHaveText('Новый запрос');
		await expect(page.getByLabel('Что нужно найти?')).toBeVisible();
		await expect(page.getByRole('button', { name: 'Собрать таблицу' })).toBeVisible();
	});
});
