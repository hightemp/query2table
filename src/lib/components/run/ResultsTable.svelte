<script lang="ts">
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
	} from '@lucide/svelte';
	import type { SchemaColumn } from '$lib/types';
	import type { RunRow } from '$lib/stores/run';
	import { formatValue, webUrl, urlLabel } from '$lib/utils/values';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	let {
		schema,
		rows,
		onrowclick,
	}: { schema: SchemaColumn[]; rows: RunRow[]; onrowclick: (row: RunRow) => void } = $props();
	let sorting = $state<SortingState>([]);
	let filter = $state('');
	let scrollElement = $state<HTMLDivElement>();
	let focusedRowId = $state<string | null>(null);
	let filteredRows = $derived(
		rows.filter(
			(row) =>
				!filter.trim() ||
				schema.some((column) =>
					formatValue(row.data[column.name]).toLowerCase().includes(filter.trim().toLowerCase())
				)
		)
	);
	let columnDefs = $derived<ColumnDef<RunRow, unknown>[]>(
		schema.map((column) => ({
			id: column.name,
			accessorFn: (row) => row.data[column.name] ?? '',
			header: column.name,
		}))
	);
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
	let focusIndex = $derived(visibleRows.findIndex((row) => row.id === focusedRowId));
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
		untrack(() =>
			get(virtualizer).setOptions({
				count,
				getScrollElement: () => element ?? null,
				getItemKey: (index) => keys[index],
				rangeExtractor: (range) =>
					[...new Set([...defaultRangeExtractor(range), ...(focused >= 0 ? [focused] : [])])].sort(
						(a, b) => a - b
					),
			})
		);
	});
	// Filtering updates the row model before the virtualizer effect runs. Drop obsolete indices during that transition.
	let items = $derived(
		$virtualizer.getVirtualItems().filter((item) => item.index < visibleRows.length)
	);
	async function moveFocus(event: KeyboardEvent, index: number) {
		if (!['ArrowDown', 'ArrowUp'].includes(event.key)) return;
		event.preventDefault();
		const next = Math.max(
			0,
			Math.min(visibleRows.length - 1, index + (event.key === 'ArrowDown' ? 1 : -1))
		);
		focusedRowId = visibleRows[next].id;
		$virtualizer.scrollToIndex(next, { align: 'auto' });
		await tick();
		scrollElement
			?.querySelector<HTMLButtonElement>(`button[data-row-index="${next}"]`)
			?.focus({ preventScroll: true });
	}
</script>

<div class="results-table-wrap">
	<div class="table-toolbar">
		<label
			><SearchIcon size={16} /><input
				type="search"
				aria-label="Search results"
				placeholder="Search results…"
				bind:value={filter}
			/></label
		><span
			>{visibleRows.length}{filter ? ` of ${rows.length}` : ''}
			{rows.length === 1 ? 'row' : 'rows'}</span
		>
	</div>
	<!-- Scroll region is keyboard-focusable to allow horizontal and vertical scrolling. -->
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<div
		class="table-scroll"
		bind:this={scrollElement}
		tabindex="0"
		role="region"
		aria-label="Results table"
	>
		{#if !schema.length}<div class="empty-state">The result schema will appear here.</div>
		{:else if !visibleRows.length}<div class="empty-state">
				{filter ? 'No matching rows. Try another search.' : 'No results yet.'}
			</div>
		{:else}
			<table style={`width: ${schema.length * 220 + 64}px`} aria-rowcount={visibleRows.length + 1}>
				<colgroup
					>{#each schema as column}<col style="width:220px" />{/each}<col
						style="width:64px"
					/></colgroup
				>
				<thead
					><tr
						>{#each table.getHeaderGroups()[0].headers as header}<th
								aria-sort={header.column.getIsSorted() === 'asc'
									? 'ascending'
									: header.column.getIsSorted() === 'desc'
										? 'descending'
										: 'none'}
								><button onclick={header.column.getToggleSortingHandler()}
									>{header.column.id}{#if header.column.getIsSorted() === 'asc'}<ArrowUpIcon
											size={14}
										/>{:else if header.column.getIsSorted() === 'desc'}<ArrowDownIcon
											size={14}
										/>{:else}<ArrowUpDownIcon size={14} />{/if}</button
								></th
							>{/each}<th><span class="sr-only">Details</span></th></tr
					></thead
				>
				<tbody>
					{#each items as item, index (item.key)}
						{@const row = visibleRows[item.index]}
						{@const gap = item.start - (items[index - 1]?.end ?? 0)}
						{#if gap > 0}<tr aria-hidden="true" class="spacer"
								><td colspan={schema.length + 1} style={`height:${gap}px`}></td></tr
							>{/if}
						<tr
							class="data-row"
							class:selected={focusedRowId === row.id}
							aria-rowindex={item.index + 2}
						>
							{#each schema as column}
								{@const value = row.original.data[column.name]}
								<td
									><div class="cell-preview" title={formatValue(value, true)}>
										{#if webUrl(value)}<ExternalLink
												href={String(value)}
												label={urlLabel(String(value))}
											/>{:else}{formatValue(value)}{/if}
									</div></td
								>
							{/each}
							<td
								><button
									class="icon-button"
									data-row-index={item.index}
									aria-label={`Open row ${item.index + 1} details`}
									onfocus={() => {
										focusedRowId = row.id;
									}}
									onkeydown={(event) => moveFocus(event, item.index)}
									onclick={() => onrowclick(row.original)}><PanelRightOpenIcon size={16} /></button
								></td
							>
						</tr>
					{/each}
					{#if items.length}<tr aria-hidden="true" class="spacer"
							><td
								colspan={schema.length + 1}
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
		border-radius: 12px;
		background: var(--app-panel);
		overflow: hidden;
	}
	.table-toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		border-bottom: 1px solid var(--app-border);
		flex-shrink: 0;
	}
	.table-toolbar label {
		display: flex;
		gap: 8px;
		align-items: center;
		color: var(--app-muted);
		flex: 1;
	}
	.table-toolbar input {
		border: 0;
		background: transparent;
		color: var(--app-text);
		width: min(100%, 320px);
		padding: 4px;
	}
	.table-toolbar span {
		font-size: 12px;
		color: var(--app-muted);
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
		font-size: 13px;
	}
	thead {
		position: sticky;
		top: 0;
		z-index: 1;
		background: var(--app-subtle);
	}
	th {
		text-align: left;
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
		background: transparent;
		color: var(--app-text);
		font-weight: 600;
		padding: 10px 12px;
		text-align: left;
		border: 0;
		overflow-wrap: anywhere;
	}
	.data-row {
		height: 64px;
	}
	.data-row td {
		height: 64px;
		padding: 10px 12px;
		border-bottom: 1px solid var(--app-border);
	}
	.data-row:hover,
	.data-row.selected {
		background: color-mix(in srgb, var(--app-accent) 5%, var(--app-panel));
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
	.spacer td {
		padding: 0;
		border: 0;
	}
</style>
