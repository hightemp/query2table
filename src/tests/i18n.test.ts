import { afterEach, describe, expect, it } from 'vitest';
import { en } from '$lib/i18n/en';
import { LOCALES } from '$lib/i18n/locales';
import {
	formatDateTime,
	formatNumber,
	i18n,
	resolveLocale,
	setLocale,
	t,
} from '$lib/i18n';

afterEach(() => setLocale('en'));

// Every translated catalog in the folder, by language code.
const catalogs: Record<string, Record<string, unknown>> = Object.fromEntries(
	Object.entries(import.meta.glob<Record<string, Record<string, unknown>>>('../lib/i18n/*.ts', { eager: true }))
		.map(([path, module]) => [path.split('/').pop()!.replace(/\.ts$/, ''), module])
		.filter(([code]) => LOCALES.some((l) => l.code === code && code !== 'en'))
		.map(([code, module]) => [code, (module as Record<string, Record<string, unknown>>)[code as string]])
);

/** Lines that read the same in every language: names, codes, units. */
const UNIVERSAL = /^(\{\w+\}|[A-Z0-9][\w.+\-/ ]*|[^\p{L}]*)$/u;

describe.each(Object.entries(catalogs))('the %s catalog', (code, catalog) => {
	const tag = LOCALES.find((l) => l.code === code)!.tag;
	const categories = new Intl.PluralRules(tag).resolvedOptions().pluralCategories;

	it('has the English keys and placeholders', () => {
		expect(Object.keys(catalog).sort()).toEqual(Object.keys(en).sort());
		for (const [key, english] of Object.entries(en)) {
			const message = catalog[key];
			expect(placeholders(message), key).toEqual(placeholders(english));
			if (typeof message === 'string') expect(message.trim(), key).not.toBe('');
		}
	});

	it('has the plural forms of the language', () => {
		for (const [key, english] of Object.entries(en)) {
			if (typeof english === 'string') continue;
			const message = catalog[key] as Record<string, string>;
			expect(typeof message, key).toBe('object');
			for (const category of categories) expect(message[category], `${key}.${category}`).toBeTruthy();
		}
	});

	it('leaves few lines in English', () => {
		if (code === 'ru') return;
		const same = Object.entries(en).filter(
			([key, english]) => typeof english === 'string' && catalog[key] === english && !UNIVERSAL.test(english) && english.split(' ').length > 1
		);
		expect(same.length, same.map(([key]) => key).join(', ')).toBeLessThanOrEqual(8);
	});
});


/** Every {placeholder} in a message or its plural forms. */
function placeholders(message: unknown): string[] {
	const text = typeof message === 'string' ? message : Object.values(message as object).join(' ');
	return [...new Set(text.match(/\{(\w+)\}/g) ?? [])].sort();
}

describe('i18n', () => {
	it('translates with parameters and falls back to English', () => {
		expect(t('nav.history')).toBe('History');
		setLocale('ru');
		expect(t('nav.history')).toBe('История');
		expect(i18n.locale).toBe('ru');
		expect(t('settings.unsavedChanges', { count: 1 })).toBe('1 несохранённое изменение');
		// Unknown keys show the key, so a missing translation is easy to spot.
		expect(t('no.such.key' as never)).toBe('no.such.key');
	});

	it('uses Russian plural forms', () => {
		setLocale('ru');
		expect(t('units.step', { count: 1 })).toBe('1 шаг');
		expect(t('units.step', { count: 3 })).toBe('3 шага');
		expect(t('units.step', { count: 5 })).toBe('5 шагов');
		expect(t('units.step', { count: 21 })).toBe('21 шаг');
		setLocale('en');
		expect(t('units.step', { count: 1 })).toBe('1 step');
		expect(t('units.step', { count: 1200 })).toBe('1,200 steps');
	});

	it('lists the interface languages with their own names', () => {
		expect(LOCALES.map((l) => l.code)).toEqual(['en', 'ru', 'de', 'fr', 'es', 'it', 'pt', 'pl', 'uk', 'tr', 'zh', 'ja', 'ko']);
		expect(LOCALES.find((l) => l.code === 'pt')).toMatchObject({ name: 'Português (Brasil)', tag: 'pt-BR' });
		expect(LOCALES.find((l) => l.code === 'zh')).toMatchObject({ name: '简体中文', tag: 'zh-CN' });
	});

	it('has a catalog for every language', () => {
		expect(Object.keys(catalogs).sort()).toEqual(LOCALES.map((l) => l.code).filter((c) => c !== 'en').sort());
	});

	it('follows the system language unless one is chosen', () => {
		expect(resolveLocale('system', ['ru-RU', 'en'])).toBe('ru');
		expect(resolveLocale('system', ['de-DE', 'en-US'])).toBe('de');
		expect(resolveLocale('system', ['uk-UA'])).toBe('uk');
		expect(resolveLocale('system', ['pt-PT'])).toBe('pt');
		expect(resolveLocale('system', ['zh-TW'])).toBe('zh');
		expect(resolveLocale('system', ['sv-SE', 'fr-CA'])).toBe('fr');
		expect(resolveLocale('system', ['sv-SE'])).toBe('en');
		expect(resolveLocale('en', ['ru-RU'])).toBe('en');
		expect(resolveLocale('ru', ['en-US'])).toBe('ru');
	});

	it('formats numbers and dates for the chosen language', () => {
		setLocale('ru');
		expect(formatNumber(1234.5).replace(/\s/g, ' ')).toBe('1 234,5');
		expect(formatDateTime(new Date(2026, 9, 3, 19, 54).getTime() / 1000)).toMatch(/03\.10\.2026.*19:54/);
		setLocale('en');
		expect(formatNumber(1234.5)).toBe('1,234.5');
	});
});
