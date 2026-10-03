import type { ImageResult } from '$lib/types';
import { webUrl } from './values';

export interface ImageView {
	image: ImageResult;
	/** Search order, used for "Found order" sorting. */
	index: number;
	title: string;
	/** Image file; null when only the thumbnail is known. */
	original: string | null;
	/** Page that shows the image. */
	page: string | null;
	domain: string;
	/** Width / height, clamped so extreme panoramas still fit a row. */
	ratio: number | null;
	pixels: number;
}

/** Decodes one punycode label (RFC 3492), e.g. "80aswg" → "рф". */
function decodePunycode(input: string): string {
	const base = 36;
	const output: number[] = [];
	const delimiter = input.lastIndexOf('-');
	for (let i = 0; i < Math.max(0, delimiter); i++) output.push(input.charCodeAt(i));
	let n = 128;
	let bias = 72;
	let i = 0;
	for (let index = delimiter + 1; index < input.length; ) {
		const oldi = i;
		for (let w = 1, k = base; ; k += base) {
			if (index >= input.length) throw new Error('Invalid punycode');
			const code = input.charCodeAt(index++);
			const digit =
				code >= 48 && code <= 57
					? code - 22
					: code >= 65 && code <= 90
						? code - 65
						: code >= 97 && code <= 122
							? code - 97
							: base;
			if (digit >= base) throw new Error('Invalid punycode');
			i += digit * w;
			const t = k <= bias ? 1 : k >= bias + 26 ? 26 : k - bias;
			if (digit < t) break;
			w *= base - t;
		}
		const length = output.length + 1;
		let delta = oldi === 0 ? Math.floor((i - oldi) / 700) : (i - oldi) >> 1;
		delta += Math.floor(delta / length);
		let k = 0;
		for (; delta > 455; k += base) delta = Math.floor(delta / 35);
		bias = Math.floor(k + (36 * delta) / (delta + 38));
		n += Math.floor(i / length);
		i %= length;
		output.splice(i++, 0, n);
	}
	return String.fromCodePoint(...output);
}

/** Readable host name: without "www." and with internationalized domains decoded. */
export function displayHost(host: string): string {
	return host
		.replace(/^www\./, '')
		.split('.')
		.map((label) => {
			if (!label.startsWith('xn--')) return label;
			try {
				return decodePunycode(label.slice(4));
			} catch {
				return label;
			}
		})
		.join('.');
}

function hostname(url: string | null): string | null {
	if (!url) return null;
	try {
		return displayHost(new URL(url).hostname);
	} catch {
		return null;
	}
}

/**
 * Display data for an image result.
 * Runs saved before the Brave Images mapping fix stored the page in `image_url`
 * and only a bare domain in `source_url`; those records have no known image file.
 */
export function describeImage(image: ImageResult, index: number): ImageView {
	const sourceUrl = webUrl(image.source_url);
	const legacy = !sourceUrl && !!image.source_url.trim();
	const page = sourceUrl ?? (legacy ? webUrl(image.image_url) : null);
	const original = legacy ? null : webUrl(image.image_url);
	const domain =
		hostname(page) ??
		(legacy ? displayHost(image.source_url.trim()) : null) ??
		hostname(original) ??
		'';
	const ratio =
		image.width && image.height
			? Math.min(3, Math.max(0.4, image.width / image.height))
			: null;
	return {
		image,
		index,
		title: image.title.trim() || domain || 'Image',
		original,
		page,
		domain,
		ratio,
		pixels: (image.width ?? 0) * (image.height ?? 0),
	};
}

export type ImageSort = 'relevance' | 'size' | 'found';

export interface ImageFilter {
	search: string;
	domain: string;
	/** Minimum length of the longer side, 0 for any size. */
	minSide: number;
	sort: ImageSort;
}

export function filterImages(views: ImageView[], filter: ImageFilter): ImageView[] {
	const needle = filter.search.trim().toLowerCase();
	const shown = views.filter(
		(view) =>
			(!needle ||
				view.title.toLowerCase().includes(needle) ||
				view.domain.toLowerCase().includes(needle)) &&
			(!filter.domain || view.domain === filter.domain) &&
			(!filter.minSide ||
				Math.max(view.image.width ?? 0, view.image.height ?? 0) >= filter.minSide)
	);
	if (filter.sort === 'size') return [...shown].sort((a, b) => b.pixels - a.pixels || a.index - b.index);
	if (filter.sort === 'found') return [...shown].sort((a, b) => a.index - b.index);
	return [...shown].sort(
		(a, b) =>
			(b.image.relevance_score ?? -1) - (a.image.relevance_score ?? -1) || a.index - b.index
	);
}

/** Domains with their image counts, most frequent first. */
export function domainCounts(views: ImageView[]): [string, number][] {
	const counts = new Map<string, number>();
	for (const view of views) if (view.domain) counts.set(view.domain, (counts.get(view.domain) ?? 0) + 1);
	return [...counts].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
}

/** Relevance is worth showing only when it tells results apart. */
export function relevanceVaries(views: ImageView[]): boolean {
	const scores = new Set(
		views.map((view) => view.image.relevance_score).filter((score) => score !== null)
	);
	return scores.size > 1;
}

/** Suggested file name without extension. */
export function imageFileName(view: ImageView): string {
	return view.title.replace(/[/\\:*?"<>|]+/g, ' ').replace(/\s+/g, ' ').trim().slice(0, 80) || 'image';
}

/** Extension taken from the image URL, if it looks like a file name. */
export function urlExtension(url: string | null): string | null {
	const match = url?.match(/\.(jpe?g|png|webp|gif|avif|svg|bmp)(?:$|[?#])/i);
	return match ? match[1].toLowerCase().replace('jpeg', 'jpg') : null;
}
