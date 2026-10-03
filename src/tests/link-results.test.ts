import { describe, it, expect } from 'vitest';
import type { LinkResult } from '$lib/types';
import {
	describeLinks,
	filterLinks,
	formatLinks,
	groupBySite,
	siteCounts,
	tierOf,
	faviconUrl,
} from '$lib/utils/linkResults';

const link = (overrides: Partial<LinkResult>): LinkResult => ({
	id: 'l',
	url: 'https://github.com/arch3rPro/PentestTools',
	title: 'PentestTools',
	description: 'A collection of tools',
	reason: '',
	relevance_score: 0.9,
	low_relevance: false,
	hidden: false,
	visited_at: null,
	created_at: 1,
	...overrides,
});

describe('link results', () => {
	it('shows the site, a readable path and a match tier', () => {
		const [view] = describeLinks([
			link({ url: 'https://www.xn--80aswg.xn--p1ai/%D1%81%D1%82%D0%B0%D1%82%D1%8C%D1%8F/?q=1' }),
		]);
		expect(view.domain).toBe('сайт.рф');
		expect(view.host).toBe('www.xn--80aswg.xn--p1ai');
		expect(view.path).toBe('статья?q=1');
		expect(tierOf(0.95)).toBe('best');
		expect(tierOf(0.7)).toBe('good');
		expect(tierOf(0.5)).toBe('partial');
		expect(tierOf(null)).toBeNull();
		expect(faviconUrl('www.xn--80aswg.xn--p1ai')).toBe('https://icons.duckduckgo.com/ip3/xn--80aswg.xn--p1ai.ico');
	});

	it('separates weak and hidden links and keeps found order', () => {
		const views = describeLinks([
			link({ id: 'a', url: 'https://a.example/', relevance_score: 0.8, created_at: 3 }),
			link({ id: 'b', url: 'https://b.example/', relevance_score: 0.95, created_at: 1 }),
			link({ id: 'weak', relevance_score: 0.2, low_relevance: true, created_at: 2 }),
			link({ id: 'gone', hidden: true, created_at: 4 }),
		]);
		const ids = (result: { main: { link: LinkResult }[] }) => result.main.map((v) => v.link.id);
		const base = { search: '', domain: '', sort: 'relevance' as const, showHidden: false };
		const result = filterLinks(views, base);
		expect(ids(result)).toEqual(['b', 'a']);
		expect(result.low.map((v) => v.link.id)).toEqual(['weak']);
		expect(result.hiddenCount).toBe(1);
		expect(ids(filterLinks(views, { ...base, sort: 'found' }))).toEqual(['b', 'a']);
		expect(ids(filterLinks(views, { ...base, showHidden: true }))).toContain('gone');
		expect(ids(filterLinks(views, { ...base, search: 'a.example' }))).toEqual(['a']);
		expect(siteCounts(views)[0]).toEqual(['github.com', 2]);
	});

	it('groups by site with the best group first', () => {
		const views = describeLinks([
			link({ id: '1', url: 'https://x.example/1', relevance_score: 0.7 }),
			link({ id: '2', url: 'https://y.example/1', relevance_score: 0.95 }),
			link({ id: '3', url: 'https://x.example/2', relevance_score: 0.8 }),
		]);
		expect(groupBySite(views).map((g) => [g.domain, g.views.length])).toEqual([
			['y.example', 1],
			['x.example', 2],
		]);
	});

	it('copies links in several formats', () => {
		const views = describeLinks([link({ title: 'Tools [new]' }), link({ id: '2', title: '', url: 'https://b.example/' })]);
		expect(formatLinks(views, 'urls')).toBe('https://github.com/arch3rPro/PentestTools\nhttps://b.example/');
		expect(formatLinks(views, 'markdown')).toBe(
			'- [Tools \\[new\\]](https://github.com/arch3rPro/PentestTools)\n- [https://b.example/](https://b.example/)'
		);
		expect(formatLinks(views, 'titled').split('\n')[0]).toBe('Tools [new] — https://github.com/arch3rPro/PentestTools');
	});
});
