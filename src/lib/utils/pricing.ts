export function pricingScope(provider: string, endpoint: string, model: string): string {
	return JSON.stringify([provider, endpoint.trim().replace(/\/+$/, ''), model.trim()]);
}
export function validPricingOverrides(raw: string): boolean {
	try {
		const values = JSON.parse(raw || '{}');
		if (!values || typeof values !== 'object' || Array.isArray(values)) return false;
		return Object.values(values).every(
			(rate: any) =>
				rate &&
				[rate.input_per_million, rate.output_per_million].every(
					(value) => typeof value === 'number' && Number.isFinite(value) && value >= 0
				) &&
				[rate.cached_input_per_million, rate.cache_write_per_million, rate.per_request].every(
					(value) =>
						value == null || (typeof value === 'number' && Number.isFinite(value) && value >= 0)
				)
		);
	} catch {
		return false;
	}
}
