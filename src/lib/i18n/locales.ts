/** Interface languages: the code of the catalog (`<code>.ts`), the name in the language itself, and the Intl tag. */
export const LOCALES = [
	{ code: 'en', name: 'English', tag: 'en-US' },
	{ code: 'ru', name: 'Русский', tag: 'ru-RU' },
	{ code: 'de', name: 'Deutsch', tag: 'de-DE' },
	{ code: 'fr', name: 'Français', tag: 'fr-FR' },
	{ code: 'es', name: 'Español', tag: 'es-ES' },
	{ code: 'it', name: 'Italiano', tag: 'it-IT' },
	{ code: 'pt', name: 'Português (Brasil)', tag: 'pt-BR' },
	{ code: 'pl', name: 'Polski', tag: 'pl-PL' },
	{ code: 'uk', name: 'Українська', tag: 'uk-UA' },
	{ code: 'tr', name: 'Türkçe', tag: 'tr-TR' },
	{ code: 'zh', name: '简体中文', tag: 'zh-CN' },
	{ code: 'ja', name: '日本語', tag: 'ja-JP' },
	{ code: 'ko', name: '한국어', tag: 'ko-KR' },
] as const;

export type Locale = (typeof LOCALES)[number]['code'];

export function isLocale(value: unknown): value is Locale {
	return LOCALES.some((locale) => locale.code === value);
}

export function localeTag(locale: Locale): string {
	return LOCALES.find((l) => l.code === locale)?.tag ?? 'en-US';
}
