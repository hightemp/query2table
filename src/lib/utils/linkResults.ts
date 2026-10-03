import type { LinkResult } from '$lib/types';
import { displayHost } from './hosts';
import { markdownLink } from './linkMenu';

export type MatchTier = 'best' | 'good' | 'partial';

export interface LinkView {
	link: LinkResult;
	/** Position in found order. */
	order: number;
	domain: string;
	/** ASCII host name, for services that expect it. */
	host: string;
	/** Readable path and query without the domain, '' for a site's home page. */
	path: string;
	tier: MatchTier | null;
}

export const TIER_LABELS: Record<MatchTier, string> = {
	best: 'Best match',
	good: 'Good match',
	partial: 'Partial match',
};

export function tierOf(score: number | null): MatchTier | null {
	if (score === null) return null;
	return score >= 0.9 ? 'best' : score >= 0.7 ? 'good' : 'partial';
}

function shorten(text: string, max: number) {
	if (text.length <= max) return text;
	const half = Math.floor((max - 1) / 2);
	return `${text.slice(0, half)}…${text.slice(text.length - half)}`;
}

function readablePath(url: URL) {
	const raw = `${url.pathname.replace(/\/+$/, '')}${url.search}`;
	let path = raw;
	try {
		path = decodeURI(raw);
	} catch {
		// Keep the encoded form when it is not valid UTF-8.
	}
	return shorten(path.replace(/^\//, ''), 70);
}

export function describeLink(link: LinkResult, order: number): LinkView {
	let domain = link.url;
	let host = '';
	let path = '';
	try {
		const url = new URL(link.url);
		host = url.hostname;
		domain = displayHost(url.hostname);
		path = readablePath(url);
	} catch {
		// Not a URL: show it as the domain.
	}
	return { link, order, domain, host, path, tier: tierOf(link.relevance_score) };
}

/** Found order; runs loaded from history are ordered by `created_at`, live runs by arrival. */
export function describeLinks(links: LinkResult[]): LinkView[] {
	const arrival = links.map((link, index) => ({ link, index }));
	arrival.sort(
		(a, b) => (a.link.created_at ?? 0) - (b.link.created_at ?? 0) || a.index - b.index
	);
	const order = new Map(arrival.map((item, position) => [item.link.id, position]));
	return links.map((link) => describeLink(link, order.get(link.id) ?? 0));
}

export type LinkSort = 'relevance' | 'site' | 'found';

export interface LinkFilter {
	search: string;
	domain: string;
	sort: LinkSort;
	showHidden: boolean;
}

export interface FilteredLinks {
	/** Links above the relevance threshold, in display order. */
	main: LinkView[];
	/** Links scored below the threshold, shown collapsed. */
	low: LinkView[];
	hiddenCount: number;
}

function compare(sort: LinkSort) {
	return (a: LinkView, b: LinkView) => {
		if (sort === 'found') return a.order - b.order;
		if (sort === 'site')
			return a.domain.localeCompare(b.domain) || (b.link.relevance_score ?? -1) - (a.link.relevance_score ?? -1);
		return (b.link.relevance_score ?? -1) - (a.link.relevance_score ?? -1) || a.order - b.order;
	};
}

export function filterLinks(views: LinkView[], filter: LinkFilter): FilteredLinks {
	const needle = filter.search.trim().toLowerCase();
	const hiddenCount = views.filter((view) => view.link.hidden).length;
	const shown = views
		.filter(
			(view) =>
				(filter.showHidden || !view.link.hidden) &&
				(!filter.domain || view.domain === filter.domain) &&
				(!needle ||
					[view.link.title, view.link.description, view.link.url, view.domain].some((text) =>
						text.toLowerCase().includes(needle)
					))
		)
		.sort(compare(filter.sort));
	return {
		main: shown.filter((view) => !view.link.low_relevance),
		low: shown.filter((view) => view.link.low_relevance),
		hiddenCount,
	};
}

export interface SiteGroup {
	domain: string;
	views: LinkView[];
}

/** Groups keep the given order inside; groups with the best link come first. */
export function groupBySite(views: LinkView[]): SiteGroup[] {
	const groups = new Map<string, LinkView[]>();
	for (const view of views) groups.set(view.domain, [...(groups.get(view.domain) ?? []), view]);
	const best = (group: LinkView[]) => Math.max(...group.map((view) => view.link.relevance_score ?? -1));
	return [...groups]
		.map(([domain, items]) => ({ domain, views: items }))
		.sort((a, b) => best(b.views) - best(a.views) || b.views.length - a.views.length || a.domain.localeCompare(b.domain));
}

export function siteCounts(views: LinkView[]): [string, number][] {
	const counts = new Map<string, number>();
	for (const view of views) counts.set(view.domain, (counts.get(view.domain) ?? 0) + 1);
	return [...counts].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
}

export type CopyFormat = 'urls' | 'markdown' | 'titled';

export const COPY_FORMATS: Record<CopyFormat, string> = {
	urls: 'URLs, one per line',
	markdown: 'Markdown list',
	titled: 'Title — URL',
};

export function formatLinks(views: LinkView[], format: CopyFormat): string {
	return views
		.map(({ link }) =>
			format === 'urls'
				? link.url
				: format === 'markdown'
					? `- ${markdownLink(link.title, link.url)}`
					: `${link.title.trim() || link.url} — ${link.url}`
		)
		.join('\n');
}

/** Stable hue for a site's letter badge. */
export function siteHue(domain: string): number {
	let hash = 0;
	for (const char of domain) hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
	return hash % 360;
}

export function faviconUrl(host: string): string {
	return `https://icons.duckduckgo.com/ip3/${encodeURIComponent(host.replace(/^www\./, ''))}.ico`;
}
