import { derived, get, readable, writable } from 'svelte/store';
import { getSetting, updateSetting } from '$lib/api/tauri';
import { persisted, readStorage, writeStorage } from '$lib/utils/storage';

export type ThemePreference = 'light' | 'dark' | 'system';

// static/theme-init.js reads the same key to apply the theme before the first paint.
const THEME_CACHE_KEY = 'q2t-theme';
const SIDEBAR_KEY = 'q2t-sidebar-collapsed';

function isThemePreference(value: unknown): value is ThemePreference {
	return value === 'light' || value === 'dark' || value === 'system';
}

export const sidebarCollapsed = persisted(SIDEBAR_KEY, false, (value) =>
	typeof value === 'boolean' ? value : null
);

const cachedTheme = readStorage(THEME_CACHE_KEY);
export const themePreference = writable<ThemePreference>(
	isThemePreference(cachedTheme) ? cachedTheme : 'system'
);

const darkQuery = '(prefers-color-scheme: dark)';
export const systemPrefersDark = readable(false, (set) => {
	if (typeof window === 'undefined' || !window.matchMedia) return;
	const media = window.matchMedia(darkQuery);
	set(media.matches);
	const update = (event: MediaQueryListEvent) => set(event.matches);
	media.addEventListener('change', update);
	return () => media.removeEventListener('change', update);
});

/** The theme actually applied to the page. */
export const currentTheme = derived(
	[themePreference, systemPrefersDark],
	([preference, prefersDark]): 'light' | 'dark' =>
		preference === 'system' ? (prefersDark ? 'dark' : 'light') : preference
);

export async function loadTheme() {
	const saved = await getSetting('theme');
	if (isThemePreference(saved)) {
		themePreference.set(saved);
		writeStorage(THEME_CACHE_KEY, saved);
	}
}

export async function setTheme(next: ThemePreference) {
	const previous = get(themePreference);
	if (previous === next) return;
	themePreference.set(next);
	writeStorage(THEME_CACHE_KEY, next);
	try {
		await updateSetting('theme', next);
	} catch (error) {
		if (get(themePreference) === next) {
			themePreference.set(previous);
			writeStorage(THEME_CACHE_KEY, previous);
		}
		throw error;
	}
}

const themeOrder: ThemePreference[] = ['light', 'dark', 'system'];

export function nextTheme(preference: ThemePreference): ThemePreference {
	return themeOrder[(themeOrder.indexOf(preference) + 1) % themeOrder.length];
}
