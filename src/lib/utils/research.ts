import { t, type MessageKey } from '$lib/i18n';
import { marked, type Token, type Tokens } from 'marked';
import type { ResearchStep } from '$lib/types';
import { displayHost } from './hosts';
import { markdownLink } from './linkMenu';

export type StepKind = 'search' | 'fetch' | 'think' | 'error';

const STEP_KINDS = ['search', 'fetch', 'think', 'error'];

export interface StepView {
	step: ResearchStep;
	/** One-based position in the run. */
	number: number;
	label: string;
	/** One line describing the step. */
	summary: string;
	domain: string | null;
	/** Page text saved by older runs, or a raw error; shown only on request. */
	rawText: string | null;
	rawKind: 'page' | 'details';
}

const READ_COUNT = /^Read page \(\d+ characters of content\)\.?$/;

function parseUrl(url: string | null | undefined): URL | null {
	if (!url) return null;
	try {
		return new URL(url);
	} catch {
		return null;
	}
}

/** "proxy-seller.me › malaysia" — host and the first path segments. */
export function pageLabel(url: URL): string {
	const host = displayHost(url.hostname);
	const path = url.pathname.replace(/^\/+|\/+$/g, '');
	let readable = path;
	try {
		readable = decodeURIComponent(path);
	} catch {
		// Keep the encoded path.
	}
	return readable ? `${host} › ${readable.split('/').slice(0, 3).join('/')}` : host;
}

/** How the research pipeline records a model reply it could not use. */
const INVALID_REPLY = /^The model produced an invalid response:/;

export function describeStep(step: ResearchStep, index: number): StepView {
	const url = parseUrl(step.url);
	const domain = url ? displayHost(url.hostname) : null;
	const content = step.content.trim();
	let summary = content;
	let rawText: string | null = null;
	let label = STEP_KINDS.includes(step.step_type) ? t(`research.step.${step.step_type}` as MessageKey) : step.step_type;
	if (step.step_type === 'error' && INVALID_REPLY.test(content)) {
		label = t('research.step.unreadable');
		summary = t('research.step.unreadableSummary');
		rawText = step.content;
	} else if (step.step_type === 'fetch') {
		if (!content || READ_COUNT.test(content)) summary = url ? pageLabel(url) : t('research.page');
		else if (content.includes('\n') || content.length > 300) {
			// Older runs stored the page text itself; its first line is the page title.
			summary = content.split('\n').find((line) => line.trim())?.trim().slice(0, 160) ?? '';
			rawText = step.content;
		}
	}
	return {
		step,
		number: index + 1,
		label,
		summary,
		domain,
		rawText,
		rawKind: step.step_type === 'fetch' ? 'page' : 'details',
	};
}

export interface ResearchSource {
	url: string;
	title: string;
	domain: string;
	/** The agent opened the page. */
	read: boolean;
	/** The answer links to the page. */
	cited: boolean;
}

/** Comparable form of a URL: no fragment, lower-case host, no trailing slash. */
function sourceKey(url: URL): string {
	return `${url.protocol}//${url.host.toLowerCase()}${url.pathname.replace(/\/+$/, '')}${url.search}`;
}

/** The address as written, without its fragment. */
function withoutHash(raw: string): string {
	return raw.replace(/#.*$/, '');
}

/** Links in Markdown text, in order: `[title](url)` and bare http(s) addresses. */
export function answerLinks(markdown: string): { url: URL; raw: string; title: string }[] {
	const found: { url: URL; raw: string; title: string; at: number }[] = [];
	const linked = /\[([^\]]*)\]\((https?:\/\/[^)\s]+)\)/g;
	for (const match of markdown.matchAll(linked)) {
		const url = parseUrl(match[2]);
		if (url) found.push({ url, raw: match[2], title: match[1].trim(), at: match.index ?? 0 });
	}
	const bare = /(?<![(\[\w])https?:\/\/[^\s)<>\]"'`]+/g;
	for (const match of markdown.matchAll(bare)) {
		const raw = match[0].replace(/[.,;:!?]+$/, '');
		const url = parseUrl(raw);
		if (url) found.push({ url, raw, title: '', at: match.index ?? 0 });
	}
	return found.sort((a, b) => a.at - b.at);
}

