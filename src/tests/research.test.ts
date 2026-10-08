import { describe, it, expect } from 'vitest';
import type { ResearchStep } from '$lib/types';
import {
	describeStep,
	collectSources,
	answerHeadings,
	slugify,
	markdownToText,
	answerWithSources,
} from '$lib/utils/research';

const step = (overrides: Partial<ResearchStep>): ResearchStep => ({
	id: 's',
	step_index: 0,
	step_type: 'search',
	content: 'malaysia proxies',
	url: null,
	...overrides,
});

describe('research steps', () => {
	it('labels steps and numbers them from one', () => {
		const view = describeStep(step({}), 0);
		expect(view.number).toBe(1);
		expect(view.label).toBe('Search');
		expect(view.summary).toBe('malaysia proxies');
		expect(view.rawText).toBeNull();
		expect(describeStep(step({ step_type: 'think', content: 'Plan next' }), 1).label).toBe('Analysis');
		expect(describeStep(step({ step_type: 'error', content: 'Timeout' }), 2).label).toBe('Request issue');
	});

	it('explains an unreadable model reply and keeps the raw error behind details', () => {
		const content =
			'The model produced an invalid response: Parse error: Could not parse a research action: expected value at line 1 column 1';
		const view = describeStep(step({ step_type: 'error', content }), 3);
		expect(view.label).toBe('Unreadable reply');
		expect(view.summary).toBe('The model’s reply could not be read. The agent asked again and continued.');
		expect(view.rawText).toBe(content);
		expect(view.rawKind).toBe('details');
		expect(describeStep(step({ step_type: 'fetch', content: 'Title\n' + 'x'.repeat(400), url: 'https://a.example' }), 0).rawKind).toBe('page');
	});

	it('shows the page title or address for read pages instead of a character count', () => {
		const counted = describeStep(
			step({ step_type: 'fetch', content: 'Read page (8000 characters of content).', url: 'https://www.proxy-seller.me/malaysia/' }),
			0
		);
		expect(counted.label).toBe('Read page');
		expect(counted.summary).toBe('proxy-seller.me › malaysia');
		expect(counted.domain).toBe('proxy-seller.me');

		const titled = describeStep(
			step({ step_type: 'fetch', content: 'Malaysia proxies — Proxy-Seller', url: 'https://proxy-seller.me/my' }),
			0
		);
		expect(titled.summary).toBe('Malaysia proxies — Proxy-Seller');
		expect(titled.rawText).toBeNull();
	});

	it('hides raw page text saved by older runs behind its first line', () => {
		const raw = 'Memory management library - cppreference.com\n\ndocument.documentElement.className="client-js";RLCONF={}';
		const view = describeStep(step({ step_type: 'fetch', content: raw, url: 'https://en.cppreference.com/cpp/memory' }), 0);
		expect(view.summary).toBe('Memory management library - cppreference.com');
		expect(view.rawText).toBe(raw);
	});
});

describe('research sources', () => {
	const steps = [
		step({ id: '1', step_type: 'search', content: 'q' }),
		step({ id: '2', step_type: 'fetch', content: 'Proxy-Seller — Malaysia', url: 'https://proxy-seller.me/malaysia' }),
		step({ id: '3', step_type: 'fetch', content: 'Read page (100 characters of content).', url: 'https://froxy.com/' }),
		step({ id: '4', step_type: 'error', content: 'Failed to fetch https://broken.example', url: 'https://broken.example/' }),
	];
	const answer =
		'Use [Proxys.io](https://proxys.io/en) or [Proxy-Seller](https://proxy-seller.me/malaysia#prices).\n\n' +
		'Free lists: https://spys.one/en/ and [section](#free).';

	it('combines read pages and links cited in the answer without duplicates', () => {
		const sources = collectSources(steps, answer);
		expect(sources.map((s) => [s.url, s.read, s.cited])).toEqual([
			['https://proxys.io/en', false, true],
			['https://proxy-seller.me/malaysia', true, true],
			['https://spys.one/en/', false, true],
			['https://froxy.com/', true, false],
		]);
		expect(sources[1].title).toBe('Proxy-Seller — Malaysia');
		expect(sources[0].title).toBe('Proxys.io');
		expect(sources[2].title).toBe('spys.one');
		expect(sources[3].domain).toBe('froxy.com');
	});

	it('works before the answer exists', () => {
		expect(collectSources(steps, null).map((s) => s.url)).toEqual([
			'https://proxy-seller.me/malaysia',
			'https://froxy.com/',
		]);
	});
});

