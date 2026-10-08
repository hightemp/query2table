import { t, type MessageKey } from '$lib/i18n';

/** What each setting is, where it lives on the Settings page and which values it accepts. */

export type SectionId = 'llm' | 'search' | 'runs' | 'network' | 'app';
export type LlmProvider = 'openrouter' | 'ollama' | 'ollama_cloud' | 'openai_compatible';
export type FieldKind = 'select' | 'text' | 'url' | 'password' | 'number' | 'range' | 'switch' | 'model';

export interface SectionDef {
	id: SectionId;
	/** Sub-headings in order. */
	groups: string[];
}

export interface FieldDef {
	key: string;
	section: SectionId;
	group: string;
	kind: FieldKind;
	/** Value the backend uses when nothing was chosen. */
	default: string;
	/** Values of a select; their names come from the catalog. */
	options?: string[];
	placeholder?: string;
	min?: number;
	max?: number;
	step?: number;
	/** Shown after the value. */
	unit?: 'tokens' | 'chars' | 's' | 'KB' | 'MB' | 'USD';
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
	{ id: 'llm', groups: ['Connection', 'Generation', 'Images', 'Pricing'] },
	{ id: 'search', groups: ['Provider', 'Pricing'] },
	{ id: 'runs', groups: ['Fetching', 'Quality', 'Content'] },
	{ id: 'network', groups: ['Proxies'] },
	{ id: 'app', groups: ['Appearance', 'Notifications', 'Files'] },
];

