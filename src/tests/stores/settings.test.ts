import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { settings } from '$lib/stores/settings';

describe('settings store', () => {
	it('starts with empty map', () => {
		const map = get(settings);
		expect(map).toBeInstanceOf(Map);
	});

	it('loads settings from backend', async () => {
		await settings.load();
		const map = get(settings);
		expect(map.get('llm_provider')).toBe('openrouter');
		expect(map.get('openrouter_model')).toBe('openai/gpt-4.1-mini');
	});

	it('saves a setting and updates store', async () => {
		await settings.load();
		await settings.save('llm_provider', 'ollama');
		const map = get(settings);
		expect(map.get('llm_provider')).toBe('ollama');
	});

	it('get helper returns fallback for missing keys', () => {
		const map = get(settings);
		expect(settings.get(map, 'nonexistent', 'default')).toBe('default');
	});
});
