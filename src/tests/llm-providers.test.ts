import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import {
	LLM_PRESETS,
	LLM_PROVIDERS,
	apiKeyKey,
	backendOf,
	endpointOf,
	modelKey,
	needsKey,
	providerName,
	urlKey,
} from '$lib/settings/llmProviders';
import { fieldByKey, fieldDescription, fieldLabel, fieldsIn, optionLabel } from '$lib/settings/schema';
import { configurationProblems } from '$lib/utils/runConfig';
import { privacyNote } from '$lib/utils/attachments';
import LlmModelPicker from '$lib/components/settings/LlmModelPicker.svelte';
import { listAnthropicModels, listOpenAiModels } from '$lib/api/tauri';

vi.mock('$lib/api/tauri', () => ({ listOpenAiModels: vi.fn(), listAnthropicModels: vi.fn() }));

const map = (entries: Record<string, string>) => new Map(Object.entries(entries));

describe('model providers', () => {
	it('lists the presets between Ollama Cloud and the custom address', () => {
		expect(LLM_PROVIDERS).toEqual([
			'openrouter',
			'anthropic',
			'ollama',
			'ollama_cloud',
			'deepseek',
			'groq',
			'mistral',
			'gemini',
			'xai',
			'openai_cloud',
			'lmstudio',
			'llamacpp',
			'openai_compatible',
		]);
		expect(fieldByKey('llm_provider')?.options).toEqual(LLM_PROVIDERS);
		expect(LLM_PRESETS.every((preset) => backendOf(preset.id) === 'openai_compatible')).toBe(true);
		expect(backendOf('ollama_cloud')).toBe('ollama_cloud');
	});

	it('keeps each preset’s key, model and address in its own settings', () => {
		expect([modelKey('deepseek'), apiKeyKey('deepseek'), urlKey('deepseek')]).toEqual(['deepseek_model', 'deepseek_api_key', null]);
		expect([modelKey('lmstudio'), apiKeyKey('lmstudio'), urlKey('lmstudio')]).toEqual(['lmstudio_model', 'lmstudio_api_key', 'lmstudio_url']);
		expect([modelKey('openai_compatible'), apiKeyKey('openai_compatible'), urlKey('openai_compatible')]).toEqual(['openai_model', 'openai_api_key', 'openai_base_url']);
		expect([modelKey('ollama'), apiKeyKey('ollama')]).toEqual(['ollama_model', null]);

		expect(endpointOf('groq', map({ groq_url: 'http://elsewhere/v1' }))).toBe('https://api.groq.com/openai/v1');
		expect(endpointOf('lmstudio', map({}))).toBe('http://localhost:1234/v1');
		expect(endpointOf('lmstudio', map({ lmstudio_url: 'http://studio.lan:1234/v1' }))).toBe('http://studio.lan:1234/v1');
		expect(endpointOf('openrouter', map({}))).toBe('https://openrouter.ai/api/v1');

		expect(providerName('openai_cloud')).toBe('OpenAI');
		expect(providerName('openai_compatible')).toBe('Server');
		expect([needsKey('gemini'), needsKey('llamacpp'), needsKey('openai_compatible')]).toEqual([true, false, false]);
	});

	it('shows the chosen preset’s fields with its own labels', () => {
		const keys = (provider: string) => fieldsIn('llm', map({ llm_provider: provider })).map((f) => f.key);
		expect(keys('deepseek')).toEqual(expect.arrayContaining(['deepseek_api_key', 'deepseek_model', 'deepseek_json_mode']));
		expect(keys('deepseek')).not.toContain('deepseek_url');
		expect(keys('deepseek')).not.toContain('openai_api_key');
		expect(keys('lmstudio')).toEqual(expect.arrayContaining(['lmstudio_url', 'lmstudio_api_key', 'lmstudio_model']));

		const key = fieldByKey('deepseek_api_key')!;
		expect(fieldLabel(key)).toBe('API key');
		expect(fieldDescription(key)).toContain('platform.deepseek.com/api_keys');
		expect(fieldDescription(fieldByKey('llamacpp_api_key')!)).toBe('Leave empty if the server does not need one.');
		expect(fieldDescription(fieldByKey('lmstudio_url')!)).toContain('LM Studio');
		expect(fieldDescription(fieldByKey('lmstudio_model')!)).toBe('A model loaded in LM Studio. Local models cost nothing.');
		expect(fieldDescription(fieldByKey('deepseek_model')!)).toContain('Pricing');
		expect(fieldByKey('lmstudio_json_mode')?.default).toBe('false');
		expect(fieldByKey('gemini_json_mode')?.default).toBe('true');

		const provider = fieldByKey('llm_provider')!;
		expect(optionLabel(provider, 'mistral')).toBe('Mistral');
		expect(optionLabel(provider, 'lmstudio')).toBe('LM Studio (local)');
		expect(optionLabel(provider, 'openai_compatible')).toBe('Custom address (OpenAI-compatible)');
	});

	it('asks cloud presets for a key and local ones only for a model', () => {
		expect(configurationProblems(map({ llm_provider: 'xai', brave_api_key: 'b' }))[0]).toMatchObject({
			message: 'xAI needs API key and model.',
			key: 'xai_api_key',
		});
		expect(configurationProblems(map({ llm_provider: 'llamacpp', brave_api_key: 'b' }))[0]).toMatchObject({
			message: 'llama.cpp needs model.',
			key: 'llamacpp_model',
		});
		expect(configurationProblems(map({ llm_provider: 'llamacpp', llamacpp_model: 'm', brave_api_key: 'b' }))).toEqual([]);
	});

	it('says where attached files go', () => {
		expect(privacyNote('deepseek', map({}))).toBe('File contents will be sent to DeepSeek.');
		expect(privacyNote('lmstudio', map({}))).toBe('Files stay on this computer.');
		expect(privacyNote('lmstudio', map({ lmstudio_url: 'http://studio.example.com:1234/v1' }))).toBe(
			'File contents will be sent to studio.example.com.'
		);
	});
});

