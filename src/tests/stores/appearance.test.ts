import { afterEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { darkTheme, lightTheme, setThemeChoice, setUiScale, themePreference, uiScale } from '$lib/stores/ui';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(async () => null) }));

afterEach(() => {
	lightTheme.set('default-light');
	darkTheme.set('default-dark');
	themePreference.set('system');
	uiScale.set('normal');
	localStorage.clear();
	vi.mocked(invoke).mockReset().mockResolvedValue(null as never);
});

describe('appearance settings', () => {
	it('saves the chosen dark theme and shows it at once', async () => {
		themePreference.set('light');
		await setThemeChoice('solarized-dark');
		expect(get(darkTheme)).toBe('solarized-dark');
		// Choosing a dark theme while the light one is shown switches to dark.
		expect(get(themePreference)).toBe('dark');
		expect(localStorage.getItem('q2t-theme-dark')).toBe('solarized-dark');
		expect(invoke).toHaveBeenCalledWith('update_setting', { key: 'theme_dark', value: 'solarized-dark' });
	});

	it('keeps System when a theme is chosen for it', async () => {
		themePreference.set('system');
		await setThemeChoice('gruvbox-light');
		expect(get(lightTheme)).toBe('gruvbox-light');
		expect(get(themePreference)).toBe('system');
	});

	it('reverts when the choice cannot be saved', async () => {
		vi.mocked(invoke).mockRejectedValueOnce(new Error('sqlite: database is locked'));
		await expect(setThemeChoice('nord')).rejects.toThrow();
		expect(get(darkTheme)).toBe('default-dark');
	});

	it('saves the interface size', async () => {
		await setUiScale('large');
		expect(get(uiScale)).toBe('large');
		expect(localStorage.getItem('q2t-ui-scale')).toBe('large');
		expect(invoke).toHaveBeenCalledWith('update_setting', { key: 'ui_scale', value: 'large' });
	});
});
