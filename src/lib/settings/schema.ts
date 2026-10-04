/** What each setting is, where it lives on the Settings page and which values it accepts. */

export type SectionId = 'llm' | 'search' | 'runs' | 'network' | 'app';
export type LlmProvider = 'openrouter' | 'ollama' | 'ollama_cloud' | 'openai_compatible';
export type FieldKind = 'select' | 'text' | 'url' | 'password' | 'number' | 'range' | 'switch' | 'model';

export interface SectionDef {
	id: SectionId;
	label: string;
	description: string;
	/** Sub-headings in order. */
	groups: string[];
}

export interface FieldDef {
	key: string;
	section: SectionId;
	group: string;
	label: string;
	description: string;
	kind: FieldKind;
	/** Value the backend uses when nothing was chosen. */
	default: string;
	options?: { label: string; value: string }[];
	placeholder?: string;
	min?: number;
	max?: number;
	step?: number;
	/** Shown after the value, e.g. "s". */
	unit?: string;
	/** Rarely changed: inside the section's Advanced block. */
	advanced?: boolean;
	/** Shown only for this LLM provider. */
	provider?: LlmProvider;
	/** Connection details and keys: kept by "Reset section". */
	keepOnReset?: boolean;
	/** Extra words the settings search should match. */
	keywords?: string;
}

export const SECTIONS: SectionDef[] = [
	{ id: 'llm', label: 'LLM', description: 'The model that reads pages and writes results.', groups: ['Connection', 'Generation', 'Pricing'] },
	{ id: 'search', label: 'Search', description: 'Web search used to find pages.', groups: ['Provider', 'Pricing'] },
	{ id: 'runs', label: 'Runs', description: 'How pages are loaded and processed during a run.', groups: ['Fetching', 'Quality', 'Content'] },
	{ id: 'network', label: 'Network', description: 'Proxies for searches and page loads.', groups: ['Proxies'] },
	{ id: 'app', label: 'Application', description: 'Appearance, notifications and local files.', groups: ['Appearance', 'Notifications', 'Files'] },
];

const enabled = [
	{ label: 'Enabled', value: 'true' },
	{ label: 'Disabled', value: 'false' },
];

