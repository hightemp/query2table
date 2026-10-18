import { Marked, type TokenizerExtension, type RendererExtension, type Tokens } from 'marked';

/** A LaTeX formula: `\(…\)` or `$…$` inline, `$$…$$` or `\[…\]` displayed. */
export interface MathToken extends Tokens.Generic {
	type: 'math';
	raw: string;
	/** LaTeX without delimiters. */
	text: string;
	display: boolean;
	/** A display formula standing as its own block, not inside a paragraph. */
	block: boolean;
}

const BLOCK = /^ {0,3}(?:\$\$([\s\S]+?)\$\$|\\\[([\s\S]+?)\\\])[ \t]*(?:\n+|$)/;
const PAIRED = /^(?:\\\(([\s\S]+?)\\\)|\\\[([\s\S]+?)\\\]|\$\$([\s\S]+?)\$\$)/;

function token(raw: string, tex: string, display: boolean, block = false): MathToken | undefined {
	const text = tex.trim();
	return text ? { type: 'math', raw, text, display, block } : undefined;
}

/**
 * `$…$` by pandoc's rule, so prices stay text: no space after the opening `$`, no space
 * before the closing one, and no digit right after it.
 */
function dollarLength(src: string): number {
	if (src[0] !== '$' || !src[1] || /[\s$]/.test(src[1])) return 0;
	for (let i = 2; i < src.length; i++) {
		if (src[i] === '\\') {
			i++;
			continue;
		}
		if (src[i] === '\n' && src[i + 1] === '\n') return 0;
		if (src[i] === '$' && !/\s/.test(src[i - 1]) && !/[0-9$]/.test(src[i + 1] ?? '')) return i + 1;
	}
	return 0;
}

const mathBlock: TokenizerExtension & RendererExtension = {
	name: 'math',
	level: 'block',
	start: (src) => src.match(/(?:^|\n) {0,3}(?:\$\$|\\\[)/)?.index,
	tokenizer(src) {
		const match = BLOCK.exec(src);
		return match ? token(match[0], match[1] ?? match[2], true, true) : undefined;
	},
	renderer: (t) => render(t as MathToken),
};

const mathInline: TokenizerExtension = {
	name: 'math',
	level: 'inline',
	start: (src) => src.match(/\\[([]|(?<!\\)\$/)?.index,
	tokenizer(src) {
		const paired = PAIRED.exec(src);
		if (paired) return token(paired[0], paired[1] ?? paired[2] ?? paired[3], !paired[1]);
		const length = dollarLength(src);
		return length ? token(src.slice(0, length), src.slice(1, length - 1), false) : undefined;
	},
};

function escape(text: string): string {
	return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

/** A placeholder that shows the source until `renderMath` draws it. */
function render({ raw, text, display, block }: MathToken): string {
	const source = escape(raw.trim());
	return block
		? `<div class="math math-display" data-tex="${escape(text)}" data-display="">${source}</div>`
		: display
			? `<span class="math math-display" data-tex="${escape(text)}" data-display="">${source}</span>`
			: `<span class="math math-inline" data-tex="${escape(text)}">${source}</span>`;
}

/** Markdown with LaTeX formulas; formulas render as placeholders for `renderMath`. */
export const markdown = new Marked({ extensions: [mathBlock, mathInline] });
