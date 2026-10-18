import { describe, expect, it } from 'vitest';
import type { Token } from 'marked';
import { markdown, type MathToken } from '$lib/utils/math';
import { markdownToText } from '$lib/utils/research';
import { renderMath } from '$lib/utils/mathRender';
import { mathjax } from '@mathjax/src/mjs/mathjax.js';

function formulas(source: string): { text: string; display: boolean }[] {
	const found: { text: string; display: boolean }[] = [];
	markdown.walkTokens(markdown.lexer(source), (token: Token) => {
		if (token.type === 'math') found.push({ text: (token as MathToken).text, display: (token as MathToken).display });
	});
	return found;
}

describe('formulas in Markdown', () => {
	it('finds inline formulas in every delimiter', () => {
		expect(formulas('Energy \\(E = mc^2\\) and $a^2 + b^2$ here.')).toEqual([
			{ text: 'E = mc^2', display: false },
			{ text: 'a^2 + b^2', display: false },
		]);
	});

	it('finds display formulas as blocks and inside paragraphs', () => {
		expect(formulas('$$\n\\int_0^1 x\\,dx\n$$')).toEqual([{ text: '\\int_0^1 x\\,dx', display: true }]);
		expect(formulas('\\[\nx = 1\n\\]')).toEqual([{ text: 'x = 1', display: true }]);
		expect(formulas('So $$x_1 = 2$$ follows.')).toEqual([{ text: 'x_1 = 2', display: true }]);
		expect(formulas('Text\n$$\ny\n$$')).toEqual([{ text: 'y', display: true }]);
	});

	it('leaves prices and escaped dollars as text', () => {
		expect(formulas('From $5 to $10 a month.')).toEqual([]);
		expect(formulas('It costs $5, or $3 with a coupon.')).toEqual([]);
		expect(formulas('Plans: $5/$10.')).toEqual([]);
		expect(formulas('A \\$x\\$ sign and $ x $ spaced.')).toEqual([]);
		expect(markdownToText('From $5 to $10 and \\$x\\$.')).toBe('From $5 to $10 and $x$.');
	});

	it('never looks inside code', () => {
		expect(formulas('Run `echo $HOME$PATH` now.')).toEqual([]);
		expect(formulas('```\n$$x$$\n\\(y\\)\n```')).toEqual([]);
	});

	it('keeps LaTeX away from emphasis and escapes', () => {
		expect(formulas('$a_1 + b_1 = c_1$ and \\(x_i * y_i * z\\)')).toEqual([
			{ text: 'a_1 + b_1 = c_1', display: false },
			{ text: 'x_i * y_i * z', display: false },
		]);
	});

	it('finds formulas in lists and tables', () => {
		expect(formulas('- one $x^2$\n- two \\(y\\)')).toHaveLength(2);
		expect(formulas('| a | b |\n|---|---|\n| $x^2$ | \\(\\sqrt{2}\\) |')).toEqual([
			{ text: 'x^2', display: false },
			{ text: '\\sqrt{2}', display: false },
		]);
	});

	it('renders placeholders that keep the source until MathJax draws them', () => {
		const html = markdown.parse('Area $\\pi r^2$.\n\n$$\na < b\n$$', { async: false }) as string;
		expect(html).toContain('<span class="math math-inline" data-tex="\\pi r^2">$\\pi r^2$</span>');
		expect(html).toContain('<div class="math math-display" data-tex="a &lt; b" data-display="">$$\na &lt; b\n$$</div>');
	});

	it('copies formulas as LaTeX source', () => {
		expect(markdownToText('Area $\\pi r_1^2$ and \\(a_1 * b\\).\n\n$$\nx_1\n$$')).toBe(
			'Area $\\pi r_1^2$ and \\(a_1 * b\\).\n\n$$\nx_1\n$$'
		);
	});
});

describe('renderMath', () => {
	it('draws a formula as SVG in the text colour', async () => {
		const svg = await renderMath('\\frac{a}{b} + \\sqrt{x} = y + z', false);
		expect(svg).toMatch(/^<svg/);
		expect(svg).toContain('currentColor');
		expect(svg).not.toContain('<mjx-container');
		expect(svg.match(/<svg/g)).toHaveLength(1);
	});

	it('loads glyphs outside the main font from the bundle', async () => {
		const load = mathjax.asyncLoad!;
		const asked: string[] = [];
		mathjax.asyncLoad = (name: string) => (asked.push(name), load(name));
		try {
			const svg = await renderMath('A \u219C B', true);
			expect(svg).toMatch(/^<svg/);
			expect(asked.some((name) => name.endsWith('/arrows.js'))).toBe(true);
		} finally {
			mathjax.asyncLoad = load;
		}
	});

	it('rejects formulas that do not parse', async () => {
		await expect(renderMath('\\frac{a', false)).rejects.toThrow();
		await expect(renderMath('\\nosuchcommand{x}', false)).rejects.toThrow();
	});

	it('has no commands that make links or raw HTML', async () => {
		await expect(renderMath('\\href{javascript:alert(1)}{x}', false)).rejects.toThrow();
		await expect(renderMath('\\style{color:red}{x}', false)).rejects.toThrow();
	});
});