describe('answer navigation and copying', () => {
	it('lists headings with unique anchors matching the rendered answer', () => {
		expect(slugify('Что значит «управление памятью» в C++')).toBe('что-значит-управление-памятью-в-c');
		expect(answerHeadings('# Title\n\nText\n\n## Part\n\n### Part\n\n```\n# not a heading\n```\n\n#### Deep')).toEqual([
			{ depth: 1, text: 'Title', slug: 'title' },
			{ depth: 2, text: 'Part', slug: 'part' },
			{ depth: 3, text: 'Part', slug: 'part-2' },
		]);
	});

	it('copies the answer as plain text', () => {
		const text = markdownToText(
			'# Proxies\n\nUse **[Proxys.io](https://proxys.io)** now.\n\n- One\n- Two\n\n| Name | Type |\n| --- | --- |\n| Froxy | Mobile |\n\n```\ncode line\n```'
		);
		expect(text).toBe(
			'Proxies\n\nUse Proxys.io (https://proxys.io) now.\n\n- One\n- Two\n\nName\tType\nFroxy\tMobile\n\ncode line'
		);
	});

	it('appends the sources to the Markdown answer', () => {
		const sources = collectSources(
			[step({ step_type: 'fetch', content: 'Froxy', url: 'https://froxy.com/' })],
			'See [Proxys](https://proxys.io).'
		);
		expect(answerWithSources('See [Proxys](https://proxys.io).', sources)).toBe(
			'See [Proxys](https://proxys.io).\n\n## Sources\n\n1. [Proxys](https://proxys.io)\n2. [Froxy](https://froxy.com/)\n'
		);
	});
});

describe('file sources', () => {
	const read = step({ step_type: 'read', content: 'report.pdf, page 3', url: 'attachment://abc?page=3' });

	it('describes reading a file by its place', () => {
		const view = describeStep(read, 0);
		expect(view.label).toBe('Read file');
		expect(view.summary).toBe('report.pdf, p. 3');
		expect(view.domain).toBe(null);
	});

	it('lists files read and cited as sources', () => {
		const answer = 'Revenue grew [report.pdf, p. 3](attachment://abc?page=3) and [prices](attachment://xyz?sheet=A&rows=2-4).';
		const sources = collectSources([read, step({ step_type: 'read', content: 'Read 2 scanned pages', url: null })], answer);
		expect(sources).toEqual([
			{ url: 'attachment://abc?page=3', title: 'report.pdf, p. 3', domain: 'report.pdf', read: true, cited: true, file: true },
			{ url: 'attachment://xyz?sheet=A&rows=2-4', title: 'prices', domain: 'prices', read: false, cited: true, file: true },
		]);
	});
});

describe('copying answers with file citations', () => {
	it('turns file links into readable places', async () => {
		const { readableAnswer } = await import('$lib/utils/research');
		const sources = [
			{ url: 'attachment://abc?page=3', title: 'report.pdf, p. 3', domain: 'report.pdf', read: true, cited: true, file: true },
			{ url: 'https://a.example', title: 'Site', domain: 'a.example', read: false, cited: true },
		];
		expect(readableAnswer('Grew [report.pdf, p. 3](attachment://abc?page=3) and [more](attachment://abc?page=5).', sources)).toBe(
			'Grew report.pdf, p. 3 and more (report.pdf, p. 5).'
		);
		expect(answerWithSources('Grew [here](attachment://abc?page=3).', sources)).toBe(
			'Grew here (report.pdf, p. 3).\n\n## Sources\n\n1. report.pdf, p. 3\n2. [Site](https://a.example)\n'
		);
	});
});
