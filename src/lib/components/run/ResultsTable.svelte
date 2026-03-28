<script lang="ts">
	import {
		getCoreRowModel,
		getSortedRowModel,
		type ColumnDef,
		type SortingState,
	} from '@tanstack/table-core';
	import { createTable as createSvelteTable } from '@tanstack/svelte-table';
	import { FlexRender } from '@tanstack/svelte-table';
	import type { SchemaColumn } from '$lib/types';
	import type { RunRow } from '$lib/stores/run';

	interface Props {
		schema: SchemaColumn[];
		rows: RunRow[];
		onrowclick: (row: RunRow) => void;
	}

	let { schema, rows, onrowclick }: Props = $props();

	let sorting = $state<SortingState>([]);

	let columnDefs = $derived<ColumnDef<RunRow, unknown>[]>(
		schema.map((col) => ({
			id: col.name,
			accessorFn: (row: RunRow) => row.data[col.name] ?? '',
			header: col.name,
			cell: (info: { getValue: () => unknown }) => String(info.getValue()),
		}))
	);

	let table = $derived(
		createSvelteTable({
			get data() { return rows; },
			get columns() { return columnDefs; },
			state: {
				get sorting() { return sorting; },
			},
			onSortingChange: (updater) => {
				if (typeof updater === 'function') {
					sorting = updater(sorting);
				} else {
					sorting = updater;
				}
			},
			getCoreRowModel: getCoreRowModel(),
			getSortedRowModel: getSortedRowModel(),
		})
	);
</script>

<div class="results-table-wrap">
	{#if schema.length === 0}
		<div class="empty-state">No schema defined yet.</div>
	{:else if rows.length === 0}
		<div class="empty-state">Waiting for results...</div>
	{:else}
		<div class="table-scroll">
			<table class="results-table">
				<thead>
					{#each table.getHeaderGroups() as headerGroup}
						<tr>
							{#each headerGroup.headers as header}
								<th
									class:sortable={header.column.getCanSort()}
									onclick={header.column.getToggleSortingHandler()}
								>
									<FlexRender content={header.column.columnDef.header} context={header.getContext()} />
									{#if header.column.getIsSorted() === 'asc'}
										<span class="sort-indicator"> ↑</span>
									{:else if header.column.getIsSorted() === 'desc'}
										<span class="sort-indicator"> ↓</span>
									{/if}
								</th>
							{/each}
						</tr>
					{/each}
				</thead>
				<tbody>
					{#each table.getRowModel().rows as row}
						<tr class="data-row" onclick={() => onrowclick(row.original)}>
							{#each row.getVisibleCells() as cell}
								<td>
									<FlexRender content={cell.column.columnDef.cell} context={cell.getContext()} />
								</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<div class="row-count">{rows.length} row{rows.length !== 1 ? 's' : ''}</div>
	{/if}
</div>

<style>
	.results-table-wrap {
		border: 1px solid var(--color-surface-300-700);
		border-radius: 8px;
		overflow: hidden;
		background: var(--color-surface-50-950);
		display: flex;
		flex-direction: column;
		min-height: 0;
		flex: 1;
	}

	.table-scroll {
		overflow: auto;
		flex: 1;
		min-height: 0;
	}

	.results-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.9rem;
	}

	.results-table thead {
		background: var(--color-surface-100-900);
		position: sticky;
		top: 0;
		z-index: 1;
	}

	.results-table th {
		padding: 10px 14px;
		text-align: left;
		font-weight: 600;
		font-size: 0.85rem;
		white-space: nowrap;
		border-bottom: 2px solid var(--color-surface-300-700);
		user-select: none;
	}

	.results-table th.sortable {
		cursor: pointer;
	}

	.results-table th.sortable:hover {
		background: var(--color-surface-200-800);
	}

	.sort-indicator {
		font-size: 0.75rem;
	}

	.results-table td {
		padding: 8px 14px;
		border-bottom: 1px solid var(--color-surface-200-800);
		max-width: 300px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.data-row {
		cursor: pointer;
	}

	.data-row:hover {
		background: var(--color-surface-100-900);
	}

	.row-count {
		padding: 8px 14px;
		font-size: 0.8rem;
		color: var(--color-surface-600-400);
		border-top: 1px solid var(--color-surface-200-800);
	}

	.empty-state {
		text-align: center;
		padding: 40px 20px;
		color: var(--color-surface-400-600);
	}
</style>