export function collectSources(steps: ResearchStep[], answer: string | null): ResearchSource[] {
	const sources = new Map<string, ResearchSource>();
	for (const { url, raw, title } of answer ? answerLinks(answer) : []) {
		const key = sourceKey(url);
		const existing = sources.get(key);
		if (existing) {
			if (!existing.title && title) existing.title = title;
			continue;
		}
		sources.set(key, {
			url: withoutHash(raw),
			title,
			domain: displayHost(url.hostname),
			read: false,
			cited: true,
		});
	}
	for (const step of steps) {
		if (step.step_type !== 'fetch') continue;
		const url = parseUrl(step.url);
		if (!url) continue;
		const view = describeStep(step, 0);
		const pageTitle = READ_COUNT.test(step.content.trim()) ? '' : view.summary;
		const key = sourceKey(url);
		const existing = sources.get(key);
		if (existing) {
			existing.read = true;
			if (pageTitle) existing.title = pageTitle;
		} else
			sources.set(key, {
				url: withoutHash(step.url!),
				title: pageTitle,
				domain: displayHost(url.hostname),
				read: true,
				cited: false,
			});
	}
	return [...sources.values()].map((source) => ({
		...source,
		title: source.title || source.domain,
	}));
}

/** Anchor text for a heading; must match the ids rendered by `Markdown.svelte`. */
export function slugify(text: string): string {
	return text
		.toLowerCase()
		.replace(/[^\p{L}\p{N}]+/gu, '-')
		.replace(/^-|-$/g, '');
}

/** Gives repeated slugs a numeric suffix: "part", "part-2", … */
export function uniqueSlugger() {
	const seen = new Map<string, number>();
	return (text: string) => {
		const base = slugify(text) || 'section';
		const count = (seen.get(base) ?? 0) + 1;
		seen.set(base, count);
		return count === 1 ? base : `${base}-${count}`;
	};
}

export interface AnswerHeading {
	depth: number;
	text: string;
	slug: string;
}

/** Text of inline tokens without formatting, e.g. for heading anchors. */
export function plainInline(tokens: Token[] | undefined): string {
	return (tokens ?? [])
		.map((token) => {
			if ('tokens' in token && token.tokens) return plainInline(token.tokens);
			return 'text' in token ? String(token.text) : '';
		})
		.join('');
}

/** Headings of levels 1–3 in document order, for the contents list. */
export function answerHeadings(markdown: string): AnswerHeading[] {
	const slug = uniqueSlugger();
	const headings: AnswerHeading[] = [];
	for (const token of marked.lexer(markdown)) {
		if (token.type !== 'heading') continue;
		const text = plainInline((token as Tokens.Heading).tokens).trim();
		const id = slug(text);
		if (token.depth <= 3) headings.push({ depth: token.depth, text, slug: id });
	}
	return headings;
}

function inlineText(tokens: Token[] | undefined): string {
	return (tokens ?? [])
		.map((token) => {
			switch (token.type) {
				case 'link': {
					const link = token as Tokens.Link;
					const label = inlineText(link.tokens);
					return label && label !== link.href ? `${label} (${link.href})` : link.href;
				}
				case 'br':
					return '\n';
				case 'image':
					return (token as Tokens.Image).text;
				default:
					if ('tokens' in token && token.tokens) return inlineText(token.tokens);
					return 'text' in token ? String(token.text) : '';
			}
		})
		.join('');
}

function blockText(tokens: Token[], indent = ''): string[] {
	const blocks: string[] = [];
	for (const token of tokens) {
		switch (token.type) {
			case 'heading':
			case 'paragraph':
				blocks.push(indent + inlineText((token as Tokens.Paragraph).tokens));
				break;
			case 'list': {
				const list = token as Tokens.List;
				blocks.push(
					list.items
						.map((item, i) => {
							const marker = list.ordered ? `${Number(list.start || 1) + i}. ` : '- ';
							return indent + marker + blockText(item.tokens, indent + '  ').join('\n').trimStart();
						})
						.join('\n')
				);
				break;
			}
			case 'table': {
				const table = token as Tokens.Table;
				const row = (cells: Tokens.TableCell[]) => cells.map((cell) => inlineText(cell.tokens)).join('\t');
				blocks.push([row(table.header), ...table.rows.map(row)].join('\n'));
				break;
			}
			case 'code':
				blocks.push((token as Tokens.Code).text);
				break;
			case 'blockquote':
				blocks.push(...blockText((token as Tokens.Blockquote).tokens, indent));
				break;
			case 'text':
				blocks.push(indent + inlineText((token as Tokens.Text).tokens ?? [token]));
				break;
			default:
				break;
		}
	}
	return blocks;
}

/** The answer without Markdown syntax: links as "text (url)", tables as tab-separated rows. */
export function markdownToText(markdown: string): string {
	return blockText(marked.lexer(markdown)).join('\n\n').trim();
}

export function answerWithSources(markdown: string, sources: ResearchSource[]): string {
	if (!sources.length) return markdown;
	const list = sources.map((source, i) => `${i + 1}. ${markdownLink(source.title, source.url)}`);
	return `${markdown.trimEnd()}\n\n## ${t('research.sourcesHeading')}\n\n${list.join('\n')}\n`;
}
