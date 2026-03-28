import { writable } from 'svelte/store';
import { getSetting, updateSetting } from '$lib/api/tauri';

export const sidebarCollapsed = writable(false);
export const currentTheme = writable<'light' | 'dark'>('dark');

export async function loadTheme() {
	const saved = await getSetting('theme');
	if (saved === 'light' || saved === 'dark') {
		currentTheme.set(saved);
	}
}

export async function toggleTheme() {
	currentTheme.update((current) => {
		const next = current === 'dark' ? 'light' : 'dark';
		updateSetting('theme', next);
		return next;
	});
}
