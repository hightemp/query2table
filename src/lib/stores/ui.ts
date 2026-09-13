import { get, writable } from 'svelte/store';
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
	const previous = get(currentTheme);
	const next = previous === 'dark' ? 'light' : 'dark';
	currentTheme.set(next);
	try {
		await updateSetting('theme', next);
	} catch (error) {
		if (get(currentTheme) === next) currentTheme.set(previous);
		throw error;
	}
}
