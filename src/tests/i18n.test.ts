import { afterEach, describe, expect, it } from 'vitest';
import { en } from '$lib/i18n/en';
import { ru } from '$lib/i18n/ru';
import {
	formatDateTime,
	formatNumber,
	i18n,
	resolveLocale,
	setLocale,
	t,
} from '$lib/i18n';

afterEach(() => setLocale('en'));

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

	it('has the same messages and placeholders in every language', () => {
		const keys = Object.keys(en).sort();
		expect(Object.keys(ru).sort()).toEqual(keys);
		for (const key of keys) {
			const message = (ru as Record<string, unknown>)[key];
			expect(placeholders(message), key).toEqual(placeholders((en as Record<string, unknown>)[key]));
			expect(typeof message === 'string' ? message.trim() : 'plural', key).not.toBe('');
		}
	});

	it('follows the system language unless one is chosen', () => {
		expect(resolveLocale('system', ['ru-RU', 'en'])).toBe('ru');
		expect(resolveLocale('system', ['de-DE', 'en-US'])).toBe('en');
		expect(resolveLocale('system', ['uk-UA'])).toBe('en');
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
