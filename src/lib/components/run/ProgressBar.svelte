<script lang="ts">
	import type { ProgressStats } from '$lib/types';

	interface Props {
		stats: ProgressStats | null;
		status: string;
	}

	let { stats, status }: Props = $props();

	let progressPercent = $derived(() => {
		if (!stats) return 0;
		if (stats.pages_total === 0) return 0;
		return Math.round((stats.pages_fetched / stats.pages_total) * 100);
	});

	let isRunning = $derived(status === 'running' || status === 'pending');

	function formatElapsed(secs: number): string {
		const m = Math.floor(secs / 60);
		const s = secs % 60;
		return m > 0 ? `${m}m ${s}s` : `${s}s`;
	}
</script>

<div class="progress-bar-container">
	<div class="progress-header">
		<span class="progress-label">
			{#if status === 'running'}
				Processing...
			{:else if status === 'pending'}
				Starting...
			{:else if status === 'paused'}
				Paused
			{:else if status === 'completed'}
				Completed
			{:else if status === 'failed'}
				Failed
			{:else if status === 'cancelled'}
				Cancelled
			{:else}
				{status}
			{/if}
		</span>
		{#if stats}
			<span class="progress-pct">{progressPercent()}%</span>
		{/if}
	</div>

	<div class="progress-track">
		<div
			class="progress-fill"
			class:running={isRunning}
			class:completed={status === 'completed'}
			class:failed={status === 'failed'}
			class:paused={status === 'paused'}
			style="width: {stats ? progressPercent() : (isRunning ? 100 : 0)}%"
		></div>
	</div>

	{#if stats}
		<div class="progress-stats">
			<span>Rows: <strong>{stats.rows_found}</strong></span>
			<span>Pages: <strong>{stats.pages_fetched}</strong>/{stats.pages_total}</span>
			<span>Queries: <strong>{stats.queries_executed}</strong>/{stats.queries_total}</span>
			<span>Time: {formatElapsed(stats.elapsed_secs)}</span>
			<span>Cost: ${stats.spent_usd.toFixed(4)}</span>
		</div>
	{/if}
</div>

<style>
	.progress-bar-container {
		padding: 12px 0;
	}

	.progress-header {
		display: flex;
		justify-content: space-between;
		margin-bottom: 6px;
		font-size: 0.9rem;
	}

	.progress-label {
		font-weight: 600;
	}

	.progress-pct {
		color: var(--color-surface-600-400);
	}

	.progress-track {
		height: 8px;
		background: var(--color-surface-200-800);
		border-radius: 4px;
		overflow: hidden;
		position: relative;
	}

	.progress-fill {
		height: 100%;
		background: var(--color-primary-500);
		border-radius: 4px;
		transition: width 0.3s ease;
	}

	.progress-fill.running {
		animation: pulse 1.5s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.6; }
	}

	.progress-fill.completed {
		background: var(--color-success-500, #22c55e);
		animation: none;
	}

	.progress-fill.failed {
		background: var(--color-error-500);
		animation: none;
	}

	.progress-fill.paused {
		background: var(--color-warning-500);
		animation: none;
	}

	.progress-stats {
		display: flex;
		gap: 16px;
		margin-top: 8px;
		font-size: 0.8rem;
		color: var(--color-surface-600-400);
		flex-wrap: wrap;
	}

	.progress-stats strong {
		color: var(--color-surface-900-100);
	}
</style>