export const FIELDS: FieldDef[] = [
	// LLM — connection
	{
		key: 'llm_provider', section: 'llm', group: 'Connection', label: 'Provider', description: 'Which LLM service to use.',
		kind: 'select', default: 'openrouter', keepOnReset: true,
		options: [
			{ label: 'OpenRouter', value: 'openrouter' },
			{ label: 'Ollama (local)', value: 'ollama' },
			{ label: 'Ollama Cloud', value: 'ollama_cloud' },
			{ label: 'OpenAI-compatible (llama.cpp, etc.)', value: 'openai_compatible' },
		],
	},
	{ key: 'openrouter_api_key', section: 'llm', group: 'Connection', provider: 'openrouter', label: 'API key', description: 'Create one at openrouter.ai/keys.', kind: 'password', default: '', placeholder: 'sk-or-…', keepOnReset: true },
	{ key: 'openrouter_model', section: 'llm', group: 'Connection', provider: 'openrouter', label: 'Model', description: 'Search and select a model from OpenRouter.', kind: 'model', default: 'openai/gpt-4.1-mini', keepOnReset: true },
	{ key: 'ollama_url', section: 'llm', group: 'Connection', provider: 'ollama', label: 'Server URL', description: 'Address of your Ollama server.', kind: 'url', default: 'http://localhost:11434', placeholder: 'http://localhost:11434', keepOnReset: true },
	{ key: 'ollama_model', section: 'llm', group: 'Connection', provider: 'ollama', label: 'Model', description: 'A model installed on the server.', kind: 'model', default: 'llama3', keepOnReset: true },
	{ key: 'ollama_cloud_api_key', section: 'llm', group: 'Connection', provider: 'ollama_cloud', label: 'API key', description: 'Create one at ollama.com/settings/keys.', kind: 'password', default: '', keepOnReset: true },
	{ key: 'ollama_cloud_model', section: 'llm', group: 'Connection', provider: 'ollama_cloud', label: 'Model', description: 'Search and select a model from Ollama Cloud.', kind: 'model', default: 'gpt-oss:120b', keepOnReset: true },
	{ key: 'ollama_cloud_url', section: 'llm', group: 'Connection', provider: 'ollama_cloud', label: 'Cloud URL', description: 'Change only for a different Ollama Cloud host.', kind: 'url', default: 'https://ollama.com', advanced: true, keepOnReset: true },
	{ key: 'openai_base_url', section: 'llm', group: 'Connection', provider: 'openai_compatible', label: 'API base URL', description: 'Including /v1, e.g. http://localhost:8080/v1 for llama.cpp.', kind: 'url', default: 'http://localhost:8080/v1', placeholder: 'http://localhost:8080/v1', keepOnReset: true },
	{ key: 'openai_api_key', section: 'llm', group: 'Connection', provider: 'openai_compatible', label: 'API key', description: 'Leave empty if the server does not need one.', kind: 'password', default: '', keepOnReset: true },
	{ key: 'openai_model', section: 'llm', group: 'Connection', provider: 'openai_compatible', label: 'Model', description: 'A model served by the server.', kind: 'model', default: '', keepOnReset: true },
	// LLM — generation
	{ key: 'llm_temperature', section: 'llm', group: 'Generation', label: 'Temperature', description: 'Lower is more consistent, higher more varied.', kind: 'range', default: '0.7', min: 0, max: 1, step: 0.05, keywords: 'creativity randomness' },
	{
		key: 'llm_reasoning_effort', section: 'llm', group: 'Generation', label: 'Thinking', description: 'How much the model may reason before answering, where supported.',
		kind: 'select', default: 'auto', keywords: 'reasoning effort',
		options: [
			{ label: 'Auto', value: 'auto' },
			{ label: 'Provider default', value: 'default' },
			{ label: 'Off', value: 'off' },
			{ label: 'On', value: 'on' },
			{ label: 'Low', value: 'low' },
			{ label: 'Medium', value: 'medium' },
			{ label: 'High', value: 'high' },
			{ label: 'Max', value: 'max' },
		],
	},
	{ key: 'llm_max_tokens', section: 'llm', group: 'Generation', label: 'Max output tokens', description: 'Upper limit per model reply; thinking counts toward it.', kind: 'number', default: '4096', min: 256, max: 200000, step: 1, advanced: true, unit: 'tokens' },
	{ key: 'openai_json_mode', section: 'llm', group: 'Generation', provider: 'openai_compatible', label: 'JSON mode', description: 'Ask the server for strict JSON replies. Turn off if it rejects the option.', kind: 'switch', default: 'true', options: enabled, advanced: true },
	// Search
	{
		key: 'search_provider', section: 'search', group: 'Provider', label: 'Search provider', description: 'Used for every search; the other one can be a backup.',
		kind: 'select', default: 'brave', keepOnReset: true,
		options: [
			{ label: 'Brave Search', value: 'brave' },
			{ label: 'Serper (Google)', value: 'serper' },
		],
	},
	{ key: 'brave_api_key', section: 'search', group: 'Provider', label: 'Brave Search API key', description: 'From api-dashboard.search.brave.com.', kind: 'password', default: '', keepOnReset: true },
	{ key: 'serper_api_key', section: 'search', group: 'Provider', label: 'Serper API key', description: 'From serper.dev.', kind: 'password', default: '', keepOnReset: true },
	{ key: 'search_fallback_enabled', section: 'search', group: 'Provider', label: 'Use the other provider as backup', description: 'When a search fails, try the other provider if it has a key.', kind: 'switch', default: 'true', options: enabled },
	{ key: 'search_results_per_query', section: 'search', group: 'Provider', label: 'Results per query', description: 'Search results requested for each query. Serper returns up to 100; Brave stops at 20.', kind: 'range', default: '20', min: 1, max: 100, step: 1 },
	{ key: 'brave_price_per_1000', section: 'search', group: 'Pricing', label: 'Brave price per 1,000 requests', description: 'In USD; leave 0 for the free plan.', kind: 'number', default: '0', min: 0, step: 0.01, unit: 'USD' },
	{ key: 'serper_price_per_1000', section: 'search', group: 'Pricing', label: 'Serper price per 1,000 requests', description: 'In USD.', kind: 'number', default: '0', min: 0, step: 0.01, unit: 'USD' },
	// Runs
	{ key: 'max_parallel_fetches', section: 'runs', group: 'Fetching', label: 'Parallel page loads', description: 'Pages loaded at the same time.', kind: 'range', default: '8', min: 1, max: 20, step: 1 },
	{ key: 'max_pages_per_query', section: 'runs', group: 'Fetching', label: 'Pages per query', description: 'New pages taken from each search query’s results.', kind: 'range', default: '10', min: 1, max: 50, step: 1 },
	{ key: 'fetch_timeout_seconds', section: 'runs', group: 'Fetching', label: 'Page load timeout', description: 'Give up on a page after this long.', kind: 'range', default: '15', min: 5, max: 120, step: 1, unit: 's', advanced: true },
	{ key: 'max_page_size_kb', section: 'runs', group: 'Fetching', label: 'Max page size', description: 'Larger pages are skipped.', kind: 'number', default: '5000', min: 100, max: 100000, step: 1, unit: 'KB', advanced: true },
	{ key: 'dedup_similarity_threshold', section: 'runs', group: 'Quality', label: 'Duplicate similarity', description: 'Rows this similar are merged into one. Higher merges less.', kind: 'range', default: '0.85', min: 0.5, max: 1, step: 0.01, keywords: 'dedup deduplication merge' },
	{ key: 'enable_content_truncation', section: 'runs', group: 'Content', label: 'Shorten long pages', description: 'Send only the first part of long pages to the model to save tokens.', kind: 'switch', default: 'true', options: enabled, advanced: true, keywords: 'truncation' },
	{ key: 'max_extraction_text_chars', section: 'runs', group: 'Content', label: 'Page text sent to the model', description: 'Characters per page when shortening is on.', kind: 'number', default: '12000', min: 1000, max: 500000, step: 1, unit: 'chars', advanced: true },
	{ key: 'max_pdf_text_chars', section: 'runs', group: 'Content', label: 'PDF text read', description: 'Characters read from each PDF.', kind: 'number', default: '500000', min: 1000, max: 5000000, step: 1, unit: 'chars', advanced: true },
	// Application
	{ key: 'notifications_enabled', section: 'app', group: 'Notifications', label: 'Run notifications', description: 'Notify when a run finishes while the window is in the background.', kind: 'switch', default: 'true', options: enabled },
	{ key: 'show_site_icons', section: 'app', group: 'Appearance', label: 'Site icons', description: 'Show website icons next to links (loaded from DuckDuckGo).', kind: 'switch', default: 'true', options: enabled, keywords: 'favicon' },
];

