import { describe, it, expect } from 'vitest';
import type { ImageResult } from '$lib/types';
import {
	describeImage,
	filterImages,
	domainCounts,
	relevanceVaries,
	imageFileName,
	urlExtension,
} from '$lib/utils/images';
import { displayHost } from '$lib/utils/hosts';

const image = (overrides: Partial<ImageResult>): ImageResult => ({
	id: 'i',
	image_url: 'https://cdn.example/a.jpg',
	thumbnail_url: 'https://thumbs.example/a',
	title: 'Tower',
	source_url: 'https://www.news.example/article',
	width: 1600,
	height: 900,
	relevance_score: null,
	...overrides,
});

describe('image results', () => {
	it('links the image file and the page that shows it', () => {
		const view = describeImage(image({}), 0);
		expect(view.original).toBe('https://cdn.example/a.jpg');
		expect(view.page).toBe('https://www.news.example/article');
		expect(view.domain).toBe('news.example');
		expect(view.ratio).toBeCloseTo(1.78, 2);
	});

	it('treats old Brave records as page links without a known image file', () => {
		const view = describeImage(
			image({ image_url: 'https://www.forbes.ru/gallery/1', source_url: 'forbes.ru', title: '' }),
			0
		);
		expect(view.original).toBeNull();
		expect(view.page).toBe('https://www.forbes.ru/gallery/1');
		expect(view.domain).toBe('forbes.ru');
		expect(view.title).toBe('forbes.ru');
	});

	it('decodes internationalized domains', () => {
		expect(displayHost('xn----7sbf0aahnq1aem.xn--80aswg')).toBe('москва-сити.сайт');
		expect(displayHost('www.xn--80aswg')).toBe('сайт');
		expect(displayHost('xn--broken!')).toBe('xn--broken!');
	});

	it('filters by text, site and size, and sorts', () => {
		const views = [
			image({ id: 'a', title: 'Small', width: 300, height: 200, relevance_score: 0.5 }),
			image({ id: 'b', title: 'Big', width: 3000, height: 2000, relevance_score: 0.9 }),
			image({ id: 'c', title: 'Other', source_url: 'https://other.example/', width: 1200, height: 800, relevance_score: 0.9 }),
		].map(describeImage);
		const ids = (filter: Partial<Parameters<typeof filterImages>[1]>) =>
			filterImages(views, { search: '', domain: '', minSide: 0, sort: 'relevance', ...filter }).map((v) => v.image.id);
		expect(ids({})).toEqual(['b', 'c', 'a']);
		expect(ids({ sort: 'size' })).toEqual(['b', 'c', 'a']);
		expect(ids({ sort: 'found' })).toEqual(['a', 'b', 'c']);
		expect(ids({ minSide: 1000 })).toEqual(['b', 'c']);
		expect(ids({ domain: 'other.example' })).toEqual(['c']);
		expect(ids({ search: 'news' })).toEqual(['b', 'a']);
		expect(domainCounts(views)).toEqual([['news.example', 2], ['other.example', 1]]);
		expect(relevanceVaries(views)).toBe(true);
		expect(relevanceVaries(views.map((v) => ({ ...v, image: { ...v.image, relevance_score: 0.5 } })))).toBe(false);
	});

	it('suggests safe file names and extensions', () => {
		expect(imageFileName(describeImage(image({ title: 'A/B: "tower"?' }), 0))).toBe('A B tower');
		expect(urlExtension('https://x.example/a.JPEG?w=2')).toBe('jpg');
		expect(urlExtension('https://x.example/page')).toBeNull();
	});
});
