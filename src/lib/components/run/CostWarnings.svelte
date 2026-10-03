<script lang="ts">
	import type { Accounting } from '$lib/types';
	import { TriangleAlertIcon } from '@lucide/svelte';
	let { accounting = null }: { accounting?: Accounting | null } = $props();
	const names: Record<string, string> = {
		brave: 'Brave Search',
		serper: 'Serper',
		openrouter: 'OpenRouter',
		ollama: 'Ollama',
		ollama_cloud: 'Ollama Cloud',
		openai_compatible: 'OpenAI-compatible',
	};
	let limitReached = $derived(
		!!accounting &&
			accounting.spent_usd >= accounting.max_budget_usd - accounting.max_budget_usd * 1e-12
	);
	let unknownProviders = $derived.by(() => {
		const totals = new Map<string, number>();
		for (const line of accounting?.breakdown ?? []) {
			if (line.unpriced_calls > 0)
				totals.set(line.provider, (totals.get(line.provider) ?? 0) + line.unpriced_calls);
		}
		return [...totals]
			.map(([provider, count]) => `${names[provider] ?? provider}: ${count}`)
			.join(' · ');
	});
	// Search prices live in the Search section, model rates under LLM.
	let pricingSection = $derived(
		(accounting?.breakdown ?? []).some(
			(line) => line.unpriced_calls > 0 && !['brave', 'serper'].includes(line.provider)
		)
			? 'llm'
			: 'search'
	);
</script>

{#if limitReached}<p class="cost-warning" role="status">
		<TriangleAlertIcon size={14} /><span
			>Spending limit reached. New requests have stopped; requests already in flight may still be
			charged.</span
		>
	</p>{/if}
{#if accounting && accounting.unpriced_calls > 0}<p
		class="cost-warning"
		role="status"
		title="These charges are excluded from the total and the spending limit. Usage & cost has details."
	>
		<TriangleAlertIcon size={14} /><span
			>{accounting.unpriced_calls}
			{accounting.unpriced_calls === 1 ? 'request' : 'requests'} with unknown cost, not counted
			toward the limit{#if unknownProviders}{' '}({unknownProviders}){/if}. <a href={`/settings#settings-${pricingSection}`}>Set prices</a></span
		>
	</p>{/if}

<style>
	.cost-warning {
		display: flex;
		align-items: flex-start;
		gap: 6px;
		color: var(--app-warning);
		font-size: var(--app-text-sm);
		overflow-wrap: anywhere;
	}
	.cost-warning :global(svg) {
		margin-top: 2px;
	}
	a {
		color: inherit;
		text-decoration: underline;
	}
</style>
