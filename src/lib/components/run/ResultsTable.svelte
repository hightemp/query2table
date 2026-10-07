<script lang="ts">
	import { tooltip } from '$lib/actions/tooltip';
	import { t } from '$lib/i18n';
	import Select from '$lib/components/common/Select.svelte';
	import Checkbox from '$lib/components/common/Checkbox.svelte';

	import { untrack, tick } from 'svelte';
	import { get } from 'svelte/store';
	import {
		getCoreRowModel,
		getSortedRowModel,
		type ColumnDef,
		type SortingState,
	} from '@tanstack/table-core';
	import { createTable } from '@tanstack/svelte-table';
	import { createVirtualizer, defaultRangeExtractor } from '@tanstack/svelte-virtual';
	import {
		ArrowUpIcon,
		ArrowDownIcon,
		ArrowUpDownIcon,
		SearchIcon,
		PanelRightOpenIcon,
		Columns3Icon,
		Rows3Icon,
		CheckIcon,
		XIcon,
	} from '@lucide/svelte';
	import type { SchemaColumn } from '$lib/types';
	import type { RunRow } from '$lib/stores/run';
	import {
		formatValue,
		formatCell,
		isMissing,
		parseBoolean,
		sortValue,
		columnLabel,
		webUrl,
		urlLabel,
	} from '$lib/utils/values';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import EmptyState from '$lib/components/common/EmptyState.svelte';
	import { hasMod, modKey } from '$lib/utils/shortcuts';
	import { persisted } from '$lib/utils/storage';
	import { copyText } from '$lib/api/tauri';
	import { toast } from '$lib/stores/toasts';

	let {
		schema,
		rows,
		onrowclick,
	}: {
		schema: SchemaColumn[];
		rows: RunRow[];
		/** Receives the clicked row and the ids of all rows in their current displayed order. */
		onrowclick: (row: RunRow, order: string[]) => void;
	} = $props();

	const EVIDENCE = '__evidence';
	const ACTIONS_WIDTH = 44;
	const EVIDENCE_WIDTH = 116;
	const LOW_CONFIDENCE = 0.6;
	const density = persisted<'comfortable' | 'compact'>('q2t-table-density', 'comfortable', (v) =>
		v === 'compact' || v === 'comfortable' ? v : null
	);
	let rowHeight = $derived($density === 'compact' ? 36 : 64);

	let sorting = $state<SortingState>([]);
	let filterInput = $state('');
	let filter = $state('');
	let filterColumn = $state('');
	let hidden = $state<string[]>([]);
	let widths = $state<Record<string, number>>({});
	let columnsMenuOpen = $state(false);
	let scrollElement = $state<HTMLDivElement>();
	let searchInput = $state<HTMLInputElement>();
	let focusedRowId = $state<string | null>(null);

	// Debounce typing so large tables stay responsive.
	$effect(() => {
		const next = filterInput;
		const timer = setTimeout(() => (filter = next), 150);
		return () => clearTimeout(timer);
	});

	let visibleSchema = $derived(schema.filter((column) => !hidden.includes(column.name)));
	let typeOf = $derived(new Map(schema.map((column) => [column.name, column.type])));

	function defaultWidth(column: SchemaColumn, index: number) {
		if (column.type === 'number' || column.type === 'date') return 140;
		if (column.type === 'boolean') return 110;
		if (column.type === 'url') return 200;
		return index === 0 ? 240 : 220;
	}
	function widthOf(name: string) {
		const index = schema.findIndex((column) => column.name === name);
		return widths[name] ?? (index >= 0 ? defaultWidth(schema[index], index) : 220);
	}

	let filteredRows = $derived.by(() => {
		const needle = filter.trim().toLowerCase();
		if (!needle) return rows;
		const columns = filterColumn ? schema.filter((c) => c.name === filterColumn) : schema;
		return rows.filter((row) =>
			columns.some(
				(column) =>
					formatCell(row.data[column.name], column.type).toLowerCase().includes(needle) ||
					formatValue(row.data[column.name]).toLowerCase().includes(needle)
			)
		);
	});

	function compare(a: unknown, b: unknown) {
		if (typeof a === 'number' && typeof b === 'number') return a < b ? -1 : a > b ? 1 : 0;
		return String(a).localeCompare(String(b), undefined, { numeric: true });
	}

	let columnDefs = $derived.by<ColumnDef<RunRow, unknown>[]>(() => {
		const data: ColumnDef<RunRow, unknown>[] = visibleSchema.map((column) => ({
			id: column.name,
			accessorFn: (row) => sortValue(row.data[column.name], column.type) ?? undefined,
			header: column.name,
			sortUndefined: 'last',
			sortingFn: (a, b, id) => compare(a.getValue(id), b.getValue(id)),
		}));
		const evidence: ColumnDef<RunRow, unknown> = {
			id: EVIDENCE,
			accessorFn: (row) => row.confidence,
			header: t('table.evidence'),
			sortingFn: (a, b, id) => compare(a.getValue(id), b.getValue(id)),
		};
		return data.length ? [data[0], evidence, ...data.slice(1)] : [evidence];
	});

	const table = createTable({
		get data() {
			return filteredRows;
		},
		get columns() {
			return columnDefs;
		},
		getRowId: (row) => row.id,
		state: {
			get sorting() {
				return sorting;
			},
		},
		sortDescFirst: false,
		onSortingChange: (update) => {
			sorting = typeof update === 'function' ? update(sorting) : update;
		},
		getCoreRowModel: getCoreRowModel(),
		getSortedRowModel: getSortedRowModel(),
	});
	let visibleRows = $derived(table.getRowModel().rows);
	let headers = $derived(table.getHeaderGroups()[0]?.headers ?? []);
	let focusIndex = $derived(visibleRows.findIndex((row) => row.id === focusedRowId));
	let tableWidth = $derived(
		ACTIONS_WIDTH +
			headers.reduce((sum, h) => sum + (h.id === EVIDENCE ? EVIDENCE_WIDTH : widthOf(h.id)), 0)
	);
	let firstColumn = $derived(headers.find((h) => h.id !== EVIDENCE)?.id ?? null);

	const virtualizer = createVirtualizer<HTMLDivElement, HTMLTableRowElement>({
		count: 0,
		getScrollElement: () => null,
		estimateSize: () => 64,
		overscan: 8,
	});
	$effect(() => {
		const count = visibleRows.length;
		const element = scrollElement;
		const focused = focusIndex;
		const keys = visibleRows.map((row) => row.id);
		const size = rowHeight;
		untrack(() => {
			const instance = get(virtualizer);
			const sizeChanged = instance.options.estimateSize(0) !== size;
			instance.setOptions({
				...instance.options,
				count,
				getScrollElement: () => element ?? null,
				estimateSize: () => size,
				getItemKey: (index) => keys[index],
				rangeExtractor: (range) =>
					[...new Set([...defaultRangeExtractor(range), ...(focused >= 0 ? [focused] : [])])].sort(
						(a, b) => a - b
					),
			});
			if (sizeChanged) instance.measure();
		});
	});
	// Filtering updates the row model before the virtualizer effect runs. Drop obsolete indices during that transition.
	let items = $derived(
		$virtualizer.getVirtualItems().filter((item) => item.index < visibleRows.length)
	);

	function open(row: RunRow) {
		onrowclick(
			row,
			visibleRows.map((item) => item.id)
		);
	}

	function rowText(row: RunRow) {
		return visibleSchema.map((column) => formatValue(row.data[column.name], true)).join('\t');
	}

	async function handleRowKey(event: KeyboardEvent, index: number, row: RunRow) {
		if (hasMod(event) && event.key.toLowerCase() === 'c' && !window.getSelection()?.toString()) {
			event.preventDefault();
			try {
				await copyText(rowText(row));
				toast(t('table.rowCopied'), 'success');
			} catch {
				toast(t('table.rowCopyFailed'), 'error');
			}
			return;
		}
		if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return;
		event.preventDefault();
		const last = visibleRows.length - 1;
		const next =
			event.key === 'Home'
				? 0
				: event.key === 'End'
					? last
					: Math.max(0, Math.min(last, index + (event.key === 'ArrowDown' ? 1 : -1)));
		focusedRowId = visibleRows[next].id;
		$virtualizer.scrollToIndex(next, { align: 'auto' });
		await tick();
		scrollElement
			?.querySelector<HTMLButtonElement>(`button[data-row-index="${next}"]`)
			?.focus({ preventScroll: true });
	}

	function handleRowClick(event: MouseEvent, row: RunRow) {
		// Links and buttons inside the row keep their own behavior; selecting text does not open the row.
		if ((event.target as Element).closest('a, button, input')) return;
		if (window.getSelection()?.toString()) return;
		focusedRowId = row.id;
		open(row);
	}

	function startResize(event: PointerEvent, id: string) {
		event.preventDefault();
		event.stopPropagation();
		const handle = event.currentTarget as HTMLElement;
		handle.setPointerCapture(event.pointerId);
		const startX = event.clientX;
		const start = widthOf(id);
		const move = (moveEvent: PointerEvent) => {
			widths[id] = Math.min(800, Math.max(80, Math.round(start + moveEvent.clientX - startX)));
		};
		const stop = () => {
			handle.removeEventListener('pointermove', move);
			handle.removeEventListener('pointerup', stop);
			handle.removeEventListener('pointercancel', stop);
		};
		handle.addEventListener('pointermove', move);
		handle.addEventListener('pointerup', stop);
		handle.addEventListener('pointercancel', stop);
	}
	function resizeWithKeys(event: KeyboardEvent, id: string) {
		if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
		event.preventDefault();
		widths[id] = Math.min(800, Math.max(80, widthOf(id) + (event.key === 'ArrowRight' ? 20 : -20)));
	}

	function toggleColumn(name: string) {
		if (hidden.includes(name)) hidden = hidden.filter((item) => item !== name);
		// Keep at least one data column visible.
		else if (visibleSchema.length > 1) hidden = [...hidden, name];
	}

	function cellClass(type: string, value: unknown) {
		return [
			type === 'number' ? 'numeric' : '',
			isMissing(value) ? 'missing' : '',
		].join(' ');
	}
