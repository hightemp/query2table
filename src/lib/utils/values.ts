import { t } from '$lib/i18n';
export function formatValue(value: unknown, detailed = false): string {
	if (value === null || value === undefined || value === '') return '—';
	if (Array.isArray(value))
		return value.length
			? value.map((item) => formatValue(item, detailed)).join(detailed ? '\n' : ' · ')
			: '[]';
	if (typeof value === 'object') return JSON.stringify(value, null, detailed ? 2 : undefined);
	return String(value);
}

export function webUrl(value: unknown): string | null {
	if (typeof value !== 'string') return null;
	try {
		const url = new URL(value);
		return ['http:', 'https:'].includes(url.protocol) ? url.href : null;
	} catch {
		return null;
	}
}

export function urlLabel(value: string): string {
	try {
		return new URL(value).hostname;
	} catch {
		return value;
	}
}

const MISSING = new Set(['', '-', '—', 'n/a', 'na', 'none', 'null', 'unknown', 'not found', 'not available', 'not specified']);

/** Values that only say the information was not found. */
export function isMissing(value: unknown): boolean {
	if (value === null || value === undefined) return true;
	if (Array.isArray(value)) return value.length === 0;
	return typeof value === 'string' && MISSING.has(value.trim().toLowerCase());
}

/** Numeric value of a number cell, accepting strings such as "1,200" or "$3.5". */
export function parseNumber(value: unknown): number | null {
	if (typeof value === 'number') return Number.isFinite(value) ? value : null;
	if (typeof value !== 'string') return null;
	const cleaned = value.trim().replace(/^[$€£¥]/, '').replace(/[,\s_]/g, '');
	if (!/^[-+]?\d*\.?\d+(e[-+]?\d+)?%?$/i.test(cleaned)) return null;
	return Number(cleaned.replace('%', ''));
}

export function parseBoolean(value: unknown): boolean | null {
	if (typeof value === 'boolean') return value;
	if (typeof value !== 'string') return null;
	const text = value.trim().toLowerCase();
	if (['true', 'yes', 'y', '1'].includes(text)) return true;
	if (['false', 'no', 'n', '0'].includes(text)) return false;
	return null;
}

/** Date from an ISO-like date string; other text is left for display as is. */
export function parseDate(value: unknown): Date | null {
	if (typeof value !== 'string' || !/^\d{4}-\d{2}(-\d{2})?([T ][\d:.]+(Z|[+-]\d{2}:?\d{2})?)?$/.test(value.trim()))
		return null;
	const date = new Date(value.trim());
	return Number.isNaN(date.getTime()) ? null : date;
}

const numberFormat = new Intl.NumberFormat(undefined, { maximumFractionDigits: 6 });

/** Display text of a cell according to its schema type. */
export function formatCell(value: unknown, type: string): string {
	if (isMissing(value)) return typeof value === 'string' && value.trim() ? value : '—';
	if (type === 'number') {
		const number = parseNumber(value);
		if (number !== null)
			return typeof value === 'string' && value.trim().endsWith('%')
				? `${numberFormat.format(number)}%`
				: numberFormat.format(number);
	}
	if (type === 'boolean') {
		const flag = parseBoolean(value);
		if (flag !== null) return flag ? t('common.yes') : t('common.no');
	}
	if (type === 'date') {
		const date = parseDate(value);
		if (date) {
			const dayOnly = /^\d{4}-\d{2}(-\d{2})?$/.test(String(value).trim());
			return dayOnly
				? date.toLocaleDateString(undefined, { timeZone: 'UTC', year: 'numeric', month: 'short', ...(String(value).trim().length > 7 ? { day: 'numeric' } : {}) })
				: date.toLocaleString();
		}
	}
	return formatValue(value);
}

/** Value used for sorting: numbers and dates compare numerically, missing values sort last. */
export function sortValue(value: unknown, type: string): number | string | null {
	if (isMissing(value)) return null;
	if (type === 'number') return parseNumber(value) ?? formatValue(value).toLowerCase();
	if (type === 'boolean') {
		const flag = parseBoolean(value);
		if (flag !== null) return flag ? 1 : 0;
	}
	if (type === 'date') return parseDate(value)?.getTime() ?? formatValue(value).toLowerCase();
	return formatValue(value).toLowerCase();
}

const ACRONYMS = new Set(['id', 'url', 'ceo', 'cto', 'cfo', 'api', 'ai', 'usd', 'eur', 'gdp', 'seo', 'ui', 'ux', 'llm', 'faq', 'pdf', 'hq', 'ip', 'sdk', 'os']);

/** Readable header for machine-style column names such as `ceo_name` → "CEO name". */
export function columnLabel(name: string): string {
	// Names that already read like text ("Company Name", "Count") are kept as written.
	if (/\s/.test(name) || (/^[A-Z]/.test(name) && !/[_-]|[a-z][A-Z]/.test(name))) return name;
	const words = name
		.replace(/([a-z])([A-Z])/g, '$1 $2')
		.split(/[_\s-]+/)
		.filter(Boolean)
		.map((word) => (ACRONYMS.has(word.toLowerCase()) ? word.toUpperCase() : word.toLowerCase()));
	if (!words.length) return name;
	if (!ACRONYMS.has(words[0].toLowerCase())) words[0] = words[0][0].toUpperCase() + words[0].slice(1);
	return words.join(' ');
}
