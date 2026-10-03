<script lang="ts">
	import { tick } from 'svelte';
	import type { Accounting } from '$lib/types';
	import type { StopConditions } from '$lib/api/tauri';
	import type { ResearchTurnState } from '$lib/stores/run';
	import { ArrowUpIcon } from '@lucide/svelte';
	import EmptyState from '$lib/components/common/EmptyState.svelte';
	import ResearchTurn from './ResearchTurn.svelte';
	import FollowUpBox from './FollowUpBox.svelte';
	import { formatUsd } from '$lib/utils/stopConditions';

	const DEFAULT_LIMITS: Required<StopConditions> = {
		target_row_count: 16,
		max_budget_usd: 1,
		max_duration_seconds: 600,
	};
	let {
		turns,
		running = false,
		liveAccounting = null,
		limits = null,
		onask,
	}: {
		turns: ResearchTurnState[];
		/** The last turn is being answered. */
		running?: boolean;
		/** Usage of the turn being answered, before it is saved with the turn. */
		liveAccounting?: Accounting | null;
		limits?: StopConditions | null;
		/** Enables follow-up questions. */
		onask?: (question: string, limits: Required<StopConditions>) => Promise<void>;
	} = $props();
	let view = $state<HTMLDivElement>();
	let scrolled = $state(false);

	let fullLimits = $derived<Required<StopConditions>>({
		target_row_count: limits?.target_row_count ?? DEFAULT_LIMITS.target_row_count,
		max_budget_usd: limits?.max_budget_usd ?? DEFAULT_LIMITS.max_budget_usd,
		max_duration_seconds: limits?.max_duration_seconds ?? DEFAULT_LIMITS.max_duration_seconds,
	});
	let lastTurn = $derived(turns.at(-1));
	let suggestions = $derived(lastTurn?.answer ? lastTurn.followUps : []);
	let conversationCost = $derived.by(() => {
		const costs = turns.map((turn, i) =>
			turn.accounting ?? (i === turns.length - 1 && running ? liveAccounting : null)
		);
		if (!costs.some(Boolean)) return null;
		return costs.reduce((sum, cost) => sum + (cost?.spent_usd ?? 0), 0);
	});

	// A new question scrolls into view.
	let shownTurns = 0;
	$effect(() => {
		const count = turns.length;
		if (shownTurns && count > shownTurns)
			void tick().then(() =>
				view
					?.querySelector(`[data-turn="${count - 1}"]`)
					?.scrollIntoView({ block: 'start' })
			);
		shownTurns = count;
	});
</script>

<div
	class="research-view"
	bind:this={view}
	onscroll={(event) => (scrolled = event.currentTarget.scrollTop > 400)}
>
	{#if turns.some((turn) => turn.answer || turn.steps.length) || running}
		{#each turns as turn, i (turn.index)}
			<div data-turn={i} class:later={i > 0}>
				<ResearchTurn
					answer={turn.answer}
					steps={turn.steps}
					status={turn.status}
					running={running && i === turns.length - 1}
					question={turn.question}
					showQuestion={i > 0}
					cost={turns.length > 1 ? (turn.accounting?.spent_usd ?? null) : null}
				/>
			</div>
		{/each}
	{:else}
		<EmptyState>No research output yet.</EmptyState>
	{/if}
	{#if conversationCost !== null && turns.length > 1}
		<p class="conversation-cost">Conversation cost: {formatUsd(conversationCost)}</p>
	{/if}
	{#if onask}
		<FollowUpBox {suggestions} disabled={running} limits={fullLimits} {onask} />
	{/if}
	{#if scrolled}
		<button
			class="button sm back-to-top"
			aria-label="Back to top"
			title="Back to top"
			onclick={() => {
				if (view) view.scrollTop = 0;
				scrolled = false;
			}}><ArrowUpIcon size={15} />Top</button
		>
	{/if}
</div>

<style>
	.research-view {
		position: relative;
		min-height: 0;
		min-width: 0;
		overflow: auto;
		flex: 1;
		scrollbar-gutter: stable;
		padding: 0 12px 0 0;
		container: research / inline-size;
	}
	.later {
		padding-top: 20px;
		border-top: 1px solid var(--app-border);
	}
	.conversation-cost {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		text-align: right;
	}
	.back-to-top {
		position: sticky;
		bottom: 140px;
		float: right;
		box-shadow: var(--app-shadow-popover);
	}
</style>