export const FIELDS: FieldDef[] = [
	// LLM — connection
	{
		key: 'llm_provider', section: 'llm', group: 'Connection',
		kind: 'select', default: 'openrouter', keepOnReset: true,
		options: [
			'openrouter',
			'ollama',
			'ollama_cloud',
			'openai_compatible',
		],
	},
	{ key: 'openrouter_api_key', section: 'llm', group: 'Connection', provider: 'openrouter', kind: 'password', default: '', placeholder: 'sk-or-…', keepOnReset: true },
	{ key: 'openrouter_model', section: 'llm', group: 'Connection', provider: 'openrouter', kind: 'model', default: 'openai/gpt-4.1-mini', keepOnReset: true },
	{ key: 'ollama_url', section: 'llm', group: 'Connection', provider: 'ollama', kind: 'url', default: 'http://localhost:11434', placeholder: 'http://localhost:11434', keepOnReset: true },
	{ key: 'ollama_model', section: 'llm', group: 'Connection', provider: 'ollama', kind: 'model', default: 'llama3', keepOnReset: true },
	{ key: 'ollama_cloud_api_key', section: 'llm', group: 'Connection', provider: 'ollama_cloud', kind: 'password', default: '', keepOnReset: true },
	{ key: 'ollama_cloud_model', section: 'llm', group: 'Connection', provider: 'ollama_cloud', kind: 'model', default: 'gpt-oss:120b', keepOnReset: true },
	{ key: 'ollama_cloud_url', section: 'llm', group: 'Connection', provider: 'ollama_cloud', kind: 'url', default: 'https://ollama.com', advanced: true, keepOnReset: true },
	{ key: 'openai_base_url', section: 'llm', group: 'Connection', provider: 'openai_compatible', kind: 'url', default: 'http://localhost:8080/v1', placeholder: 'http://localhost:8080/v1', keepOnReset: true },
	{ key: 'openai_api_key', section: 'llm', group: 'Connection', provider: 'openai_compatible', kind: 'password', default: '', keepOnReset: true },
	{ key: 'openai_model', section: 'llm', group: 'Connection', provider: 'openai_compatible', kind: 'model', default: '', keepOnReset: true },
	// LLM — generation
	{ key: 'llm_temperature', section: 'llm', group: 'Generation', kind: 'range', default: '0.7', min: 0, max: 1, step: 0.05, keywords: 'creativity randomness' },
	{
		key: 'llm_reasoning_effort', section: 'llm', group: 'Generation',
		kind: 'select', default: 'auto', keywords: 'reasoning effort',
		options: [
			'auto',
			'default',
			'off',
			'on',
			'low',
			'medium',
			'high',
			'max',
		],
	},
	{ key: 'llm_max_tokens', section: 'llm', group: 'Generation', kind: 'number', default: '4096', min: 256, max: 200000, step: 1, advanced: true, unit: 'tokens' },
	{ key: 'openai_json_mode', section: 'llm', group: 'Generation', provider: 'openai_compatible', kind: 'switch', default: 'true', advanced: true },
	// LLM — images (attached pictures and scanned pages)
	{ key: 'llm_vision', section: 'llm', group: 'Images', kind: 'select', default: 'auto', options: ['auto', 'on', 'off'], keywords: 'vision pictures photos scans' },
	{ key: 'vision_model', section: 'llm', group: 'Images', kind: 'model', default: '', keepOnReset: true, keywords: 'vision pictures photos scans ocr' },
	{ key: 'vision_max_pages', section: 'llm', group: 'Images', kind: 'number', default: '50', min: 1, max: 1000, step: 1, advanced: true, keywords: 'scans ocr pdf' },
	// Search
	{
		key: 'search_provider', section: 'search', group: 'Provider',
		kind: 'select', default: 'brave', keepOnReset: true,
		options: [
			'brave',
			'serper',
		],
	},
	{ key: 'brave_api_key', section: 'search', group: 'Provider', kind: 'password', default: '', keepOnReset: true },
	{ key: 'serper_api_key', section: 'search', group: 'Provider', kind: 'password', default: '', keepOnReset: true },
	{ key: 'search_fallback_enabled', section: 'search', group: 'Provider', kind: 'switch', default: 'true' },
	{ key: 'search_results_per_query', section: 'search', group: 'Provider', kind: 'range', default: '20', min: 1, max: 100, step: 1 },
	{ key: 'brave_price_per_1000', section: 'search', group: 'Pricing', kind: 'number', default: '0', min: 0, step: 0.01, unit: 'USD' },
	{ key: 'serper_price_per_1000', section: 'search', group: 'Pricing', kind: 'number', default: '0', min: 0, step: 0.01, unit: 'USD' },
	// Runs
	{ key: 'max_parallel_fetches', section: 'runs', group: 'Fetching', kind: 'range', default: '8', min: 1, max: 20, step: 1 },
	{ key: 'max_pages_per_query', section: 'runs', group: 'Fetching', kind: 'range', default: '10', min: 1, max: 50, step: 1 },
	{ key: 'fetch_timeout_seconds', section: 'runs', group: 'Fetching', kind: 'range', default: '15', min: 5, max: 120, step: 1, unit: 's', advanced: true },
	{ key: 'max_page_size_kb', section: 'runs', group: 'Fetching', kind: 'number', default: '5000', min: 100, max: 100000, step: 1, unit: 'KB', advanced: true },
	{ key: 'dedup_similarity_threshold', section: 'runs', group: 'Quality', kind: 'range', default: '0.85', min: 0.5, max: 1, step: 0.01, keywords: 'dedup deduplication merge' },
	{ key: 'enable_content_truncation', section: 'runs', group: 'Content', kind: 'switch', default: 'true', advanced: true, keywords: 'truncation' },
	{ key: 'max_extraction_text_chars', section: 'runs', group: 'Content', kind: 'number', default: '12000', min: 1000, max: 500000, step: 1, unit: 'chars', advanced: true },
	{ key: 'max_pdf_text_chars', section: 'runs', group: 'Content', kind: 'number', default: '500000', min: 1000, max: 5000000, step: 1, unit: 'chars', advanced: true },
	{ key: 'attachment_max_mb', section: 'runs', group: 'Content', kind: 'number', default: '50', min: 1, max: 500, step: 1, unit: 'MB', advanced: true, keywords: 'attach files upload size' },
	// Application
	{ key: 'notifications_enabled', section: 'app', group: 'Notifications', kind: 'switch', default: 'true' },
	{ key: 'show_site_icons', section: 'app', group: 'Appearance', kind: 'switch', default: 'true', keywords: 'favicon' },
];

export function sectionLabel(id: SectionId): string {
	return t(`settings.section.${id}`);
}

export function sectionDescription(id: SectionId): string {
	return t(`settings.section.${id}.description`);
}

export function groupLabel(group: string): string {
	return t(`settings.group.${group}` as MessageKey);
}

export function fieldLabel(field: FieldDef): string {
	return t(`settings.field.${field.key}` as MessageKey);
}

export function fieldDescription(field: FieldDef): string {
	return t(`settings.field.${field.key}.description` as MessageKey);
}

export function optionLabel(field: FieldDef, value: string): string {
	return t(`settings.option.${field.key}.${value}` as MessageKey);
}

export function unitLabel(unit: NonNullable<FieldDef['unit']>): string {
	return t(`settings.unit.${unit}`);
}

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
	if (field.max === undefined) return t('settings.minError', { min: field.min ?? 0 });
	return t(whole ? 'settings.rangeWholeError' : 'settings.rangeError', { min: field.min ?? 0, max: field.max });
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
		return t('settings.urlError');
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
		const text = `${fieldLabel(field)} ${fieldDescription(field)} ${field.keywords ?? ''}`.toLowerCase();
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
