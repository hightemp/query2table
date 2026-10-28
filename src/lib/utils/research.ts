import { t, type MessageKey } from '$lib/i18n';
import type { Token, Tokens } from 'marked';
import { markdown as marked, type MathToken } from './math';
import type { ResearchImage, ResearchStep } from '$lib/types';
import { displayHost } from './hosts';
import { markdownLink } from './linkMenu';
import { attachmentPlace, placeLabel } from './attachments';

export type StepKind = 'search' | 'image_search' | 'view_image' | 'video_search' | 'video' | 'tools' | 'fetch' | 'read' | 'think' | 'error';

const STEP_KINDS = ['search', 'image_search', 'view_image', 'video_search', 'video', 'tools', 'fetch', 'read', 'think', 'error'];

/** A place in an attached file: `attachment://<id>?page=3`. */
export function isAttachmentUrl(url: string | null | undefined): boolean {
	return !!url && url.startsWith('attachment://');
}

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
	const url = isAttachmentUrl(step.url) ? null : parseUrl(step.url);
	const domain = url ? displayHost(url.hostname) : null;
	const content = step.content.trim();
	let summary = content;
	let rawText: string | null = null;
	let label = STEP_KINDS.includes(step.step_type) ? t(`research.step.${step.step_type}` as MessageKey) : step.step_type;
	if (step.step_type === 'error' && INVALID_REPLY.test(content)) {
		label = t('research.step.unreadable');
		summary = t('research.step.unreadableSummary');
		rawText = step.content;
	} else if (step.step_type === 'read') {
		const found = attachmentPlace(step.url);
		if (found) summary = placeLabel(fileName(content), found.place);
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
	/** A place in an attached file rather than a web page. */
	file?: boolean;
	/** Pictures of the answer that come from this page. */
	images?: string[];
	/** A video: its kept thumbnail (the address the answer uses), channel and length. */
	video?: { thumbnail: string | null; channel: string | null; duration: number | null };
}

/** Comparable form of a URL: no fragment, lower-case host, no trailing slash. */
function sourceKey(url: URL): string {
	return `${url.protocol}//${url.host.toLowerCase()}${url.pathname.replace(/\/+$/, '')}${url.search}`;
}

/** A video's key: its source key without the time to start at. */
function videoKey(url: URL): string {
	const copy = new URL(url.href);
	for (const name of ['t', 'start', 'time_continue']) copy.searchParams.delete(name);
	copy.hash = '';
	return sourceKey(copy);
}

/** `3:33`, or `1:02:03` from an hour; empty when unknown. */
export function formatDuration(seconds: number | null | undefined): string {
	if (seconds === null || seconds === undefined || !Number.isFinite(seconds)) return '';
	const total = Math.max(0, Math.round(seconds));
	const [h, m, s] = [Math.floor(total / 3600), Math.floor(total / 60) % 60, total % 60];
	const pad = (n: number) => String(n).padStart(2, '0');
	return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`;
}

export interface VideoCard {
	title: string;
	thumbnail: string;
	video: string;
}

/** Video cards in Markdown, in order: `[![title](thumbnail)](video)`, each thumbnail once. */
export function answerVideoCards(markdown: string): VideoCard[] {
	const found: VideoCard[] = [];
	const card = /\[\s*!\[([^\]]*)\]\(\s*<?([^\s)>]+)>?(?:\s+"[^"]*")?\s*\)\s*\]\(\s*<?([^\s)>]+)>?(?:\s+"[^"]*")?\s*\)/g;
	for (const match of markdown.matchAll(card))
		if (!found.some((c) => c.thumbnail === match[2])) found.push({ title: match[1].trim(), thumbnail: match[2], video: match[3] });
	return found;
}

/** The address as written, without its fragment. */
function withoutHash(raw: string): string {
	return raw.replace(/#.*$/, '');
}

/** Links in Markdown text, in order: `[title](url)` and bare http(s) addresses. */
export function answerLinks(markdown: string): { url: URL; raw: string; title: string }[] {
	const found: { url: URL; raw: string; title: string; at: number }[] = [];
	// Pictures (`![alt](url)`) are not links; their pages come from the kept images.
	const linked = /(?<!!)\[([^\]]*)\]\(((?:https?|attachment):\/\/[^)\s]+)\)/g;
	for (const match of markdown.matchAll(linked)) {
		// The outer brackets of a card: `[![title](thumbnail)](video)`.
		if (match[1].trimStart().startsWith('![')) continue;
		const url = parseUrl(match[2]);
		if (url) found.push({ url, raw: match[2], title: match[1].trim(), at: match.index ?? 0 });
	}
	const bare = /(?<![(\[\w<])https?:\/\/[^\s)<>\]"'`]+/g;
	for (const match of markdown.matchAll(bare)) {
		const raw = match[0].replace(/[.,;:!?]+$/, '');
		const url = parseUrl(raw);
		if (url) found.push({ url, raw, title: '', at: match.index ?? 0 });
	}
	return found.sort((a, b) => a.at - b.at);
}

/** "report.pdf" of "report.pdf, page 3". */
function fileName(label: string): string {
	return label.split(', ')[0].trim();
}

