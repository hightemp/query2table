<script lang="ts">
	import type { ProgressStats } from '$lib/types';
	let {
		stats,
		status,
		runType = 'table',
	}: { stats: ProgressStats | null; status: string; runType?: string } = $props();
	const units: Record<string, string> = {
		table: 'Rows',
		images: 'Images',
		links: 'Links',
		research: 'Searches',
	};
	function elapsed(seconds: number) {
		const secs = Math.floor(seconds);
		return secs >= 60 ? `${Math.floor(secs / 60)}m ${secs % 60}s` : `${secs}s`;
	}
</script>

<div class="progress-stats" aria-label="Run statistics">
	{#if stats}
		<span>{units[runType] ?? 'Results'} <strong>{stats.rows_found}</strong></span>
		<span
			>Pages <strong>{stats.pages_fetched}</strong>{#if stats.pages_total > 0}
				/ {stats.pages_total}{/if}</span
		>
		<span
			>{runType === 'research' ? 'Steps' : 'Queries'}
			<strong>{stats.queries_executed}</strong>{#if stats.queries_total > 0}
				/ {stats.queries_total}{/if}</span
		>
		<span>{elapsed(stats.elapsed_secs)}</span><span>${stats.spent_usd.toFixed(4)}</span>
	{:else if ['pending', 'running'].includes(status)}<span>Waiting for the first results…</span>{/if}
</div>

<style>
	.progress-stats {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 16px;
		color: var(--app-muted);
		font-size: 12px;
	}
	strong {
		color: var(--app-text);
		font-variant-numeric: tabular-nums;
	}
</style>
