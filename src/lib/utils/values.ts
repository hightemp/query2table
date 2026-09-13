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
