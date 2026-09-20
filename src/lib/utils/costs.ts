import type { Accounting } from '$lib/types';
export function storedCosts(raw: string | null | undefined): {
	accounting: Accounting | null;
	legacy: number | null;
} {
	try {
		const stats = JSON.parse(raw || '{}');
		const accounting = stats.accounting;
		const fields = [
			'spent_usd',
			'max_budget_usd',
			'reported_usd',
			'estimated_usd',
			'reported_calls',
			'estimated_calls',
			'unpriced_calls',
			'pending_calls',
			'missing_usage_calls',
			'prompt_tokens',
			'completion_tokens',
			'reasoning_tokens',
			'cached_prompt_tokens',
			'llm_calls',
			'search_calls',
			'fetch_calls',
		];
		if (
			accounting &&
			fields.every(
				(key) =>
					typeof accounting[key] === 'number' &&
					Number.isFinite(accounting[key]) &&
					accounting[key] >= 0
			) &&
			Array.isArray(accounting.breakdown) &&
			accounting.breakdown.every(
				(line: any) =>
					line &&
					typeof line.provider === 'string' &&
					typeof line.model === 'string' &&
					[
						'calls',
						'reported_usd',
						'estimated_usd',
						'unpriced_calls',
						'prompt_tokens',
						'completion_tokens',
					].every(
						(key) => typeof line[key] === 'number' && Number.isFinite(line[key]) && line[key] >= 0
					) &&
					(!line.pricing ||
						(typeof line.pricing.source === 'string' &&
							['input_per_million', 'output_per_million', 'per_request'].every(
								(key) =>
									typeof line.pricing[key] === 'number' &&
									Number.isFinite(line.pricing[key]) &&
									line.pricing[key] >= 0
							)))
			)
		) {
			return { accounting, legacy: null };
		}
		return {
			accounting: null,
			legacy:
				!('accounting' in stats) &&
				typeof stats.spent_usd === 'number' &&
				Number.isFinite(stats.spent_usd) &&
				stats.spent_usd >= 0
					? stats.spent_usd
					: null,
		};
	} catch {
		return { accounting: null, legacy: null };
	}
}
