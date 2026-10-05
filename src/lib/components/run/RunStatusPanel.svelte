<script lang="ts">
	import { t, type MessageKey } from '$lib/i18n';
	import type { LogEntryEvent } from '$lib/types';
	let {
		status,
		runType = 'table',
		activity = [],
	}: { status: string; runType?: string; activity?: LogEntryEvent[] } = $props();
	const common = ['search_executor', 'fetcher', 'stopping_controller'];
	/** Log roles that name the current operation in each mode. */
	const operations: Record<string, string[]> = {
		table: [...common, 'interpreter', 'planner', 'schema_planner', 'search_planner', 'query_expander', 'extractor', 'deduplicator', 'validator'],
		images: ['image_pipeline', 'image_searcher', 'image_search_planner', 'image_ranker', 'image_storage', 'stopping_controller'],
		links: [...common, 'link_pipeline', 'link_search_planner', 'link_ranker', 'link_storage'],
		research: ['research', ...common],
	};
	let latest = $derived([...activity].reverse().find((entry) => operations[runType]?.includes(entry.role)));
	let label = $derived(
		status === 'schema_review'
			? t('operation.waitingSchema')
			: status === 'paused'
				? t('operation.paused')
				: status === 'pending'
					? t('operation.preparing')
					: latest
						? t(`operation.${latest.role === 'schema_planner' ? 'planner' : latest.role}` as MessageKey)
						: t('operation.waiting')
	);
	let running = $derived(status === 'running' || status === 'pending');
</script>

<div class="current-operation">
	<span class="activity-dot" class:running></span><span role="status">{label}</span>
	{#if activity.length}<details>
			<summary>{t('operation.activity')}</summary>
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
