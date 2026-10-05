<script lang="ts">
	import { t, type MessageKey } from '$lib/i18n';
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
	const units = { table: 'units.row', images: 'units.image', links: 'units.link', research: 'units.step' } as const;

	// The run stops at whichever limit it reaches first, so the bar follows the nearest one.
	let nearest = $derived.by(() => {
		const candidates: { fraction: number; name: string; detail: string }[] = [];
		const target = limits?.target_row_count;
		if (target) {
			const done = runType === 'research' ? (stats?.queries_executed ?? 0) : (stats?.rows_found ?? 0);
			candidates.push({
				fraction: done / target,
				name: t(`runProgress.target.${runType in units ? runType : 'table'}` as MessageKey),
				detail: t('runProgress.targetDetail', {
					done,
					target: t(units[runType as keyof typeof units] ?? 'units.row', { count: target }),
				}),
			});
		}
		const budget = accounting?.max_budget_usd ?? limits?.max_budget_usd;
		if (budget)
			candidates.push({
				fraction: (accounting?.spent_usd ?? 0) / budget,
				name: t('runProgress.spending'),
				detail: t('runProgress.targetDetail', { done: formatUsd(accounting?.spent_usd ?? 0), target: formatUsd(budget) }),
			});
		const duration = limits?.max_duration_seconds;
		if (duration)
			candidates.push({
				fraction: (stats?.elapsed_secs ?? 0) / duration,
				name: t('runProgress.time'),
				detail: t('runProgress.timeDetail', { done: Math.floor((stats?.elapsed_secs ?? 0) / 60), total: formatMinutes(duration) }),
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
			aria-label={t('runProgress.label')}
			aria-valuemin={0}
			aria-valuemax={100}
			aria-valuenow={percent}
			aria-valuetext={`${t('runProgress.of', { percent, limit: nearest.name })}: ${nearest.detail}`}
		>
			<div class="fill" style={`width:${percent}%`}></div>
		</div>
		<span class="caption">{t('runProgress.of', { percent, limit: nearest.name })} · {nearest.detail}</span>
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
