<script lang="ts">
	import { CheckIcon, ChevronDownIcon, ChevronRightIcon, CircleIcon, LoaderCircleIcon, MinusIcon, TriangleAlertIcon } from '@lucide/svelte';
	import { t } from '$lib/i18n';
	import { tooltip } from '$lib/actions/tooltip';
	import type { ProgressStats, SearchQueryInfo } from '$lib/types';
	import { queriesSummary, stageViews } from '$lib/utils/stages';

	let {
		runType,
		stage = null,
		status,
		progress = null,
		queries = [],
		compact = false,
	}: {
		runType: string;
		stage?: string | null;
		status: string;
		progress?: ProgressStats | null;
		queries?: SearchQueryInfo[];
		/** One line above the results; the full panel stands in for results not yet found. */
		compact?: boolean;
	} = $props();

	let views = $derived(stageViews(runType, stage, status, progress, queries));
	let finished = $derived(['completed', 'cancelled', 'failed'].includes(status));
	let listOpen = $state(false);
	// The full panel shows the queries while they run.
	let showList = $derived(listOpen || (!compact && stage === 'search'));
	let summary = $derived(queriesSummary(queries));
	const listId = $props.id();
</script>

{#snippet stateIcon(state: string)}
	{#if state === 'done'}<CheckIcon size={14} />
	{:else if state === 'active'}<LoaderCircleIcon size={14} class="spin" />
	{:else if state === 'skipped'}<MinusIcon size={14} />
	{:else}<CircleIcon size={10} />{/if}
{/snippet}

<section class="run-stages" class:compact aria-label={t('stages.region')}>
	{#if !(compact && finished)}
		<ol class="ladder">
			{#each views as view (view.id)}
				<li class="stage {view.state}" aria-current={view.state === 'active' ? 'step' : undefined}>
					<span class="icon">{@render stateIcon(view.state)}</span>
					<span class="label">{view.label}</span>
					{#if view.detail}<span class="detail">{view.detail}</span>{/if}
					{#if view.state === 'skipped'}<span class="detail">{t('stages.notNeeded')}</span>{/if}
				</li>
			{/each}
		</ol>
	{/if}
	{#if queries.length}
		<button class="list-toggle" aria-expanded={showList} aria-controls={listId} onclick={() => (listOpen = !showList)}>
			{#if showList}<ChevronDownIcon size={14} />{:else}<ChevronRightIcon size={14} />{/if}
			{t('stages.searchQueries')}<span class="summary">{summary}</span>
		</button>
		{#if showList}
			<ul class="queries" id={listId} aria-label={t('stages.searchQueries')}>
				{#each queries as query (query.id)}
					<li class="query {query.status}">
						<span class="icon">
							{#if query.status === 'completed'}<CheckIcon size={13} />
							{:else if query.status === 'running'}<LoaderCircleIcon size={13} class="spin" />
							{:else if query.status === 'failed'}<TriangleAlertIcon size={13} />
							{:else if query.status === 'skipped'}<MinusIcon size={13} />
							{:else}<CircleIcon size={9} />{/if}
						</span>
						<span class="text">{query.query_text}</span>
						{#if query.language}<span class="language">{query.language}</span>{/if}
						<span class="outcome">
							{#if query.status === 'completed'}{t('stages.found', { count: query.result_count })}
							{:else if query.status === 'failed'}<span use:tooltip={query.error ?? ''}>{t('stages.queryFailed')}</span>
							{:else if query.status === 'skipped'}{t('stages.querySkipped')}
							{:else if query.status === 'running'}{t('stages.queryRunning')}
							{:else}{t('stages.queryWaiting')}{/if}
						</span>
					</li>
				{/each}
			</ul>
		{/if}
	{/if}
</section>

<style>
	.run-stages {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 16px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius-lg);
		background: var(--app-panel);
		min-height: 0;
		overflow: auto;
	}
	.run-stages.compact {
		gap: 6px;
		padding: 0;
		border: 0;
		background: none;
		overflow: visible;
	}
	.ladder {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.compact .ladder {
		flex-direction: row;
		flex-wrap: wrap;
		gap: 4px 14px;
		font-size: var(--app-text-sm);
	}
	.stage {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--app-muted);
	}
	.stage .icon,
	.query .icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 16px;
		flex-shrink: 0;
	}
	.stage.done .icon {
		color: var(--app-success);
	}
	.stage.active {
		color: var(--app-text);
		font-weight: 600;
	}
	.stage.active .icon {
		color: var(--app-accent);
	}
	.detail {
		color: var(--app-muted);
		font-weight: 400;
		font-size: var(--app-text-sm);
		font-variant-numeric: tabular-nums;
	}
	.run-stages :global(.spin) {
		animation: spin 1s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	.list-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		width: fit-content;
		padding: 0;
		border: 0;
		background: none;
		color: var(--app-text);
		font-size: var(--app-text-md);
		font-weight: 600;
	}
	.compact .list-toggle {
		font-size: var(--app-text-sm);
	}
	.summary {
		color: var(--app-muted);
		font-weight: 400;
	}
	.queries {
		display: flex;
		flex-direction: column;
		max-height: 320px;
		overflow: auto;
		border-top: 1px solid var(--app-border);
	}
	.query {
		display: grid;
		grid-template-columns: 16px minmax(0, 1fr) auto auto;
		align-items: center;
		gap: 8px;
		padding: 5px 2px;
		border-bottom: 1px solid var(--app-border);
		font-size: var(--app-text-sm);
	}
	.query .text {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.query.pending,
	.query.skipped {
		color: var(--app-muted);
	}
	.query.completed .icon {
		color: var(--app-success);
	}
	.query.running .icon {
		color: var(--app-accent);
	}
	.query.failed .icon,
	.query.failed .outcome {
		color: var(--app-danger);
	}
	.language {
		padding: 0 6px;
		border-radius: var(--app-radius-pill);
		background: var(--app-subtle);
		color: var(--app-muted);
		font-size: var(--app-text-xs);
		text-transform: uppercase;
	}
	.outcome {
		color: var(--app-muted);
		white-space: nowrap;
	}
	@media (prefers-reduced-motion: reduce) {
		.run-stages :global(.spin) {
			animation: none;
		}
	}
</style>
