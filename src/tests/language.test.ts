import { afterEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { i18n, language, loadLanguage, setLanguage, setLocale } from '$lib/i18n';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(async () => null) }));

afterEach(() => {
	setLocale('en');
	language.preference = 'system';
	localStorage.clear();
});

describe('interface language', () => {
	it('applies a chosen language at once and remembers it', async () => {
		await setLanguage('ru');
		expect(i18n.locale).toBe('ru');
		expect(language.preference).toBe('ru');
		expect(localStorage.getItem('q2t-language')).toBe('ru');
		expect(invoke).toHaveBeenCalledWith('update_setting', { key: 'ui_language', value: 'ru' });
		await setLanguage('en');
		expect(i18n.locale).toBe('en');
	});

	it('loads the saved language, cached first so the app does not flash English', async () => {
		localStorage.setItem('q2t-language', 'ru');
		vi.mocked(invoke).mockResolvedValueOnce('en' as never);
		const loading = loadLanguage();
		expect(i18n.locale).toBe('ru');
		await loading;
		expect(i18n.locale).toBe('en');
		expect(language.preference).toBe('en');
	});

	it('reverts when the choice cannot be saved', async () => {
		vi.mocked(invoke).mockRejectedValueOnce(new Error('sqlite: database is locked'));
		await expect(setLanguage('ru')).rejects.toThrow();
		expect(i18n.locale).toBe('en');
		expect(language.preference).toBe('system');
	});
});
