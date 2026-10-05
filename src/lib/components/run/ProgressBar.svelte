<script lang="ts">
	import { t, type MessageKey } from '$lib/i18n';
	import type { ProgressStats } from '$lib/types';
	let {
		stats,
		status,
		runType = 'table',
		target = null,
	}: {
		stats: ProgressStats | null;
		status: string;
		runType?: string;
		target?: number | null;
	} = $props();
	const units: Record<string, MessageKey> = {
		table: 'progress.rows',
		images: 'progress.images',
		links: 'progress.links',
		research: 'progress.searches',
	};
	function elapsed(seconds: number) {
		const secs = Math.floor(seconds);
		return secs >= 60
			? t('duration.minutesSeconds', { m: Math.floor(secs / 60), s: secs % 60 })
			: t('duration.seconds', { s: secs });
	}
</script>

<div class="progress-stats" aria-label={t('progress.label')}>
	{#if stats}
		<span
			>{t(units[runType] ?? 'progress.results')} <strong>{stats.rows_found}</strong>{#if target && runType !== 'research'}{` / ${target}`}{/if}</span
		>
		{#if runType !== 'images'}<span
				>{t('progress.pages')} <strong>{stats.pages_fetched}</strong>{#if stats.pages_total > 0}{` / ${stats.pages_total}`}{/if}</span
			>{/if}
		<span
			>{runType === 'research' ? t('progress.steps') : t('progress.queries')}
			<strong>{stats.queries_executed}</strong>{#if stats.queries_total > 0}{` / ${stats.queries_total}`}{/if}</span
		>
		<span>{elapsed(stats.elapsed_secs)}</span>
	{:else if ['pending', 'running'].includes(status)}<span>{t('progress.waiting')}</span>{/if}
</div>

<style>
	.progress-stats {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 16px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	strong {
		color: var(--app-text);
		font-variant-numeric: tabular-nums;
	}
</style>
