<script lang="ts">
	import { t, type MessageKey } from '$lib/i18n';
	import Select from '$lib/components/common/Select.svelte';
	import { onDestroy, onMount, tick } from 'svelte';
	import { get } from 'svelte/store';
	import { goto } from '$app/navigation';
	import { SearchIcon, XIcon, TrashIcon, DownloadIcon, PlusIcon } from '@lucide/svelte';
	import type { HistoryRun, HistorySort } from '$lib/types';
	import {
		deleteWithUndo,
		history,
		loadHistory,
		loadMoreHistory,
		rememberHistoryScroll,
		renameHistoryRun,
		setHistoryFilter,
		togglePin,
	} from '$lib/stores/history';
	import { runInProgress, runState } from '$lib/stores/run';
	import { showContextMenu } from '$lib/stores/contextMenu';
	import { toast } from '$lib/stores/toasts';
	import HistoryRow from '$lib/components/history/HistoryRow.svelte';
	import BulkExportDialog from '$lib/components/history/BulkExportDialog.svelte';
	import ExportDialog from '$lib/components/run/ExportDialog.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import EmptyState from '$lib/components/common/EmptyState.svelte';
	import { groupRuns } from '$lib/utils/history';
	import { runMenuItems } from '$lib/utils/runActions';
	import { errorText, presentError } from '$lib/utils/errors';
	import { hasMod } from '$lib/utils/shortcuts';

	const TYPES: { value: string; label: MessageKey }[] = [
		{ value: 'all', label: 'history.all' },
		{ value: 'table', label: 'mode.table' },
		{ value: 'images', label: 'mode.images' },
		{ value: 'links', label: 'mode.links' },
		{ value: 'research', label: 'mode.research' },
	];

	let list = $state<HTMLDivElement>();
	let searchInput = $state<HTMLInputElement>();
	let search = $state(get(history).filter.search);
	let now = $state(Date.now());
	let selected = $state<string[]>([]);
	let lastSelected: string | null = null;
	let renamingId = $state<string | null>(null);
	let bulkExport = $state<string[] | null>(null);
	let exportRun = $state<HistoryRun | null>(null);

	let groups = $derived(groupRuns($history.runs, now, $history.filter.sort));
	let ordered = $derived(groups.flatMap((group) => group.runs));
	let total = $derived(Object.values($history.counts).reduce((sum, n) => sum + n, 0));
	let filtered = $derived(
		!!$history.filter.search.trim() || $history.filter.runType !== 'all' || $history.filter.status !== 'any'
	);

	onMount(() => {
		const clock = setInterval(() => (now = Date.now()), 60_000);
		const state = get(history);
		if (state.loaded) {
			// Coming back from a run: show the list where it was, then refresh it in place.
			void tick().then(async () => {
				if (list) list.scrollTop = state.scrollTop;
				if (state.runs.length > 50) return; // reloading would drop the later pages
				await loadHistory();
				await tick();
				if (list) list.scrollTop = state.scrollTop;
			});
		} else void loadHistory();
		return () => clearInterval(clock);
	});
	onDestroy(() => clearTimeout(searchTimer));

	let searchTimer: ReturnType<typeof setTimeout> | undefined;
	function onSearch(value: string) {
		search = value;
		clearTimeout(searchTimer);
		searchTimer = setTimeout(() => void setHistoryFilter({ search: value }), 250);
	}
	function clearFilters() {
		search = '';
		clearTimeout(searchTimer);
		void setHistoryFilter({ search: '', runType: 'all', status: 'any' });
	}

	function onScroll() {
		if (!list) return;
		rememberHistoryScroll(list.scrollTop);
		if (list.scrollTop + list.clientHeight >= list.scrollHeight - 300) void loadMoreHistory();
	}

	function select(run: HistoryRun, checked: boolean, range: boolean) {
		if (range && lastSelected) {
			const from = ordered.findIndex((item) => item.id === lastSelected);
			const to = ordered.findIndex((item) => item.id === run.id);
			if (from >= 0 && to >= 0) {
				const ids = ordered.slice(Math.min(from, to), Math.max(from, to) + 1).map((item) => item.id);
				selected = checked ? [...new Set([...selected, ...ids])] : selected.filter((id) => !ids.includes(id));
				lastSelected = run.id;
				return;
			}
		}
		selected = checked ? [...selected, run.id] : selected.filter((id) => id !== run.id);
		lastSelected = run.id;
	}

	/** The run being worked on opens live on the query page. */
	function open(run: HistoryRun, event: MouseEvent) {
		if (list) rememberHistoryScroll(list.scrollTop);
		if (get(runState).runId === run.id && runInProgress()) {
			event.preventDefault();
			void goto('/');
		}
	}

	async function remove(ids: string[]) {
		try {
			await deleteWithUndo(ids);
			selected = selected.filter((id) => !ids.includes(id));
		} catch (error) {
			const explained = presentError(error, 'history');
			// Messages the app wrote itself are clearer than the generic explanation.
			toast(
				explained.generic
					? errorText(error)
					: t('history.deleteFailed', { reason: explained.title.toLowerCase() }),
				'error'
			);
		}
	}

	async function rename(run: HistoryRun, title: string | null) {
		renamingId = null;
		if (title === null || title.trim() === (run.title || run.query)) return;
		try {
			await renameHistoryRun(run.id, title);
		} catch (error) {
			toast(errorText(error), 'error');
		}
	}

	function menu(run: HistoryRun, point: { x: number; y: number }) {
		showContextMenu(point, t('common.actionsFor', { name: run.title || run.query }), runMenuItems(run, {
			open: () => void goto(`/history/${encodeURIComponent(run.id)}`),
			exportRun: () => (exportRun = run),
			rename: () => (renamingId = run.id),
			togglePin: async () => {
				try {
					await togglePin(run);
				} catch (error) {
					toast(errorText(error), 'error');
				}
			},
			remove: () => void remove([run.id]),
		}));
	}

	function onKeydown(event: KeyboardEvent) {
		if (hasMod(event) && event.key.toLowerCase() === 'f') {
			event.preventDefault();
			searchInput?.focus();
		}
	}