</script>

<svelte:window
	onkeydown={(event) => {
		// Native find does not search the virtualized rows, so the shortcut focuses the table search.
		if (hasMod(event) && !event.shiftKey && event.key.toLowerCase() === 'f' && searchInput) {
			event.preventDefault();
			searchInput.focus();
			searchInput.select();
		} else if (event.key === 'Escape' && columnsMenuOpen) {
			columnsMenuOpen = false;
		}
	}}
/>
<div class="results-table-wrap" class:compact={$density === 'compact'}>
	<div class="table-toolbar">
		<label class="search"
			><SearchIcon size={16} /><input
				class="input sm"
				type="search"
				aria-label={t('table.search')}
				placeholder={t('table.searchPlaceholder', { shortcut: `${modKey}+F` })}
				bind:this={searchInput}
				bind:value={filterInput}
			/></label
		>
		{#if schema.length > 1}<Select
				size="sm"
				class="column-filter"
				label={t('table.searchColumn')}
				bind:value={filterColumn}
				options={[
					{ value: '', label: t('table.allColumns') },
					...schema.map((column) => ({ value: column.name, label: columnLabel(column.name) })),
				]}
			/>{/if}

		<span class="count"
			>{filter
				? t('table.countOf', { shown: visibleRows.length, count: rows.length })
				: t('units.row', { count: rows.length })}</span
		>
		<div class="toolbar-actions">
			<button
				class="icon-button ghost sm"
				aria-pressed={$density === 'compact'}
				aria-label={t('table.compact')}
				use:tooltip={t('table.compact')}
				onclick={() =>
					density.update((value) => (value === 'compact' ? 'comfortable' : 'compact'))}
				><Rows3Icon size={16} /></button
			>
			{#if schema.length > 1}
				<div class="columns-menu">
					<button
						class="button ghost sm"
						aria-expanded={columnsMenuOpen}
						aria-controls="columns-menu"
						onclick={() => (columnsMenuOpen = !columnsMenuOpen)}
						><Columns3Icon size={16} />{t('table.columns')}{#if hidden.length}{` (${visibleSchema.length}/${schema.length})`}{/if}</button
					>
					{#if columnsMenuOpen}
						<div class="menu" id="columns-menu" role="group" aria-label={t('table.visibleColumns')}>
							{#each schema as column}
								<Checkbox
									checked={!hidden.includes(column.name)}
									disabled={!hidden.includes(column.name) && visibleSchema.length === 1}
									onchange={() => toggleColumn(column.name)}>{columnLabel(column.name)}</Checkbox
								>
							{/each}
							{#if hidden.length}<button class="button ghost sm" onclick={() => (hidden = [])}
									>{t('table.showAll')}</button
								>{/if}
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</div>
	<!-- Scroll region is keyboard-focusable to allow horizontal and vertical scrolling. -->
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<div
		class="table-scroll"
		bind:this={scrollElement}
		tabindex="0"
		role="region"
		aria-label={t('table.label')}
	>
		{#if !schema.length}<EmptyState>{t('table.schemaPending')}</EmptyState>
		{:else if !visibleRows.length}<EmptyState
				>{filter ? t('table.noMatch') : t('table.empty')}</EmptyState
			>
		{:else}
			<table style={`width: ${tableWidth}px`} aria-rowcount={visibleRows.length + 1}>
				<colgroup>
					<col style={`width:${ACTIONS_WIDTH}px`} />
					{#each headers as header (header.id)}<col
							style={`width:${header.id === EVIDENCE ? EVIDENCE_WIDTH : widthOf(header.id)}px`}
						/>{/each}
				</colgroup>
				<thead
					><tr
						><th class="sticky actions-cell"><span class="sr-only">{t('table.details')}</span></th
						>{#each headers as header (header.id)}
							{@const sorted = header.column.getIsSorted()}
							{@const column = schema.find((item) => item.name === header.id)}
							<th
								class:sticky={header.id === firstColumn}
								class:first-column={header.id === firstColumn}
								class:numeric={column?.type === 'number'}
								aria-sort={sorted === 'asc'
									? 'ascending'
									: sorted === 'desc'
										? 'descending'
										: 'none'}
								><button
									class:sorted={!!sorted}
									use:tooltip={header.id === EVIDENCE
										? t('table.evidenceHint')
										: column?.description || undefined}
									onclick={header.column.getToggleSortingHandler()}
									><span class="header-label"
										>{header.id === EVIDENCE ? t('table.evidence') : columnLabel(header.id)}</span
									>{#if sorted === 'asc'}<ArrowUpIcon size={14} />{:else if sorted === 'desc'}<ArrowDownIcon
											size={14}
										/>{:else}<ArrowUpDownIcon size={14} />{/if}</button
								>{#if header.id !== EVIDENCE}
									<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
									<span
										class="resize-handle"
										role="separator"
										aria-orientation="vertical"
										aria-label={t('table.resizeColumn', { name: columnLabel(header.id) })}
										aria-valuenow={widthOf(header.id)}
										tabindex="0"
										onpointerdown={(event) => startResize(event, header.id)}
										onkeydown={(event) => resizeWithKeys(event, header.id)}
										ondblclick={() => delete widths[header.id]}
									></span>{/if}</th
							>{/each}</tr
					></thead
				>
				<tbody>
					{#each items as item, index (item.key)}
						{@const row = visibleRows[item.index]}
						{@const gap = item.start - (items[index - 1]?.end ?? 0)}
						{@const low = row.original.confidence < LOW_CONFIDENCE}
						{#if gap > 0}<tr aria-hidden="true" class="spacer"
								><td colspan={headers.length + 1} style={`height:${gap}px`}></td></tr
							>{/if}
						<!-- Clicking anywhere on a row is a mouse shortcut; keyboard users use the details button. -->
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<tr
							class="data-row"
							class:selected={focusedRowId === row.id}
							class:low-confidence={low}
							aria-rowindex={item.index + 2}
							onclick={(event) => handleRowClick(event, row.original)}
						>
							<td class="sticky actions-cell"
								><button
									class="icon-button ghost sm"
									data-row-index={item.index}
									aria-label={t('table.openRow', { n: item.index + 1 })}
									use:tooltip={t('table.openRowHint', { shortcut: `${modKey}+C` })}
									onfocus={() => {
										focusedRowId = row.id;
									}}
									onkeydown={(event) => handleRowKey(event, item.index, row.original)}
									onclick={() => open(row.original)}><PanelRightOpenIcon size={16} /></button
								></td
							>
							{#each headers as header (header.id)}
								{#if header.id === EVIDENCE}
									{@const sources = row.original.sources}
									{@const percent = Math.round(row.original.confidence * 100)}
									<td class="evidence"
										><div class="confidence" use:tooltip={t('table.confidence', { percent })}>
											<span class="meter"><span style={`width:${percent}%`}></span></span>{percent}%
										</div>
										{#if sources !== undefined}<div class="sources">
												{t('units.source', { count: sources })}
											</div>{/if}</td
									>
								{:else}
									{@const value = row.original.data[header.id]}
									{@const type = typeOf.get(header.id) ?? 'text'}
									{@const flag =
										type === 'boolean' && !isMissing(value) ? parseBoolean(value) : null}
									<td
										class={cellClass(type, value)}
										class:sticky={header.id === firstColumn}
										class:first-column={header.id === firstColumn}
										><div class="cell-preview" use:tooltip={{ text: formatValue(value, true), whenTruncated: !webUrl(value) }}>
											{#if webUrl(value)}<ExternalLink
													href={String(value)}
													label={urlLabel(String(value))}
												/>{:else if flag !== null}<span class="flag" class:yes={flag}
													>{#if flag}<CheckIcon size={14} />{:else}<XIcon size={14} />{/if}{flag
														? t('common.yes')
														: t('common.no')}</span
												>{:else}{formatCell(value, type)}{/if}
										</div></td
									>
								{/if}
							{/each}
						</tr>
					{/each}
					{#if items.length}<tr aria-hidden="true" class="spacer"
							><td
								colspan={headers.length + 1}
								style={`height:${Math.max(0, $virtualizer.getTotalSize() - items[items.length - 1].end)}px`}
							></td></tr
						>{/if}
				</tbody>
			</table>
		{/if}
	</div>
</div>

<style>
	.results-table-wrap {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
		min-height: 0;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius-lg);
		background: var(--app-panel);
		overflow: hidden;
	}
	.table-toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--app-border);
		flex-shrink: 0;
	}
	.search {
		display: flex;
		gap: 8px;
		align-items: center;
		color: var(--app-muted);
	}
	.search input {
		width: min(320px, 40vw);
	}
	:global(.select.column-filter) {
		width: auto;
		max-width: 180px;
	}
	.count {
		font-size: var(--app-text-sm);
		color: var(--app-muted);
	}
	.toolbar-actions {
		display: flex;
		align-items: center;
		gap: 4px;
		margin-left: auto;
	}
	.columns-menu {
		position: relative;
	}
	.menu {
		position: absolute;
		top: calc(100% + 4px);
		right: 0;
		z-index: 10;
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 200px;
		max-height: 320px;
		overflow: auto;
		padding: 6px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		background: var(--app-panel);
		box-shadow: var(--app-shadow-popover);
	}
	.menu :global(.checkbox) {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 5px 8px;
		border-radius: var(--app-radius-sm);
		font-size: var(--app-text-md);
		white-space: nowrap;
	}
	.menu :global(.checkbox:hover) {
		background: var(--app-subtle);
	}
	.table-scroll {
		flex: 1;
		min-height: 0;
		overflow: auto;
		scrollbar-gutter: stable;
		overflow-anchor: none;
	}
	table {
		border-collapse: separate;
		border-spacing: 0;
		table-layout: fixed;
		min-width: 100%;
		font-size: var(--app-text-md);
	}
	thead {
		position: sticky;
		top: 0;
		z-index: 3;
	}
	th {
		position: relative;
		text-align: left;
		background: var(--app-subtle);
		border-bottom: 1px solid var(--app-border);
		padding: 0;
		height: 40px;
	}
	th button {
		display: flex;
		width: 100%;
		justify-content: space-between;
		align-items: center;
		gap: 8px;
		color: var(--app-text);
		font-weight: 600;
		padding: 10px 12px;
		text-align: left;
	}
	.header-label {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	/* Sort hints appear on hover or focus; the active sort stays visible. */
	th button :global(svg) {
		opacity: 0;
		color: var(--app-muted);
	}
	th button:hover :global(svg),
	th button:focus-visible :global(svg),
	th button.sorted :global(svg) {
		opacity: 1;
	}
	th button.sorted :global(svg) {
		color: var(--app-accent);
	}
	.resize-handle {
		position: absolute;
		top: 0;
		right: -3px;
		bottom: 0;
		width: 7px;
		cursor: col-resize;
		z-index: 1;
		touch-action: none;
	}
	.resize-handle:hover,
	.resize-handle:focus-visible {
		outline: none;
		background: linear-gradient(
			90deg,
			transparent 2px,
			var(--app-accent) 2px,
			var(--app-accent) 4px,
			transparent 4px
		);
	}
	.sticky {
		position: sticky;
		z-index: 2;
	}
	.actions-cell {
		left: 0;
		padding: 0 4px;
		text-align: center;
	}
	.first-column {
		left: 44px;
		box-shadow: 1px 0 0 var(--app-border);
	}
	.data-row td {
		height: 64px;
		padding: 10px 12px;
		border-bottom: 1px solid var(--app-border);
		background: var(--app-panel);
		cursor: pointer;
	}
	.data-row td.actions-cell {
		padding: 0 4px;
	}
	.data-row.low-confidence td {
		background: color-mix(in srgb, var(--app-warning) 6%, var(--app-panel));
	}
	.data-row:hover td,
	.data-row.selected td {
		background: color-mix(in srgb, var(--app-accent) 7%, var(--app-panel));
	}
	.cell-preview {
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
		overflow-wrap: anywhere;
		line-height: 20px;
		max-height: 40px;
		white-space: pre-wrap;
	}
	.numeric {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}
	th.numeric button {
		flex-direction: row-reverse;
	}
	.missing {
		color: var(--app-muted);
		font-style: italic;
	}
	.flag {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--app-muted);
	}
	.flag.yes {
		color: var(--app-success);
	}
	.evidence {
		font-size: var(--app-text-sm);
		font-variant-numeric: tabular-nums;
	}
	.confidence {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.meter {
		width: 36px;
		height: 4px;
		border-radius: var(--app-radius-pill);
		background: var(--app-subtle);
		overflow: hidden;
	}
	.meter span {
		display: block;
		height: 100%;
		background: var(--app-success);
	}
	.low-confidence .meter span {
		background: var(--app-warning);
	}
	.sources {
		color: var(--app-muted);
	}
	.compact .data-row td {
		height: 36px;
	}
	.compact .data-row td:not(.actions-cell) {
		padding-top: 6px;
		padding-bottom: 6px;
	}
	.compact .cell-preview {
		display: block;
		max-height: 20px;
		white-space: nowrap;
		text-overflow: ellipsis;
	}
	.compact .sources {
		display: none;
	}
	.spacer td {
		padding: 0;
		border: 0;
	}
</style>