describe('model picker for a preset', () => {
	beforeEach(() => vi.mocked(listOpenAiModels).mockReset().mockResolvedValue(['gemini-2.5-flash']));
	afterEach(cleanup);

	it('loads the preset’s catalog with its key', async () => {
		render(LlmModelPicker, {
			id: 'gemini_model',
			provider: 'openai_compatible',
			preset: 'gemini',
			value: '',
			baseUrl: 'https://generativelanguage.googleapis.com/v1beta/openai',
			apiKey: 'g-key',
			onchange: vi.fn(),
		});
		await fireEvent.focus(screen.getByRole('combobox'));
		expect(await screen.findByRole('option', { name: 'gemini-2.5-flash' })).toBeInTheDocument();
		expect(listOpenAiModels).toHaveBeenCalledWith('https://generativelanguage.googleapis.com/v1beta/openai', 'g-key', 'gemini');
		expect(screen.getByRole('listbox', { name: /Gemini/ })).toBeInTheDocument();
	});
});

describe('Anthropic', () => {
	afterEach(cleanup);

	it('has its own key, model and PDF switch', () => {
		expect([backendOf('anthropic'), modelKey('anthropic'), apiKeyKey('anthropic'), urlKey('anthropic')]).toEqual([
			'anthropic',
			'anthropic_model',
			'anthropic_api_key',
			null,
		]);
		expect(endpointOf('anthropic', map({}))).toBe('https://api.anthropic.com/v1');
		expect([providerName('anthropic'), needsKey('anthropic')]).toEqual(['Anthropic', true]);
		const keys = (provider: string) => fieldsIn('llm', map({ llm_provider: provider })).map((f) => f.key);
		expect(keys('anthropic')).toEqual(expect.arrayContaining(['anthropic_api_key', 'anthropic_model', 'anthropic_native_pdf']));
		expect(keys('deepseek')).not.toContain('anthropic_native_pdf');
		expect(fieldByKey('anthropic_native_pdf')?.default).toBe('false');
		expect(optionLabel(fieldByKey('llm_provider')!, 'anthropic')).toBe('Anthropic (Claude)');
		expect(configurationProblems(map({ llm_provider: 'anthropic', brave_api_key: 'b' }))[0]).toMatchObject({
			message: 'Anthropic needs API key and model.',
			key: 'anthropic_api_key',
		});
		expect(privacyNote('anthropic', map({}))).toBe('File contents will be sent to Anthropic.');
	});

	it('lists Claude models with the key', async () => {
		vi.mocked(listAnthropicModels).mockReset().mockResolvedValue(['claude-sonnet-5-5']);
		render(LlmModelPicker, { id: 'anthropic_model', provider: 'anthropic', value: '', apiKey: 'ant-key', onchange: vi.fn() });
		await fireEvent.focus(screen.getByRole('combobox'));
		expect(await screen.findByRole('option', { name: 'claude-sonnet-5-5' })).toBeInTheDocument();
		expect(listAnthropicModels).toHaveBeenCalledWith('ant-key');
		expect(screen.getByRole('listbox', { name: /Anthropic/ })).toBeInTheDocument();
	});
});
