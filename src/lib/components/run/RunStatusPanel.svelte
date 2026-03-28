<script lang="ts">
	import { logs } from '$lib/stores/logs';
	import { LoaderCircleIcon, CheckCircle2Icon, AlertCircleIcon } from '@lucide/svelte';

	interface Props {
		status: string;
	}

	let { status }: Props = $props();

	// Pipeline phases in order
	const phases = [
		{ key: 'interpreter', label: 'Analyzing query' },
		{ key: 'planner', label: 'Planning schema' },
		{ key: 'schema_review', label: 'Waiting for schema confirmation' },
		{ key: 'search_planner', label: 'Planning search queries' },
		{ key: 'search_executor', label: 'Searching the web' },
		{ key: 'fetcher', label: 'Fetching pages' },
		{ key: 'extractor', label: 'Extracting data' },
		{ key: 'deduplicator', label: 'Deduplicating results' },
	];

	// Derive the active phase from the latest logs
	let activePhaseKey = $derived.by(() => {
		const items = $logs;
		// Walk logs backwards to find the last role that matches a known phase
		for (let i = items.length - 1; i >= 0; i--) {
			const msg = items[i].message;
			for (const phase of phases) {
				if (msg.includes(`[${phase.key}]`)) {
					return phase.key;
				}
			}
		}
		if (status === 'schema_review') return 'schema_review';
		if (status === 'pending') return null;
		return null;
	});

	// Determine phase status: completed, active, or pending
	function phaseStatus(key: string): 'completed' | 'active' | 'pending' {
		if (!activePhaseKey) return key === phases[0].key && status === 'pending' ? 'active' : 'pending';
		const activeIdx = phases.findIndex((p) => p.key === activePhaseKey);
		const thisIdx = phases.findIndex((p) => p.key === key);
		if (thisIdx < activeIdx) return 'completed';
		if (thisIdx === activeIdx) {
			if (status === 'completed' || status === 'failed' || status === 'cancelled') return 'completed';
			return 'active';
		}
		return 'pending';
	}

	// Get recent log messages for the active phase
	let recentActivity = $derived.by(() => {
		const items = $logs;
		const recent: string[] = [];
		for (let i = items.length - 1; i >= 0 && recent.length < 3; i--) {
			if (items[i].level === 'INFO' || items[i].level === 'WARN') {
				recent.unshift(items[i].message);
			}
		}
		return recent;
	});

	let isTerminal = $derived(status === 'completed' || status === 'failed' || status === 'cancelled');
</script>

<div class="status-panel">
	<div class="phase-list">
		{#each phases as phase}
			{@const ps = phaseStatus(phase.key)}
			<div class="phase-item" class:completed={ps === 'completed'} class:active={ps === 'active'} class:pending={ps === 'pending'}>
				<div class="phase-icon">
					{#if ps === 'completed'}
						<CheckCircle2Icon size={18} />
					{:else if ps === 'active'}
						<div class="spinner">
							<LoaderCircleIcon size={18} />
						</div>
					{:else}
						<div class="dot"></div>
					{/if}
				</div>
				<span class="phase-label">{phase.label}</span>
			</div>
		{/each}
	</div>

	{#if recentActivity.length > 0 && !isTerminal}
		<div class="activity-feed">
			{#each recentActivity as msg, i}
				<div class="activity-line" class:latest={i === recentActivity.length - 1}>
					{msg}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.status-panel {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 16px;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 10px;
		background: var(--color-surface-50-950);
		margin-bottom: 12px;
	}

	.phase-list {
		display: flex;
		flex-wrap: wrap;
		gap: 6px 16px;
	}

	.phase-item {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 0.85rem;
		color: var(--color-surface-400-600);
		transition: color 0.3s;
	}

	.phase-item.completed {
		color: var(--color-success-500, #22c55e);
	}

	.phase-item.active {
		color: var(--color-primary-500);
		font-weight: 600;
	}

	.phase-icon {
		display: flex;
		align-items: center;
		flex-shrink: 0;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--color-surface-300-700);
		margin: 5px;
	}

	.spinner {
		animation: spin 1.2s linear infinite;
		display: flex;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.activity-feed {
		display: flex;
		flex-direction: column;
		gap: 2px;
		font-size: 0.8rem;
		font-family: monospace;
		color: var(--color-surface-600-400);
		max-height: 72px;
		overflow: hidden;
		border-top: 1px solid var(--color-surface-200-800);
		padding-top: 8px;
	}

	.activity-line {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		opacity: 0.5;
	}

	.activity-line.latest {
		opacity: 1;
		color: var(--color-surface-900-100);
		animation: fadeIn 0.3s ease;
	}

	@keyframes fadeIn {
		from { opacity: 0; transform: translateY(4px); }
		to { opacity: 1; transform: translateY(0); }
	}
</style>
