import { en } from './en';
import { ru } from './ru';
import { isLocale, localeTag, type Locale } from './locales';

export type { Locale } from './locales';
export type LanguagePreference = 'system' | Locale;
export type PluralMessage = { zero?: string; one?: string; two?: string; few?: string; many?: string; other: string };
export type Message = string | PluralMessage;
export type MessageKey = keyof typeof en;
export type Params = Record<string, string | number>;

// English is the fallback and Russian the most used; the other catalogs load when chosen.
const catalogs: Partial<Record<Locale, Record<string, Message>>> = { en, ru };
const loaders = import.meta.glob<Record<string, Record<string, Message>>>([
	'./*.ts',
	'!./en.ts',
	'!./ru.ts',
	'!./index.ts',
	'!./locales.ts',
	'!./*.svelte.ts',
]);

/** The interface language; reading it inside a template or $derived makes that code follow changes. */
export const i18n = $state<{ locale: Locale }>({ locale: 'en' });

let wanted: Locale = 'en';

function show(locale: Locale) {
	i18n.locale = locale;
	if (typeof document !== 'undefined') document.documentElement.lang = locale;
}

/** Switches the language, loading its catalog first; at once when it is loaded already. */
export function setLocale(locale: Locale): Promise<void> {
	wanted = locale;
	const load = loaders[`./${locale}.ts`];
	if (catalogs[locale] || !load) {
		show(locale);
		return Promise.resolve();
	}
	return load().then((module) => {
		catalogs[locale] = module[locale];
		// A later choice made while this one loaded wins.
		if (wanted === locale) show(locale);
	});
}

/** The language to use for a preference, given the system's preferred languages. */
export function resolveLocale(preference: LanguagePreference, systemLanguages: readonly string[]): Locale {
	if (preference !== 'system') return preference;
	// The first system language the app speaks wins (by its main part: pt-PT is Portuguese); English otherwise.
	const known = systemLanguages.map((l) => l.toLowerCase().split(/[-_]/)[0]).find(isLocale);
	return known ?? 'en';
}

export function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
	return value.toLocaleString(localeTag(i18n.locale), options);
}

/** Date and time in the interface language; `seconds` is Unix time. */
export function formatDateTime(seconds: number, options?: Intl.DateTimeFormatOptions): string {
	return new Date(seconds * 1000).toLocaleString(
		localeTag(i18n.locale),
		options ?? { dateStyle: 'short', timeStyle: 'short' }
	);
}

/** BCP 47 tag for Intl APIs. */
export function intlLocale(): string {
	return localeTag(i18n.locale);
}

function pick(message: PluralMessage, count: number, locale: Locale): string {
	const form = new Intl.PluralRules(localeTag(locale)).select(count) as keyof PluralMessage;
	return message[form] ?? message.other;
}

/** Translates a message; `{name}` is replaced from params and `count` chooses the plural form. */
export function t(key: MessageKey, params?: Params): string {
	const locale = i18n.locale;
	const message = catalogs[locale]?.[key] ?? en[key];
	if (message === undefined) return key;
	const count = typeof params?.count === 'number' ? params.count : undefined;
	const text = typeof message === 'string' ? message : pick(message, count ?? 0, locale);
	if (!params) return text;
	return text.replace(/\{(\w+)\}/g, (match, name: string) => {
		const value = params[name];
		if (value === undefined) return match;
		return typeof value === 'number' ? formatNumber(value) : value;
	});
}