/** Picture addresses in Markdown, in order: `![alt](url)`. */
export function answerImages(markdown: string): string[] {
	const found: string[] = [];
	for (const match of markdown.matchAll(/!\[[^\]]*\]\(\s*<?([^\s)>]+)>?(?:\s+"[^"]*")?\s*\)/g))
		if (!found.includes(match[1])) found.push(match[1]);
	return found;
}

export function collectSources(steps: ResearchStep[], answer: string | null, images: ResearchImage[] = []): ResearchSource[] {
	const sources = new Map<string, ResearchSource>();
	// Videos come after the pages: shown as cards, cited at a moment, or watched.
	const videos = new Map<string, ResearchSource>();
	for (const card of answer ? answerVideoCards(answer) : []) {
		const url = parseUrl(card.video);
		if (!url || videos.has(videoKey(url))) continue;
		const kept = images.find((image) => image.url === card.thumbnail && image.video_url);
		videos.set(videoKey(url), {
			url: withoutHash(card.video),
			title: card.title,
			domain: displayHost(url.hostname),
			read: false,
			cited: true,
			video: { thumbnail: kept ? kept.url : null, channel: kept?.channel ?? null, duration: kept?.duration ?? null },
		});
	}
	for (const step of steps) {
		if (step.step_type !== 'video') continue;
		const url = parseUrl(step.url);
		if (!url) continue;
		const title = step.content.replace(/ · from [\d:]+$/, '').trim();
		const existing = videos.get(videoKey(url));
		if (existing) {
			existing.read = true;
			if (title) existing.title = title;
		} else
			videos.set(videoKey(url), {
				url: withoutHash(step.url!),
				title,
				domain: displayHost(url.hostname),
				read: true,
				cited: false,
				video: { thumbnail: null, channel: null, duration: null },
			});
	}
	for (const { url, raw, title } of answer ? answerLinks(answer) : []) {
		const video = videos.get(videoKey(url));
		if (video) {
			video.cited = true;
			continue;
		}
		const key = sourceKey(url);
		const existing = sources.get(key);
		if (existing) {
			if (!existing.title && title) existing.title = title;
			continue;
		}
		const file = isAttachmentUrl(raw);
		sources.set(key, {
			url: withoutHash(raw),
			title,
			domain: file ? fileName(title) : displayHost(url.hostname),
			read: false,
			cited: true,
			...(file ? { file } : {}),
		});
	}
	// Pages the answer's pictures come from count as cited.
	for (const address of answer ? answerImages(answer) : []) {
		const image = images.find((item) => item.url === address);
		const page = parseUrl(image?.source_url);
		if (!image?.source_url || !page || image.video_url) continue;
		const key = sourceKey(page);
		const existing = sources.get(key);
		if (existing) existing.images = [...(existing.images ?? []), address];
		else
			sources.set(key, {
				url: withoutHash(image.source_url),
				title: '',
				domain: displayHost(page.hostname),
				read: false,
				cited: true,
				images: [address],
			});
	}
	for (const step of steps) {
		if (step.step_type === 'read' && isAttachmentUrl(step.url)) {
			const url = parseUrl(step.url)!;
			const key = sourceKey(url);
			const name = fileName(step.content);
			const label = placeLabel(name, attachmentPlace(step.url)?.place ?? {});
			const existing = sources.get(key);
			if (existing) {
				existing.read = true;
				existing.title = label || existing.title;
				existing.domain = name || existing.domain;
			} else sources.set(key, { url: step.url!, title: label, domain: name, read: true, cited: false, file: true });
			continue;
		}
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
	return [...sources.values(), ...videos.values()].map((source) => ({
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
					// A video card: its title and the video.
					const only = link.tokens?.filter((t) => !(t.type === 'text' && !String(t.text).trim()));
					if (only?.length === 1 && only[0].type === 'image') {
						const title = (only[0] as Tokens.Image).text.trim();
						return title ? `${title} (${link.href})` : link.href;
					}
					const label = inlineText(link.tokens);
					return label && label !== link.href ? `${label} (${link.href})` : link.href;
				}
				case 'br':
					return '\n';
				case 'image': {
					const image = token as Tokens.Image;
					return image.text ? `${image.text} (${image.href})` : image.href;
				}
				case 'math':
					return (token as MathToken).raw;
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
			case 'math':
				blocks.push(indent + (token as MathToken).raw.trim());
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

/** Links to attached files become text naming the place: they mean nothing outside the app. */
export function readableAnswer(markdown: string, sources: ResearchSource[]): string {
	return markdown.replace(/\[([^\]]*)\]\((attachment:\/\/[^)\s]+)\)/g, (_, text: string, url: string) => {
		const found = attachmentPlace(url);
		const known = sources.find((s) => s.file && attachmentPlace(s.url)?.id === found?.id);
		const name = known?.domain || t('attachments.preview');
		const place = placeLabel(name, found?.place ?? {});
		return text.includes(name) ? text.trim() : `${text.trim()} (${place})`;
	});
}

export function answerWithSources(markdown: string, sources: ResearchSource[]): string {
	const text = readableAnswer(markdown, sources);
	if (!sources.length) return text;
	const list = sources.map((source, i) => `${i + 1}. ${source.file ? source.title : markdownLink(source.title, source.url)}`);
	return `${text.trimEnd()}\n\n## ${t('research.sourcesHeading')}\n\n${list.join('\n')}\n`;
}
