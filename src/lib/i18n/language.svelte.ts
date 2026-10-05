import { getSetting, updateSetting } from '$lib/api/tauri';
import { readStorage, writeStorage } from '$lib/utils/storage';
import { resolveLocale, setLocale, type LanguagePreference } from './state.svelte';

const CACHE_KEY = 'q2t-language';
const SETTING_KEY = 'ui_language';

function isPreference(value: unknown): value is LanguagePreference {
	return value === 'system' || value === 'en' || value === 'ru';
}

function systemLanguages(): readonly string[] {
	return typeof navigator === 'undefined' ? [] : (navigator.languages ?? [navigator.language]);
}

/** The language chosen in Settings. */
export const language = $state<{ preference: LanguagePreference }>({ preference: 'system' });

function apply(preference: LanguagePreference) {
	language.preference = preference;
	setLocale(resolveLocale(preference, systemLanguages()));
}

/** Applies the cached choice at once, then the one saved in settings. */
export async function loadLanguage() {
	const cached = readStorage(CACHE_KEY);
	apply(isPreference(cached) ? cached : 'system');
	const saved = await getSetting(SETTING_KEY);
	if (isPreference(saved)) {
		apply(saved);
		writeStorage(CACHE_KEY, saved);
	}
}

export async function setLanguage(preference: LanguagePreference) {
	const previous = language.preference;
	apply(preference);
	writeStorage(CACHE_KEY, preference);
	try {
		await updateSetting(SETTING_KEY, preference);
	} catch (error) {
		apply(previous);
		writeStorage(CACHE_KEY, previous);
		throw error;
	}
}
