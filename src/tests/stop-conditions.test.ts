import { describe, it, expect } from 'vitest';
import {
	parseStopConditions,
	stopInputFromSettings,
	formatUsd,
	formatMinutes,
} from '$lib/utils/stopConditions';
import { configurationProblems } from '$lib/utils/runConfig';

describe('stop conditions', () => {
	it('converts minutes to seconds and always sends every limit', () => {
		expect(parseStopConditions({ target: '25', budget: '0.5', duration: '15' })).toEqual({
			conditions: { target_row_count: 25, max_budget_usd: 0.5, max_duration_seconds: 900 },
			errors: {},
		});
	});

	it('rejects empty, fractional and out-of-range values instead of dropping them', () => {
		const { conditions, errors } = parseStopConditions({ target: '2.5', budget: '', duration: '0' });
		expect(conditions).toBeNull();
		expect(Object.keys(errors).sort()).toEqual(['budget', 'duration', 'target']);
		expect(parseStopConditions({ target: '10', budget: '0', duration: '5' }).errors.budget).toMatch(
			'$0.01'
		);
	});

	it('starts from the last used values saved in settings', () => {
		const settings = new Map([
			['target_row_count', '120'],
			['max_budget_usd', '2.5'],
			['max_duration_seconds', '1800'],
		]);
		expect(stopInputFromSettings(settings)).toEqual({
			target: '120',
			budget: '2.50',
			duration: '30',
		});
		expect(stopInputFromSettings(new Map([['max_budget_usd', 'oops']])).budget).toBe('1.00');
	});

	it('formats the summary', () => {
		expect(formatUsd(1)).toBe('$1.00');
		expect(formatUsd(0.125)).toBe('$0.125');
		expect(formatMinutes(20)).toBe('0 min');
		expect(formatMinutes(600)).toBe('10 min');
		expect(formatMinutes(7200)).toBe('2 h');
	});
});

describe('run configuration check', () => {
	const base = { search_provider: 'brave', brave_api_key: 'key' };

	it('does not block when settings have not been loaded', () => {
		expect(configurationProblems(new Map())).toEqual([]);
	});

	it('requires the active LLM provider credentials and model', () => {
		const problems = configurationProblems(
			new Map(Object.entries({ ...base, llm_provider: 'openrouter', openrouter_model: '' }))
		);
		expect(problems).toEqual([
			{ message: 'OpenRouter needs API key and model.', section: 'llm' },
		]);
		expect(
			configurationProblems(
				new Map(Object.entries({ ...base, llm_provider: 'ollama', ollama_url: 'http://x', ollama_model: 'llama3' }))
			)
		).toEqual([]);
	});

	it('requires the primary search provider key', () => {
		const problems = configurationProblems(
			new Map(
				Object.entries({
					llm_provider: 'ollama',
					ollama_url: 'http://x',
					ollama_model: 'm',
					search_provider: 'serper',
					brave_api_key: 'only brave',
				})
			)
		);
		expect(problems).toEqual([{ message: 'Serper needs an API key.', section: 'search' }]);
	});
});
