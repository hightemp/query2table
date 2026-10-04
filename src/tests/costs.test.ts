import { afterEach, describe, expect, it } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import CostSummary from '$lib/components/run/CostSummary.svelte';
import RunNotices from '$lib/components/run/RunNotices.svelte';
import ModelPricing from '$lib/components/settings/ModelPricing.svelte';
import { storedCosts } from '$lib/utils/costs';
import { pricingScope, validPricingOverrides } from '$lib/utils/pricing';
import type { Accounting } from '$lib/types';

afterEach(cleanup);
const accounting: Accounting = {
	spent_usd: 0.25,
	max_budget_usd: 1,
	reported_usd: 0.25,
	estimated_usd: 0,
	reported_calls: 1,
	estimated_calls: 0,
	unpriced_calls: 0,
	pending_calls: 0,
	missing_usage_calls: 0,
	prompt_tokens: 100,
	completion_tokens: 50,
	reasoning_tokens: 20,
	cached_prompt_tokens: 0,
	llm_calls: 1,
	search_calls: 0,
	fetch_calls: 0,
	breakdown: [],
};
describe('honest cost display and model-scoped rates', () => {
	it('explains unknown attempts by provider and leaves zero-rate searches quiet', async () => {
		const line = (provider: string, unknown: number) => ({
			provider,
			model: 'fixture',
			calls: unknown,
			unpriced_calls: unknown,
			reported_usd: 0,
			estimated_usd: 0,
			prompt_tokens: 0,
			completion_tokens: 0,
			pricing: null,
		});
		const unknown = {
			...accounting,
			unpriced_calls: 50,
			search_calls: 48,
			breakdown: [
				line('brave', 27),
				line('serper', 21),
				line('openrouter', 1),
				line('openrouter', 1),
			],
		};
		const view = render(RunNotices, { issues: [], accounting: unknown, runStatus: 'completed' });
		await fireEvent.click(screen.getByRole('button', { name: /1 notice/ }));
		expect(screen.getByRole('article', { name: '50 requests with unknown cost' })).toHaveTextContent(
			'They are not counted toward the spending limit (Brave Search: 27 · Serper: 21 · OpenRouter: 2).'
		);
		// Unknown model rates are set in the LLM section, search prices in Search.
		expect(screen.getByRole('link', { name: 'Set prices' })).toHaveAttribute(
			'href',
			'/settings#settings-llm'
		);
		await view.rerender({ accounting: { ...accounting, search_calls: 48, estimated_calls: 48 } });
		expect(screen.queryByRole('region', { name: 'Run notices' })).not.toBeInTheDocument();
		cleanup();
		render(CostSummary, { accounting: unknown });
		expect(
			screen.getByText(/A saved rate or provider billing response is missing/)
		).toHaveTextContent('Settings changes apply to new runs');
	});
	it('distinguishes reported, estimated, partial and unknown totals', async () => {
		const view = render(CostSummary, { accounting });
		expect(screen.getByText('Reported cost $0.2500')).toBeInTheDocument();
		await view.rerender({ accounting: { ...accounting, estimated_calls: 1 } });
		expect(screen.getByText('Estimated cost $0.2500')).toBeInTheDocument();
		await view.rerender({ accounting: { ...accounting, unpriced_calls: 1 } });
		expect(screen.getByText('Partial cost $0.2500')).toBeInTheDocument();
		render(RunNotices, { issues: [], accounting: { ...accounting, unpriced_calls: 1 }, runStatus: 'completed' });
		expect(screen.getByRole('button', { name: /1 notice/ })).toHaveTextContent('1 request with unknown cost');
		await view.rerender({
			accounting: {
				...accounting,
				reported_calls: 0,
				reported_usd: 0,
				spent_usd: 0,
				unpriced_calls: 1,
			},
		});
		expect(screen.getByText('Cost unknown')).toBeInTheDocument();
	});
	it('labels old stored totals as estimates and tolerates malformed history stats', () => {
		expect(storedCosts('{bad')).toEqual({ accounting: null, legacy: null });
		expect(storedCosts('{"spent_usd":0.2}')).toEqual({ accounting: null, legacy: 0.2 });
		render(CostSummary, { legacy: 0.2 });
		expect(screen.getByText('Historical estimate $0.2000')).toBeInTheDocument();
	});
	it('manual zero rates are valid while blanks, negatives and invalid JSON are not', () => {
		const key = pricingScope('ollama', 'http://localhost:11434/', 'local');
		expect(key).toBe(pricingScope('ollama', 'http://localhost:11434', 'local'));
		expect(
			validPricingOverrides(
				JSON.stringify({ [key]: { input_per_million: 0, output_per_million: 0 } })
			)
		).toBe(true);
		expect(
			validPricingOverrides(
				JSON.stringify({ [key]: { input_per_million: null, output_per_million: 0 } })
			)
		).toBe(false);
		expect(
			validPricingOverrides(
				JSON.stringify({ [key]: { input_per_million: -1, output_per_million: 0 } })
			)
		).toBe(false);
		expect(validPricingOverrides('[]')).toBe(false);
	});
	it('model or endpoint changes never reuse another scope’s custom rate', async () => {
		const key = pricingScope('ollama', 'http://localhost:11434', 'alpha');
		let value = JSON.stringify({ [key]: { input_per_million: 1, output_per_million: 2 } });
		const view = render(ModelPricing, {
			provider: 'ollama',
			endpoint: 'http://localhost:11434',
			model: 'alpha',
			value,
			onchange: (next: string) => {
				value = next;
			},
		});
		expect(screen.getByRole('checkbox')).toBeChecked();
		expect(screen.getByLabelText('Input USD / 1M tokens')).toHaveValue(1);
		await view.rerender({ model: 'beta' });
		expect(screen.getByRole('checkbox')).not.toBeChecked();
		await fireEvent.click(screen.getByRole('checkbox'));
		expect(Object.keys(JSON.parse(value))).toHaveLength(2);
		expect(JSON.parse(value)[key].input_per_million).toBe(1);
		await view.rerender({ value });
		expect(screen.getByRole('checkbox')).toBeChecked();
		await view.rerender({ model: 'alpha', endpoint: 'http://another-server' });
		expect(screen.getByRole('checkbox')).not.toBeChecked();
	});
});
