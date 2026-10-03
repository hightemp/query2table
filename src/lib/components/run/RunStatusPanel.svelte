<script lang="ts">
	import type { LogEntryEvent } from '$lib/types';
	let {
		status,
		runType = 'table',
		activity = [],
	}: { status: string; runType?: string; activity?: LogEntryEvent[] } = $props();
	const common: Record<string, string> = {
		search_executor: 'Searching the web',
		fetcher: 'Fetching pages',
		stopping_controller: 'Finishing the run',
	};
	const operations: Record<string, Record<string, string>> = {
		table: {
			...common,
			interpreter: 'Analyzing query',
			planner: 'Planning schema',
			schema_planner: 'Planning schema',
			search_planner: 'Planning searches',
			query_expander: 'Expanding searches',
			extractor: 'Extracting data',
			deduplicator: 'Deduplicating results',
			validator: 'Validating results',
		},
		images: {
			image_pipeline: 'Preparing image search',
			image_searcher: 'Searching images',
			image_search_planner: 'Planning image searches',
			image_ranker: 'Ranking images',
			image_storage: 'Storing images',
			stopping_controller: 'Finishing the run',
		},
		links: {
			...common,
			link_pipeline: 'Preparing link search',
			link_search_planner: 'Planning link searches',
			link_ranker: 'Ranking links',
			link_storage: 'Storing links',
		},
		research: { research: 'Researching sources', ...common },
	};
	let latest = $derived([...activity].reverse().find((entry) => operations[runType]?.[entry.role]));
	let label = $derived(
		status === 'schema_review'
			? 'Waiting for schema confirmation'
			: status === 'paused'
				? 'Paused — resume when ready'
				: status === 'pending'
					? 'Preparing your run'
					: latest
						? operations[runType][latest.role]
						: 'Waiting for activity'
	);
	let running = $derived(status === 'running' || status === 'pending');
</script>

<div class="current-operation">
	<span class="activity-dot" class:running></span><span role="status">{label}</span>
	{#if activity.length}<details>
			<summary>Activity</summary>
			<div class="activity-popover">
				{#each activity.slice(-8) as entry}<p>{entry.message}</p>{/each}
			</div>
		</details>{/if}
</div>

<style>
	.current-operation {
		display: flex;
		gap: 8px;
		align-items: center;
		color: var(--app-muted);
		font-size: var(--app-text-md);
		min-width: 0;
		flex-wrap: wrap;
	}
	.activity-dot {
		width: 7px;
		height: 7px;
		background: var(--app-muted);
		border-radius: 50%;
		flex-shrink: 0;
	}
	.running {
		background: var(--app-accent);
		animation: pulse 1.6s ease-in-out infinite;
	}
	details {
		position: relative;
	}
	summary {
		cursor: pointer;
		font-size: var(--app-text-sm);
	}
	.activity-popover {
		position: absolute;
		top: calc(100% + 6px);
		left: 0;
		z-index: 20;
		width: min(560px, 70vw);
		max-height: 220px;
		overflow: auto;
		padding: 8px 12px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		background: var(--app-panel);
		box-shadow: var(--app-shadow-popover);
		color: var(--app-text);
		font-size: var(--app-text-sm);
		overflow-wrap: anywhere;
	}
	p {
		margin: 4px 0;
	}
	@keyframes pulse {
		50% {
			opacity: 0.35;
		}
	}
</style>
