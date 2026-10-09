import { describe, expect, it } from 'vitest';
import {
	FIELDS,
	SECTIONS,
	fieldByKey,
	fieldsIn,
	isDefault,
	maskProxyUrl,
	matchingFields,
	sectionResetValues,
	validateField,
	validationErrors,
} from '$lib/settings/schema';

const map = (entries: Record<string, string>) => new Map(Object.entries(entries));

describe('settings schema', () => {
	it('has five sections, each with its own fields', () => {
		expect(SECTIONS.map((s) => s.id)).toEqual(['llm', 'search', 'runs', 'network', 'app']);
		for (const field of FIELDS) expect(SECTIONS.some((s) => s.id === field.section)).toBe(true);
		// Settings that change nothing are not offered.
		expect(fieldByKey('precision_recall')).toBeUndefined();
		expect(fieldByKey('evidence_strictness')).toBeUndefined();
		expect(fieldByKey('fetch_timeout_seconds')?.advanced).toBe(true);
	});

	it('shows only the chosen provider’s connection fields', () => {
		const keys = (provider: string) =>
			fieldsIn('llm', map({ llm_provider: provider })).map((f) => f.key);
		expect(keys('ollama')).toEqual(expect.arrayContaining(['ollama_url', 'ollama_model', 'llm_temperature']));
		expect(keys('ollama')).not.toContain('openrouter_api_key');
		expect(keys('openai_compatible')).toContain('openai_json_mode');
		expect(keys('openrouter')).not.toContain('openai_json_mode');
	});

	it('checks numbers against their ranges and URLs for a scheme', () => {
		const temperature = fieldByKey('llm_temperature')!;
		expect(validateField(temperature, '0.4')).toBeNull();
		expect(validateField(temperature, '1.5')).toBe('Enter a number from 0 to 1.');
		expect(validateField(temperature, '')).toBe('Enter a number from 0 to 1.');
		const fetches = fieldByKey('max_parallel_fetches')!;
		expect(validateField(fetches, '2.5')).toBe('Enter a whole number from 1 to 20.');
		// Serper returns up to 100 results per query; Brave stops at 20 on its own.
		const results = fieldByKey('search_results_per_query')!;
		expect(validateField(results, '100')).toBeNull();
		expect(validateField(results, '101')).toBe('Enter a whole number from 1 to 100.');
		expect(validateField(fieldByKey('brave_price_per_1000')!, '')).toBeNull();
		expect(validateField(fieldByKey('brave_price_per_1000')!, '-1')).toBe('Enter 0 or more.');
		expect(validateField(fieldByKey('ollama_url')!, 'localhost:11434')).toBe('Enter an address starting with http:// or https://.');
		expect(validateField(fieldByKey('ollama_url')!, 'http://localhost:11434')).toBeNull();
		const errors = validationErrors(map({ llm_provider: 'ollama', llm_temperature: '3', ollama_url: 'x' }));
		expect(Object.keys(errors).sort()).toEqual(['llm_temperature', 'ollama_url']);
	});

	it('knows defaults and resets a section without touching keys or connections', () => {
		expect(isDefault(fieldByKey('llm_temperature')!, '0.7')).toBe(true);
		expect(isDefault(fieldByKey('llm_temperature')!, '0.70')).toBe(true);
		expect(isDefault(fieldByKey('llm_temperature')!, '0.2')).toBe(false);
		const reset = sectionResetValues('llm', map({ llm_provider: 'ollama_cloud', llm_temperature: '0.2', llm_max_tokens: '9000' }));
		expect(reset).toEqual({
			llm_temperature: '0.7',
			llm_max_tokens: '4096',
			llm_reasoning_effort: 'auto',
			llm_context_size: 'medium',
			llm_vision: 'auto',
			max_inline_images: '8',
			vision_max_pages: '50',
		});
		expect(Object.keys(sectionResetValues('search', map({})))).not.toContain('brave_api_key');
	});

	it('finds settings by name, description or keyword', () => {
		const keys = (query: string) => matchingFields(query, map({ llm_provider: 'openrouter' })).map((f) => f.key);
		expect(keys('timeout')).toEqual(['fetch_timeout_seconds']);
		expect(keys('temperature')).toEqual(['llm_temperature']);
		expect(keys('creativity')).toEqual(['llm_temperature']);
		expect(keys('')).toEqual([]);
	});

	it('hides proxy passwords', () => {
		expect(maskProxyUrl('http://user:secret@proxy.example:8080')).toBe('http://user:••••@proxy.example:8080');
		expect(maskProxyUrl('socks5://proxy.example:1080')).toBe('socks5://proxy.example:1080');
		expect(maskProxyUrl('not a url')).toBe('not a url');
	});
});
