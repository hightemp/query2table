import { t } from '$lib/i18n';

/**
 * Model providers. OpenRouter, Anthropic, Ollama and Ollama Cloud have clients of their own; the presets are
 * services speaking the OpenAI Chat Completions API, run by the one OpenAI-compatible client with
 * their address. Mirrors `src-tauri/src/providers/llm/presets.rs` (checked by a Rust test).
 */
export interface LlmPreset {
	id: string;
	name: string;
	/** Chat Completions base address. */
	url: string;
	/** Runs on the user's machine: free, address editable, key optional. */
	local: boolean;
	/** Strict JSON replies by default. */
	jsonMode: boolean;
	/** Where to create a key (cloud services). */
	keyPage?: string;
}

export const LLM_PRESETS: LlmPreset[] = [
	{ id: 'deepseek', name: 'DeepSeek', url: 'https://api.deepseek.com/v1', local: false, jsonMode: true, keyPage: 'platform.deepseek.com/api_keys' },
	{ id: 'groq', name: 'Groq', url: 'https://api.groq.com/openai/v1', local: false, jsonMode: true, keyPage: 'console.groq.com/keys' },
	{ id: 'mistral', name: 'Mistral', url: 'https://api.mistral.ai/v1', local: false, jsonMode: true, keyPage: 'console.mistral.ai/api-keys' },
	{ id: 'gemini', name: 'Gemini', url: 'https://generativelanguage.googleapis.com/v1beta/openai', local: false, jsonMode: true, keyPage: 'aistudio.google.com/apikey' },
	{ id: 'xai', name: 'xAI', url: 'https://api.x.ai/v1', local: false, jsonMode: true, keyPage: 'console.x.ai' },
	{ id: 'openai_cloud', name: 'OpenAI', url: 'https://api.openai.com/v1', local: false, jsonMode: true, keyPage: 'platform.openai.com/api-keys' },
	{ id: 'lmstudio', name: 'LM Studio', url: 'http://localhost:1234/v1', local: true, jsonMode: false },
	{ id: 'llamacpp', name: 'llama.cpp', url: 'http://localhost:8080/v1', local: true, jsonMode: true },
];

/** The client a provider's model list and requests go through. */
export type LlmBackend = 'openrouter' | 'anthropic' | 'ollama' | 'ollama_cloud' | 'openai_compatible';

/** `llm_provider` values in the order of the list; "Custom address" comes last. */
export const LLM_PROVIDERS: string[] = ['openrouter', 'anthropic', 'ollama', 'ollama_cloud', ...LLM_PRESETS.map((p) => p.id), 'openai_compatible'];

const presets = new Map(LLM_PRESETS.map((preset) => [preset.id, preset]));

export function presetOf(provider: string): LlmPreset | undefined {
	return presets.get(provider);
}

export function backendOf(provider: string): LlmBackend {
	if (provider === 'anthropic' || provider === 'ollama' || provider === 'ollama_cloud' || provider === 'openai_compatible') return provider;
	return presets.has(provider) ? 'openai_compatible' : 'openrouter';
}

/** Name for messages, costs and summaries. */
export function providerName(provider: string): string {
	const names: Record<string, string> = { openrouter: 'OpenRouter', anthropic: 'Anthropic', ollama: 'Ollama', ollama_cloud: 'Ollama Cloud', openai_compatible: t('models.server') };
	return presets.get(provider)?.name ?? names[provider] ?? provider;
}

/** Setting with the provider's model. */
export function modelKey(provider: string): string {
	if (presets.has(provider)) return `${provider}_model`;
	return ({ anthropic: 'anthropic_model', ollama: 'ollama_model', ollama_cloud: 'ollama_cloud_model', openai_compatible: 'openai_model' } as Record<string, string>)[provider] ?? 'openrouter_model';
}

/** Setting with the provider's API key; Ollama has none. */
export function apiKeyKey(provider: string): string | null {
	if (presets.has(provider)) return `${provider}_api_key`;
	return ({ openrouter: 'openrouter_api_key', anthropic: 'anthropic_api_key', ollama_cloud: 'ollama_cloud_api_key', openai_compatible: 'openai_api_key' } as Record<string, string>)[provider] ?? null;
}

/** Setting with the provider's address, when it can be changed. */
export function urlKey(provider: string): string | null {
	const preset = presets.get(provider);
	if (preset) return preset.local ? `${provider}_url` : null;
	return ({ ollama: 'ollama_url', ollama_cloud: 'ollama_cloud_url', openai_compatible: 'openai_base_url' } as Record<string, string>)[provider] ?? null;
}

const DEFAULT_URLS: Record<string, string> = {
	openrouter: 'https://openrouter.ai/api/v1',
	anthropic: 'https://api.anthropic.com/v1',
	ollama: 'http://localhost:11434',
	ollama_cloud: 'https://ollama.com',
	openai_compatible: 'http://localhost:8080/v1',
};

/** The address requests go to with these settings (also the scope of manual rates). */
export function endpointOf(provider: string, settings: Map<string, string>): string {
	const key = urlKey(provider);
	const fallback = presets.get(provider)?.url ?? DEFAULT_URLS[provider] ?? DEFAULT_URLS.openrouter;
	return (key && settings.get(key)?.trim()) || fallback;
}

/** Whether runs need a key: cloud services do, local servers and Ollama do not. */
export function needsKey(provider: string): boolean {
	const preset = presets.get(provider);
	if (preset) return !preset.local;
	return provider === 'openrouter' || provider === 'anthropic' || provider === 'ollama_cloud';
}
