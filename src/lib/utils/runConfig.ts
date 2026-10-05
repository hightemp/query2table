import { t, type MessageKey } from '$lib/i18n';
export interface ConfigProblem {
	message: string;
	/** Settings section id, used as `/settings#settings-<section>`. */
	section: 'llm' | 'search';
	/** The first setting to fill in. */
	key: string;
}

const providerNames: Record<string, string> = {
	openrouter: 'OpenRouter',
	ollama: 'Ollama',
	ollama_cloud: 'Ollama Cloud',
	openai_compatible: 'OpenAI-compatible',
};

/** Settings each LLM provider needs before a run can start. Mirrors the backend checks. */
const llmRequirements: Record<string, { key: string; label: MessageKey }[]> = {
	openrouter: [
		{ key: 'openrouter_api_key', label: 'config.apiKey' },
		{ key: 'openrouter_model', label: 'config.model' },
	],
	ollama: [
		{ key: 'ollama_url', label: 'config.serverUrl' },
		{ key: 'ollama_model', label: 'config.model' },
	],
	ollama_cloud: [
		{ key: 'ollama_cloud_api_key', label: 'config.apiKey' },
		{ key: 'ollama_cloud_model', label: 'config.model' },
	],
	openai_compatible: [
		{ key: 'openai_base_url', label: 'config.baseUrl' },
		{ key: 'openai_model', label: 'config.model' },
	],
};

const searchRequirements: Record<string, { key: string; name: string }> = {
	brave: { key: 'brave_api_key', name: 'Brave Search' },
	serper: { key: 'serper_api_key', name: 'Serper' },
};

/**
 * Lists settings that would make a new run fail at startup.
 * Returns nothing while settings are not loaded; the backend then reports the problem.
 */
export function configurationProblems(settings: Map<string, string>): ConfigProblem[] {
	if (!settings.size) return [];
	const problems: ConfigProblem[] = [];
	const filled = (key: string) => !!settings.get(key)?.trim();

	const provider = settings.get('llm_provider') || 'openrouter';
	const missing = (llmRequirements[provider] ?? []).filter((item) => !filled(item.key));
	if (missing.length)
		problems.push({
			message: t('config.needs', {
				provider: providerNames[provider] ?? provider,
				items: missing.map((item) => t(item.label)).join(t('config.and')),
			}),
			section: 'llm',
			key: missing[0].key,
		});

	const search = searchRequirements[settings.get('search_provider') || 'brave'];
	if (search && !filled(search.key))
		problems.push({ message: t('config.searchKey', { name: search.name }), section: 'search', key: search.key });

	return problems;
}
