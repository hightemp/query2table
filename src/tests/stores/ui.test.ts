import { describe, it, expect, vi, afterEach } from 'vitest';
import { get } from 'svelte/store';
import {
	sidebarCollapsed,
	themePreference,
	currentTheme,
	setTheme,
	loadTheme,
	nextTheme,
} from '$lib/stores/ui';
import * as api from '$lib/api/tauri';

describe('ui store', () => {
	afterEach(() => {
		vi.restoreAllMocks();
		themePreference.set('system');
	});

	it('sidebar starts expanded', () => {
		expect(get(sidebarCollapsed)).toBe(false);
	});

	it('remembers the sidebar state on this device', () => {
		sidebarCollapsed.set(true);
		expect(localStorage.getItem('q2t-sidebar-collapsed')).toBe('true');
		sidebarCollapsed.set(false);
		expect(localStorage.getItem('q2t-sidebar-collapsed')).toBe('false');
	});

	it('follows the system theme by default', () => {
		expect(get(themePreference)).toBe('system');
		// jsdom has no matchMedia, so the system theme reads as light.
		expect(get(currentTheme)).toBe('light');
	});

	it('applies an explicit theme and caches it for the next start', async () => {
		const save = vi.spyOn(api, 'updateSetting').mockResolvedValue();
		await setTheme('dark');
		expect(get(currentTheme)).toBe('dark');
		expect(localStorage.getItem('q2t-theme')).toBe('dark');
		expect(save).toHaveBeenCalledWith('theme', 'dark');
	});

	it('loads the saved preference, including system', async () => {
		vi.spyOn(api, 'getSetting').mockResolvedValue('dark');
		await loadTheme();
		expect(get(themePreference)).toBe('dark');
		vi.spyOn(api, 'getSetting').mockResolvedValue('system');
		await loadTheme();
		expect(get(themePreference)).toBe('system');
		expect(localStorage.getItem('q2t-theme')).toBe('system');
	});

	it('restores the previous theme and reports a persistence failure', async () => {
		themePreference.set('dark');
		vi.spyOn(api, 'updateSetting').mockRejectedValueOnce(new Error('Database locked'));
		await expect(setTheme('light')).rejects.toThrow('Database locked');
		expect(get(themePreference)).toBe('dark');
		expect(localStorage.getItem('q2t-theme')).toBe('dark');
	});

	it('cycles light → dark → system', () => {
		expect(nextTheme('light')).toBe('dark');
		expect(nextTheme('dark')).toBe('system');
		expect(nextTheme('system')).toBe('light');
	});
});
