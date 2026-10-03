<script lang="ts">
	import type { Accounting } from '$lib/types';
	let {
		accounting = null,
		legacy = null,
	}: { accounting?: Accounting | null; legacy?: number | null } = $props();
	const usd = (amount: number) =>
		amount > 0 && amount < 0.000001
			? '< $0.000001'
			: `$${amount.toFixed(6).replace(/0{1,2}$/, '')}`;
	let title = $derived.by(() => {
		if (!accounting)
			return legacy !== null ? `Historical estimate ${usd(legacy)}` : 'Cost not recorded';
		if (accounting.llm_calls + accounting.search_calls === 0) return 'No provider requests yet';
		const priced = accounting.reported_calls + accounting.estimated_calls;
		if (!priced && accounting.pending_calls) return 'Cost pending';
		if (!priced && accounting.unpriced_calls) return 'Cost unknown';
		if (accounting.unpriced_calls || accounting.pending_calls)
			return `Partial cost ${usd(accounting.spent_usd)}`;
		return `${accounting.estimated_calls ? 'Estimated' : 'Reported'} cost ${usd(accounting.spent_usd)}`;
	});
	let unknownProviders = $derived.by(() => {
		const totals = new Map<string, number>();
		for (const line of accounting?.breakdown ?? []) {
			if (line.unpriced_calls > 0)
				totals.set(line.provider, (totals.get(line.provider) ?? 0) + line.unpriced_calls);
		}
		const names: Record<string, string> = {
			brave: 'Brave Search',
			serper: 'Serper',
			openrouter: 'OpenRouter',
			ollama: 'Ollama',
			ollama_cloud: 'Ollama Cloud',
			openai_compatible: 'OpenAI-compatible',
		};
		return [...totals]
			.map(([provider, count]) => `${names[provider] ?? provider}: ${count}`)
			.join(' · ');
	});
</script>

<details class="cost-summary">
	<summary>{title}<span>Usage &amp; cost</span></summary>
	<div class="cost-details">
		{#if accounting}
			<p>
				Reported charges: {usd(accounting.reported_usd)} · Rate estimates: {usd(
					accounting.estimated_usd
				)}
			</p>
			<p>
				{accounting.llm_calls} LLM attempts · {accounting.search_calls} search attempts · {accounting.pending_calls}
				in flight
			</p>
			<p>
				Known token totals: {accounting.prompt_tokens.toLocaleString()} input · {accounting.completion_tokens.toLocaleString()}
				output · {accounting.cached_prompt_tokens.toLocaleString()} cached input. Thinking tokens are
				included in output.
			</p>
			{#if accounting.missing_usage_calls}<p>
					Token usage was not reported for {accounting.missing_usage_calls} LLM attempts; these tokens
					are not counted as zero.
				</p>{/if}
			{#each accounting.breakdown as line}
				<div class="cost-line">
					<strong>{line.provider} · {line.model}</strong
					>{#if line.requested_model && line.requested_model !== line.model}<span
							>Requested as {line.requested_model}</span
						>{/if}<span
						>{line.calls} attempts · reported {usd(line.reported_usd)} · estimated {usd(
							line.estimated_usd
						)}{#if line.unpriced_calls}
							· {line.unpriced_calls} unpriced{/if}</span
					>
					{#if line.pricing}<span
							>{line.pricing.source}: {#if ['brave', 'serper'].includes(line.provider)}{usd(
									line.pricing.per_request
								)} per request{:else}{usd(line.pricing.input_per_million)} input / {usd(
									line.pricing.output_per_million
								)} output per million tokens{#if line.pricing.cached_input_per_million != null}; {usd(
										line.pricing.cached_input_per_million
									)} cached input per million{/if}{#if line.pricing.cache_write_per_million != null};
									{usd(line.pricing.cache_write_per_million)} cache write per million{/if}{#if line.pricing.per_request};
									{usd(line.pricing.per_request)} per request{/if}{/if}.</span
						>{#if line.pricing.credit_based}<span
								>Estimated usage-credit consumption; included credits, subscription fees and cash
								payments may differ.</span
							>{/if}{/if}
				</div>
			{/each}
			<p>
				When cache usage or cache rates are unavailable, token estimates use the regular input rate.
			</p>
			<p>
				Run spending limit: {usd(accounting.max_budget_usd)}. New requests stop when accounted
				spending reaches this limit. Requests already in flight can exceed it.
			</p>
			{#if accounting.unpriced_calls > 0}<p>
					A saved rate or provider billing response is missing for these attempts. Timeouts and
					cancelled requests may have no final billing information. Settings changes apply to new
					runs; earlier costs keep the information available during that run.
				</p>{/if}
		{:else if legacy !== null}<p>
				An earlier version recorded this estimate without actual usage or model-specific rates.
			</p>{/if}
	</div>
</details>
{#if accounting && accounting.spent_usd >= accounting.max_budget_usd - accounting.max_budget_usd * 1e-12}<p
		class="cost-warning"
		role="status"
	>
		Spending limit reached. New provider requests have stopped; requests already in flight may still
		incur charges.
	</p>{/if}
{#if accounting && accounting.unpriced_calls > 0}<p class="cost-warning" role="status">
		Request attempts with unknown cost: {accounting.unpriced_calls}.
		{#if unknownProviders}<span>{unknownProviders}.</span>{/if}
		These charges are excluded from the total and spending limit. Expand Usage &amp; cost for details
		or review <a href="/settings">pricing in Settings</a> for new runs.
	</p>{/if}

<style>
	.cost-summary {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		min-width: 0;
	}
	summary {
		cursor: pointer;
		font-weight: 600;
	}
	summary span {
		margin-left: 10px;
		font-weight: 400;
	}
	.cost-details {
		max-height: 160px;
		overflow: auto;
		border-left: 2px solid var(--app-border);
		padding: 4px 12px;
		margin-top: 8px;
		overflow-wrap: anywhere;
	}
	p {
		margin: 5px 0;
	}
	.cost-line {
		display: flex;
		flex-direction: column;
		margin: 8px 0;
		gap: 3px;
	}
	.cost-warning {
		color: var(--app-warning);
		font-size: var(--app-text-sm);
		margin: 4px 0;
		overflow-wrap: anywhere;
	}
	.cost-warning a {
		color: inherit;
		text-decoration: underline;
	}
</style>