const byKey = new Map(FIELDS.map((field) => [field.key, field]));

export function fieldByKey(key: string): FieldDef | undefined {
	return byKey.get(key);
}

function provider(settings: Map<string, string>): LlmProvider {
	return (settings.get('llm_provider') || 'openrouter') as LlmProvider;
}

/** Fields shown for a section with the current provider. */
export function fieldsIn(section: SectionId, settings: Map<string, string>): FieldDef[] {
	return FIELDS.filter((f) => f.section === section && (!f.provider || f.provider === provider(settings)));
}

function numeric(field: FieldDef) {
	return field.kind === 'number' || field.kind === 'range';
}

function rangeText(field: FieldDef) {
	const whole = Number.isInteger(field.step ?? 1);
	const format = (n: number) => n.toLocaleString('en-US');
	if (field.max === undefined) return `Enter ${format(field.min ?? 0)} or more.`;
	return `Enter ${whole ? 'a whole number' : 'a number'} from ${format(field.min ?? 0)} to ${format(field.max)}.`;
}

/** An explanation when the value cannot be saved, otherwise null. */
export function validateField(field: FieldDef, value: string): string | null {
	const text = value.trim();
	if (numeric(field)) {
		// Prices may be left empty: no price recorded.
		if (!text && field.group === 'Pricing') return null;
		const n = Number(text);
		const whole = Number.isInteger(field.step ?? 1);
		if (
			!text ||
			!Number.isFinite(n) ||
			(field.min !== undefined && n < field.min) ||
			(field.max !== undefined && n > field.max) ||
			(whole && !Number.isInteger(n))
		)
			return rangeText(field);
	}
	if (field.kind === 'url' && text && !/^https?:\/\/\S+$/i.test(text))
		return 'Enter an address starting with http:// or https://.';
	return null;
}

/** Problems with the values of the fields currently shown. */
export function validationErrors(settings: Map<string, string>): Record<string, string> {
	const errors: Record<string, string> = {};
	for (const section of SECTIONS)
		for (const field of fieldsIn(section.id, settings)) {
			const error = validateField(field, settings.get(field.key) ?? field.default);
			if (error) errors[field.key] = error;
		}
	return errors;
}

export function isDefault(field: FieldDef, value: string): boolean {
	if (numeric(field)) return Number(value) === Number(field.default) && value.trim() !== '';
	return value === field.default;
}

/** Values "Reset section" applies: defaults for everything but keys, providers and connections. */
export function sectionResetValues(section: SectionId, settings: Map<string, string>): Record<string, string> {
	return Object.fromEntries(
		fieldsIn(section, settings)
			.filter((field) => !field.keepOnReset)
			.map((field) => [field.key, field.default])
	);
}

/** Fields matching a settings search; empty for an empty query. */
export function matchingFields(query: string, settings: Map<string, string>): FieldDef[] {
	const words = query.toLowerCase().split(/\s+/).filter(Boolean);
	if (!words.length) return [];
	return SECTIONS.flatMap((section) => fieldsIn(section.id, settings)).filter((field) => {
		const text = `${field.label} ${field.description} ${field.keywords ?? ''}`.toLowerCase();
		return words.every((word) => text.includes(word));
	});
}

/** A proxy address with its password hidden. */
export function maskProxyUrl(raw: string): string {
	try {
		const url = new URL(raw);
		if (!url.password) return raw;
		return raw.replace(`:${url.password}@`, ':••••@');
	} catch {
		return raw;
	}
}
