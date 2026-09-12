import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { sidebarCollapsed, currentTheme } from '$lib/stores/ui';

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
});