</script>

<svelte:window onkeydown={onKeydown} />

<div class="history-page">
	<header class="page-header">
		<div>
			<h1>{t('history.title')}</h1>
			<p>{t('history.subtitle')}</p>
		</div>
	</header>

	<div class="toolbar">
		<label class="search"
			><SearchIcon size={15} /><input
				bind:this={searchInput}
				type="search"
				class="input sm"
				placeholder={t('history.searchPlaceholder')}
				aria-label={t('history.search')}
				value={search}
				oninput={(event) => onSearch(event.currentTarget.value)}
			/></label
		>
		<div class="types" role="group" aria-label={t('history.runType')}>
			{#each TYPES as type (type.value)}
				{@const count = type.value === 'all' ? total : ($history.counts[type.value] ?? 0)}
				<button
					class="chip"
					aria-pressed={$history.filter.runType === type.value}
					onclick={() => void setHistoryFilter({ runType: type.value })}
					>{t(type.label)} <span class="count">{count}</span></button
				>
			{/each}
		</div>
		<Select
			size="sm"
			label={t('history.status')}
			value={$history.filter.status}
			onchange={(status) => void setHistoryFilter({ status })}
			options={[
				{ value: 'any', label: t('history.anyStatus') },
				{ value: 'completed', label: t('status.completed') },
				{ value: 'failed', label: t('status.failed') },
				{ value: 'cancelled', label: t('status.cancelled') },
				{ value: 'active', label: t('history.inProgress') },
			]}
		/>
		<Select
			size="sm"
			label={t('history.sort')}
			value={$history.filter.sort}
			onchange={(sort) => void setHistoryFilter({ sort: sort as HistorySort })}
			options={[
				{ value: 'newest', label: t('history.newest') },
				{ value: 'oldest', label: t('history.oldest') },
				{ value: 'results', label: t('history.mostResults') },
				{ value: 'cost', label: t('history.highestCost') },
			]}
		/>

	</div>

	{#if selected.length}
		<div class="bulk" role="toolbar" aria-label={t('history.selectedRuns')}>
			<strong>{t('common.selectedCount', { count: selected.length })}</strong>
			<button class="button sm" onclick={() => (bulkExport = ordered.filter((r) => selected.includes(r.id)).map((r) => r.id))}
				><DownloadIcon size={14} />{t('history.export')}</button
			>
			<button class="button sm danger" onclick={() => void remove([...selected])}
				><TrashIcon size={14} />{t('history.delete')}</button
			>
			<button class="button sm ghost" onclick={() => (selected = [])}
				><XIcon size={14} />{t('common.clearSelection')}</button
			>
		</div>
	{/if}

	{#if $history.error}<ErrorNotice error={$history.error} context="history" />{/if}

	<div class="history-list" class:selecting={selected.length > 0} bind:this={list} onscroll={onScroll}>
		{#if !$history.loaded}
			<EmptyState role="status">{t('history.loading')}</EmptyState>
		{:else if !$history.runs.length}
			{#if filtered}
				<EmptyState
					>{t('history.noMatch')}
					<button class="button sm" onclick={clearFilters}>{t('history.clearFilters')}</button></EmptyState
				>
			{:else}
				<EmptyState
					>{t('history.empty')}
					<a class="button sm accent" href="/"><PlusIcon size={14} />{t('history.newQuery')}</a></EmptyState
				>
			{/if}
		{:else}
			{#each groups as group, i (group.label ?? i)}
				<section class="history-group" aria-label={group.label ?? t('history.runs')}>
					{#if group.label}<h2>{group.label}</h2>{/if}
					<ul>
						{#each group.runs as run (run.id)}
							<HistoryRow
								{run}
								{now}
								selected={selected.includes(run.id)}
								renaming={renamingId === run.id}
								onselect={(checked, range) => select(run, checked, range)}
								onopen={(event) => open(run, event)}
								onmenu={(point) => menu(run, point)}
								onrename={(title) => void rename(run, title)}
							/>
						{/each}
					</ul>
				</section>
			{/each}
			{#if $history.loading && $history.runs.length}<p class="more" role="status">{t('history.loadingMore')}</p>{/if}
		{/if}
	</div>
</div>

{#if bulkExport}
	<BulkExportDialog runIds={bulkExport} onclose={() => (bulkExport = null)} />
{/if}
{#if exportRun}
	<ExportDialog runId={exportRun.id} runType={exportRun.run_type} onclose={() => (exportRun = null)} />
{/if}

<style>
	.history-page {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
	}
	.toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 12px;
		flex-shrink: 0;
		padding-bottom: 12px;
	}
	.search {
		display: flex;
		align-items: center;
		gap: 6px;
		flex: 1 1 220px;
		max-width: 360px;
		color: var(--app-muted);
	}
	.search input {
		flex: 1;
		min-width: 0;
	}
	.toolbar :global(.select) {
		width: auto;
		flex: 0 0 auto;
	}
	.types {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 3px 10px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius-pill);
		font-size: var(--app-text-md);
	}
	.chip[aria-pressed='true'] {
		border-color: var(--app-accent);
		color: var(--app-accent);
		background: color-mix(in srgb, var(--app-accent) 10%, transparent);
	}
	.count {
		color: var(--app-muted);
		font-size: var(--app-text-xs);
		font-variant-numeric: tabular-nums;
	}
	.bulk {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
		margin-bottom: 8px;
		padding: 6px 10px;
		border-radius: var(--app-radius);
		background: color-mix(in srgb, var(--app-accent) 8%, var(--app-panel));
	}
	.bulk strong {
		margin-right: 4px;
	}
	.history-list {
		flex: 1;
		min-height: 0;
		overflow: auto;
		scrollbar-gutter: stable;
		padding-right: 12px;
	}
	.history-group h2 {
		position: sticky;
		top: 0;
		z-index: 2;
		margin: 0;
		padding: 10px 10px 4px;
		background: var(--app-bg);
		color: var(--app-muted);
		font-size: var(--app-text-xs);
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}
	ul {
		list-style: none;
		margin: 0 0 8px;
		padding: 0;
	}
	.more {
		padding: 12px;
		color: var(--app-muted);
		text-align: center;
	}
</style>
