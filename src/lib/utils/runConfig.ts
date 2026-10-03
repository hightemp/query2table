export interface ConfigProblem {
	message: string;
	/** Settings section id, used as `/settings#settings-<section>`. */
	section: 'llm' | 'search';
}

const providerNames: Record<string, string> = {
	openrouter: 'OpenRouter',
	ollama: 'Ollama',
	ollama_cloud: 'Ollama Cloud',
	openai_compatible: 'OpenAI-compatible',
};

/** Settings each LLM provider needs before a run can start. Mirrors the backend checks. */
const llmRequirements: Record<string, { key: string; label: string }[]> = {
	openrouter: [
		{ key: 'openrouter_api_key', label: 'API key' },
		{ key: 'openrouter_model', label: 'model' },
	],
	ollama: [
		{ key: 'ollama_url', label: 'server URL' },
		{ key: 'ollama_model', label: 'model' },
	],
	ollama_cloud: [
		{ key: 'ollama_cloud_api_key', label: 'API key' },
		{ key: 'ollama_cloud_model', label: 'model' },
	],
	openai_compatible: [
		{ key: 'openai_base_url', label: 'API base URL' },
		{ key: 'openai_model', label: 'model' },
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
	const missing = (llmRequirements[provider] ?? [])
		.filter((item) => !filled(item.key))
		.map((item) => item.label);
	if (missing.length)
		problems.push({
			message: `${providerNames[provider] ?? provider} needs ${missing.join(' and ')}.`,
			section: 'llm',
		});

	const search = searchRequirements[settings.get('search_provider') || 'brave'];
	if (search && !filled(search.key))
		problems.push({ message: `${search.name} needs an API key.`, section: 'search' });

	return problems;
}
