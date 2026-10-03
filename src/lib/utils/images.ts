import type { ImageResult } from '$lib/types';
import { webUrl } from './values';
import { displayHost, hostname } from './hosts';

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
