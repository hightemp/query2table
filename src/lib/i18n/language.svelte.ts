import { getSetting, updateSetting } from '$lib/api/tauri';
import { readStorage, writeStorage } from '$lib/utils/storage';
import { resolveLocale, setLocale, type LanguagePreference } from './state.svelte';
import { isLocale } from './locales';

const CACHE_KEY = 'q2t-language';
const SETTING_KEY = 'ui_language';

function isPreference(value: unknown): value is LanguagePreference {
	return value === 'system' || isLocale(value);
}

function systemLanguages(): readonly string[] {
	return typeof navigator === 'undefined' ? [] : (navigator.languages ?? [navigator.language]);
}

/** The language chosen in Settings. */
export const language = $state<{ preference: LanguagePreference }>({ preference: 'system' });

function apply(preference: LanguagePreference): Promise<void> {
	language.preference = preference;
	return setLocale(resolveLocale(preference, systemLanguages()));
}

/** Before the first render: the cached language with its catalog, so the app does not flash English. */
export async function preloadLanguage() {
	const cached = readStorage(CACHE_KEY);
	await apply(isPreference(cached) ? cached : 'system').catch(() => {});
}

/** Applies the cached choice at once, then the one saved in settings. */
export async function loadLanguage() {
	const cached = readStorage(CACHE_KEY);
	apply(isPreference(cached) ? cached : 'system');
	const saved = await getSetting(SETTING_KEY);
	if (isPreference(saved)) {
		await apply(saved);
		writeStorage(CACHE_KEY, saved);
	}
}

export async function setLanguage(preference: LanguagePreference) {
	const previous = language.preference;
	await apply(preference);
	writeStorage(CACHE_KEY, preference);
	try {
		await updateSetting(SETTING_KEY, preference);
	} catch (error) {
		await apply(previous);
		writeStorage(CACHE_KEY, previous);
		throw error;
	}
}
