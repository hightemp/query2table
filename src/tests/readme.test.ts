import { describe, expect, it } from 'vitest';
import { LOCALES } from '$lib/i18n/locales';

// The READMEs as text, and every repository file a README may link to (paths from the root).
const readmes = import.meta.glob<string>(['/README.md', '/docs/readme/*.md'], { query: '?raw', import: 'default', eager: true });
const files = new Set(Object.keys(import.meta.glob(['/README.md', '/LICENSE', '/docs/**', '/images/**', '/screenshots/**', '/src-tauri/**/*.json'])));

const pathOf = (code: string) => (code === 'en' ? '/README.md' : `/docs/readme/README.${code}.md`);

/** A link relative to `from`, as a path from the root. */
function resolve(from: string, target: string): string {
	const parts = from.split('/').slice(0, -1);
	for (const part of decodeURI(target).split('/')) {
		if (part === '..') parts.pop();
		else if (part !== '.' && part !== '') parts.push(part);
	}
	return parts.join('/');
}

describe.each(LOCALES.map((l) => [l.code, l.name] as const))('the %s README', (code, name) => {
	const path = pathOf(code);
	const text = readmes[path];

	it('exists and starts with the line of languages', () => {
		expect(text, path).toBeTypeOf('string');
		const head = text.split('\n').slice(0, 12).join('\n');
		for (const other of LOCALES) {
			if (other.code === code) expect(head, other.code).toContain(`<b>${name}</b>`);
			else expect(head, other.code).toMatch(new RegExp(`href="[^"]*README(\\.${other.code})?\\.md">${other.name.replace(/[()]/g, '\\$&')}</a>`));
		}
	});

	it('links only to files that exist', () => {
		const targets = [...text.matchAll(/(?:\]\(|src="|href=")([^)"#\s]+)/g)].map((m) => m[1]).filter((t) => !/^(https?:|mailto:)/.test(t));
		expect(targets.length).toBeGreaterThan(5);
		for (const target of targets) expect(files.has(resolve(path, target)), target).toBe(true);
	});
});
