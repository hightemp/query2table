<script lang="ts">
	import type { Accounting, ProgressStats } from '$lib/types';
	import type { StopConditions } from '$lib/api/tauri';
	import { formatUsd, formatMinutes } from '$lib/utils/stopConditions';
	let {
		stats,
		limits,
		accounting = null,
		runType = 'table',
		paused = false,
	}: {
		stats: ProgressStats | null;
		limits: StopConditions | null;
		accounting?: Accounting | null;
		runType?: string;
		paused?: boolean;
	} = $props();
	const units: Record<string, string> = {
		table: 'rows',
		images: 'images',
		links: 'links',
		research: 'steps',
	};

	// The run stops at whichever limit it reaches first, so the bar follows the nearest one.
	let nearest = $derived.by(() => {
		const candidates: { fraction: number; name: string; detail: string }[] = [];
		const target = limits?.target_row_count;
		if (target) {
			const done = runType === 'research' ? (stats?.queries_executed ?? 0) : (stats?.rows_found ?? 0);
			candidates.push({
				fraction: done / target,
				name: `${units[runType] ?? 'results'} target`,
				detail: `${done} of ${target} ${units[runType] ?? 'results'}`,
			});
		}
		const budget = accounting?.max_budget_usd ?? limits?.max_budget_usd;
		if (budget)
			candidates.push({
				fraction: (accounting?.spent_usd ?? 0) / budget,
				name: 'spending limit',
				detail: `${formatUsd(accounting?.spent_usd ?? 0)} of ${formatUsd(budget)}`,
			});
		const duration = limits?.max_duration_seconds;
		if (duration)
			candidates.push({
				fraction: (stats?.elapsed_secs ?? 0) / duration,
				name: 'time limit',
				detail: `${Math.floor((stats?.elapsed_secs ?? 0) / 60)} of ${formatMinutes(duration)}`,
			});
		return candidates.sort((a, b) => b.fraction - a.fraction)[0] ?? null;
	});
	let percent = $derived(nearest ? Math.min(100, Math.round(nearest.fraction * 100)) : 0);
</script>

{#if nearest}
	<div class="run-progress" class:paused>
		<div
			class="track"
			role="progressbar"
			aria-label="Run progress"
			aria-valuemin={0}
			aria-valuemax={100}
			aria-valuenow={percent}
			aria-valuetext={`${percent}% of the ${nearest.name}: ${nearest.detail}`}
		>
			<div class="fill" style={`width:${percent}%`}></div>
		</div>
		<span class="caption">{percent}% of the {nearest.name} · {nearest.detail}</span>
	</div>
{/if}

<style>
	.run-progress {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
	}
	.track {
		flex: 1;
		height: 4px;
		border-radius: var(--app-radius-pill);
		background: var(--app-subtle);
		overflow: hidden;
	}
	.fill {
		height: 100%;
		border-radius: inherit;
		background: var(--app-accent);
		transition: width 0.4s ease;
	}
	.paused .fill {
		background: var(--app-warning);
	}
	.caption {
		flex-shrink: 0;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		font-variant-numeric: tabular-nums;
	}
</style>
