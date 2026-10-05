import { en } from './en';
import { ru } from './ru';

export type Locale = 'en' | 'ru';
export type LanguagePreference = 'system' | Locale;
export type PluralMessage = { one?: string; few?: string; many?: string; other: string };
export type Message = string | PluralMessage;
export type MessageKey = keyof typeof en;
export type Params = Record<string, string | number>;

const catalogs: Record<Locale, Record<string, Message>> = { en, ru };

/** The interface language; reading it inside a template or $derived makes that code follow changes. */
export const i18n = $state<{ locale: Locale }>({ locale: 'en' });

export function setLocale(locale: Locale) {
	i18n.locale = locale;
	if (typeof document !== 'undefined') document.documentElement.lang = locale;
}

/** The language to use for a preference, given the system's preferred languages. */
export function resolveLocale(preference: LanguagePreference, systemLanguages: readonly string[]): Locale {
	if (preference !== 'system') return preference;
	// The first system language the app speaks wins; English otherwise.
	const known = systemLanguages.map((l) => l.toLowerCase().slice(0, 2)).find((l) => l === 'ru' || l === 'en');
	return known === 'ru' ? 'ru' : 'en';
}

export function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
	return value.toLocaleString(i18n.locale === 'ru' ? 'ru-RU' : 'en-US', options);
}

/** Date and time in the interface language; `seconds` is Unix time. */
export function formatDateTime(seconds: number, options?: Intl.DateTimeFormatOptions): string {
	return new Date(seconds * 1000).toLocaleString(
		i18n.locale === 'ru' ? 'ru-RU' : 'en-US',
		options ?? { dateStyle: 'short', timeStyle: 'short' }
	);
}

/** BCP 47 tag for Intl APIs. */
export function intlLocale(): string {
	return i18n.locale === 'ru' ? 'ru-RU' : 'en-US';
}

function pick(message: PluralMessage, count: number, locale: Locale): string {
	const form = new Intl.PluralRules(locale).select(count) as keyof PluralMessage;
	return message[form] ?? message.other;
}

/** Translates a message; `{name}` is replaced from params and `count` chooses the plural form. */
export function t(key: MessageKey, params?: Params): string {
	const locale = i18n.locale;
	const message = catalogs[locale][key] ?? catalogs.en[key];
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
