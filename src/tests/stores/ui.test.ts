import { describe, it, expect, vi } from 'vitest';
import { get } from 'svelte/store';
import { sidebarCollapsed, currentTheme, toggleTheme } from '$lib/stores/ui';
import * as api from '$lib/api/tauri';

describe('ui store', () => {
	it('sidebar starts expanded', () => {
		expect(get(sidebarCollapsed)).toBe(false);
	});

	it('theme defaults to dark', () => {
		expect(get(currentTheme)).toBe('dark');
	});

	it('can toggle sidebar', () => {
		sidebarCollapsed.set(true);
		expect(get(sidebarCollapsed)).toBe(true);
		sidebarCollapsed.set(false);
		expect(get(sidebarCollapsed)).toBe(false);
	});

	it('restores the saved theme and reports a persistence failure', async () => {
		currentTheme.set('dark');
		const save = vi.spyOn(api, 'updateSetting').mockRejectedValueOnce(new Error('Database locked'));
		await expect(toggleTheme()).rejects.toThrow('Database locked');
		expect(get(currentTheme)).toBe('dark');
		save.mockRestore();
	});
});
