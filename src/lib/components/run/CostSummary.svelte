<script lang="ts">
	import { t } from '$lib/i18n';
	import type { Accounting } from '$lib/types';
	let {
		accounting = null,
		legacy = null,
		inline = false,
	}: { accounting?: Accounting | null; legacy?: number | null; inline?: boolean } = $props();
	const usd = (amount: number) =>
		amount > 0 && amount < 0.000001
			? '< $0.000001'
			: `$${amount.toFixed(6).replace(/0{1,2}$/, '')}`;
	let title = $derived.by(() => {
		if (!accounting)
			return legacy !== null ? t('cost.historical', { amount: usd(legacy) }) : t('cost.notRecorded');
		if (accounting.llm_calls + accounting.search_calls === 0) return t('cost.noRequests');
		const priced = accounting.reported_calls + accounting.estimated_calls;
		if (!priced && accounting.pending_calls) return t('cost.pending');
		if (!priced && accounting.unpriced_calls) return t('cost.unknown');
		if (accounting.unpriced_calls || accounting.pending_calls)
			return t('cost.partial', { amount: usd(accounting.spent_usd) });
		return t(accounting.estimated_calls ? 'cost.estimated' : 'cost.reported', { amount: usd(accounting.spent_usd) });
	});
</script>

<details class="cost-summary" class:inline>
	<summary>{title}<span>{t('cost.usage')}</span></summary>
	<div class="cost-details">
		{#if accounting}
			<p>{t('cost.charges', { reported: usd(accounting.reported_usd), estimated: usd(accounting.estimated_usd) })}</p>
			<p>
				{t('cost.attempts', { llm: accounting.llm_calls, search: accounting.search_calls, pending: accounting.pending_calls })}
			</p>
			<p>
				{t('cost.tokens', {
					input: accounting.prompt_tokens,
					output: accounting.completion_tokens,
					cached: accounting.cached_prompt_tokens,
				})}
			</p>
			{#if accounting.missing_usage_calls}<p>{t('cost.missingUsage', { count: accounting.missing_usage_calls })}</p>{/if}
			{#each accounting.breakdown as line}
				<div class="cost-line">
					<strong>{line.provider} · {line.model}</strong
					>{#if line.requested_model && line.requested_model !== line.model}<span
							>{t('cost.requestedAs', { model: line.requested_model })}</span
						>{/if}<span
						>{t('cost.lineAttempts', { calls: line.calls, reported: usd(line.reported_usd), estimated: usd(line.estimated_usd) })}{#if line.unpriced_calls}{t(
								'cost.lineUnpriced',
								{ count: line.unpriced_calls }
							)}{/if}</span
					>
					{#if line.pricing}<span
							>{line.pricing.source}: {#if ['brave', 'serper'].includes(line.provider)}{t('cost.perRequest', {
									amount: usd(line.pricing.per_request ?? 0),
								})}{:else}{t('cost.perMillion', {
									input: usd(line.pricing.input_per_million),
									output: usd(line.pricing.output_per_million),
								})}{#if line.pricing.cached_input_per_million != null}{t('cost.cachedPerMillion', {
										amount: usd(line.pricing.cached_input_per_million),
									})}{/if}{#if line.pricing.cache_write_per_million != null}{t('cost.cacheWritePerMillion', {
										amount: usd(line.pricing.cache_write_per_million),
									})}{/if}{#if line.pricing.per_request}{t('cost.alsoPerRequest', {
										amount: usd(line.pricing.per_request),
									})}{/if}{/if}.</span
						>{#if line.pricing.credit_based}<span>{t('cost.creditBased')}</span>{/if}{/if}
				</div>
			{/each}
			<p>{t('cost.cacheNote')}</p>
			<p>{t('cost.limit', { amount: usd(accounting.max_budget_usd) })}</p>
			{#if accounting.unpriced_calls > 0}<p>{t('cost.unpricedNote')}</p>{/if}
		{:else if legacy !== null}<p>{t('cost.legacyNote')}</p>{/if}
	</div>
</details>

<style>
	.cost-summary {
		position: relative;
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
		position: absolute;
		top: calc(100% + 6px);
		right: 0;
		z-index: 20;
		width: min(560px, 80vw);
		max-height: min(360px, 50vh);
		overflow: auto;
		padding: 8px 14px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		background: var(--app-panel);
		box-shadow: var(--app-shadow-popover);
		color: var(--app-text);
		overflow-wrap: anywhere;
	}
	/* In page flow (history), open below the summary instead of over the results. */
	.cost-summary.inline .cost-details {
		position: static;
		width: auto;
		max-height: 160px;
		margin-top: 8px;
		border: 0;
		border-left: 2px solid var(--app-border);
		border-radius: 0;
		background: transparent;
		box-shadow: none;
		color: inherit;
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
</style>
