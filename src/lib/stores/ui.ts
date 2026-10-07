import { derived, get, readable, writable } from 'svelte/store';
import { getSetting, updateSetting } from '$lib/api/tauri';
import { persisted, readStorage, writeStorage } from '$lib/utils/storage';
import { isUiScale, resolveTheme, themeById, themeFor, type ThemeMode, type UiScale } from '$lib/themes';

export type ThemePreference = 'light' | 'dark' | 'system';

// static/theme-init.js reads the same key to apply the theme before the first paint.
const THEME_CACHE_KEY = 'q2t-theme';
const THEME_LIGHT_KEY = 'q2t-theme-light';
const THEME_DARK_KEY = 'q2t-theme-dark';
const SCALE_KEY = 'q2t-ui-scale';
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

/** Themes used for the light and dark appearance. */
export const lightTheme = writable(themeFor('light', readStorage(THEME_LIGHT_KEY)));
export const darkTheme = writable(themeFor('dark', readStorage(THEME_DARK_KEY)));
const pair = { light: { store: lightTheme, key: THEME_LIGHT_KEY, setting: 'theme_light' }, dark: { store: darkTheme, key: THEME_DARK_KEY, setting: 'theme_dark' } };

const cachedScale = readStorage(SCALE_KEY);
export const uiScale = writable<UiScale>(isUiScale(cachedScale) ? cachedScale : 'normal');

/** The light or dark appearance actually applied to the page. */
export const currentTheme = derived(
	[themePreference, systemPrefersDark],
	([preference, prefersDark]): 'light' | 'dark' =>
		preference === 'system' ? (prefersDark ? 'dark' : 'light') : preference
);

/** Id of the color theme applied to the page. */
export const appliedTheme = derived(
	[themePreference, lightTheme, darkTheme, systemPrefersDark],
	([preference, light, dark, prefersDark]) => resolveTheme(preference, light, dark, prefersDark)
);

export async function loadTheme() {
	const [saved, light, dark, scale] = await Promise.all(
		['theme', 'theme_light', 'theme_dark', 'ui_scale'].map((key) => getSetting(key))
	);
	if (isThemePreference(saved)) {
		themePreference.set(saved);
		writeStorage(THEME_CACHE_KEY, saved);
	}
	for (const [mode, id] of [['light', light], ['dark', dark]] as const) {
		if (typeof id !== 'string') continue;
		const theme = themeFor(mode, id);
		pair[mode].store.set(theme);
		writeStorage(pair[mode].key, theme);
	}
	if (isUiScale(scale)) {
		uiScale.set(scale);
		writeStorage(SCALE_KEY, scale);
	}
}

/** Saves `value` locally at once and in settings, undoing the local change if saving fails. */
async function saveAppearance<T extends string>(store: { set(value: T): void }, current: T, value: T, key: string, setting: string) {
	store.set(value);
	writeStorage(key, value);
	try {
		await updateSetting(setting, value);
	} catch (error) {
		store.set(current);
		writeStorage(key, current);
		throw error;
	}
}

/** Uses a theme for its light or dark appearance and shows it, unless System is followed. */
export async function setThemeChoice(id: string) {
	const mode: ThemeMode | undefined = themeById(id)?.mode;
	if (!mode) return;
	const { store, key, setting } = pair[mode];
	const previous = get(store);
	if (get(themePreference) !== 'system' && get(themePreference) !== mode) await setTheme(mode);
	if (previous !== id) await saveAppearance(store, previous, id, key, setting);
}

export async function setUiScale(scale: UiScale) {
	const previous = get(uiScale);
	if (previous !== scale) await saveAppearance(uiScale, previous, scale, SCALE_KEY, 'ui_scale');
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
