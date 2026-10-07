import { describe, expect, it } from 'vitest';
import css from '../app.css?raw';
import { THEMES, COLOR_TOKENS, resolveTheme, themeById } from '$lib/themes';


/** Custom properties declared in the CSS block for one selector. */
function tokensOf(selector: string): [string, string][] {
	const start = css.indexOf(`\n${selector} {`);
	if (start < 0) return [];
	const body = css.slice(start, css.indexOf('}', start));
	return [...body.matchAll(/(--app-[a-z-]+):\s*([^;]+);/g)].map((m) => [m[1], m[2].trim()]);
}

function luminance(hex: string) {
	const [r, g, b] = [1, 3, 5].map((i) => {
		const c = parseInt(hex.slice(i, i + 2), 16) / 255;
		return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
	});
	return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a: string, b: string) {
	const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
	return (hi + 0.05) / (lo + 0.05);
}

describe('themes', () => {
	it('offers the default, Solarized, Nord, Dracula, Gruvbox and Rosé Pine themes', () => {
		expect(THEMES.filter((t) => t.mode === 'light').map((t) => t.id)).toEqual([
			'default-light',
			'solarized-light',
			'nord-light',
			'gruvbox-light',
			'rose-pine-dawn',
		]);
		expect(THEMES.filter((t) => t.mode === 'dark').map((t) => t.id)).toEqual([
			'default-dark',
			'solarized-dark',
			'nord',
			'dracula',
			'gruvbox-dark',
			'rose-pine',
		]);
	});

	it('defines the same colors in app.css', () => {
		for (const theme of THEMES) {
			const selector =
				theme.id === 'default-light' ? ':root' : theme.id === 'default-dark' ? ':root.dark' : `:root[data-theme='${theme.id}']`;
			const declared = tokensOf(selector).filter(([token]) => (COLOR_TOKENS as readonly string[]).includes(token));
			expect(Object.fromEntries(declared), theme.id).toEqual(theme.colors);
		}
	});

	it('keeps text readable in every theme', () => {
		for (const { id, colors } of THEMES) {
			for (const fg of ['--app-text', '--app-muted', '--app-accent', '--app-success', '--app-warning', '--app-danger'] as const)
				for (const bg of ['--app-bg', '--app-panel'] as const)
					expect(contrast(colors[fg], colors[bg]), `${id} ${fg} on ${bg}`).toBeGreaterThanOrEqual(4.5);
			expect(contrast(colors['--app-on-accent'], colors['--app-accent-solid']), id).toBeGreaterThanOrEqual(4.5);
		}
	});

	it('follows the light or dark choice, and the system for System', () => {
		const choice = { light: 'solarized-light', dark: 'dracula' } as const;
		expect(resolveTheme('light', choice.light, choice.dark, true)).toBe('solarized-light');
		expect(resolveTheme('dark', choice.light, choice.dark, false)).toBe('dracula');
		expect(resolveTheme('system', choice.light, choice.dark, true)).toBe('dracula');
		expect(resolveTheme('system', choice.light, choice.dark, false)).toBe('solarized-light');
		// Unknown or mismatched saved ids fall back to the defaults.
		expect(resolveTheme('dark', 'default-light', 'gone' as never, false)).toBe('default-dark');
		expect(resolveTheme('light', 'nord' as never, 'nord', false)).toBe('default-light');
		expect(themeById('nord')?.mode).toBe('dark');
	});
});
