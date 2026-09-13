<script lang="ts">
	import { untrack } from 'svelte';
	import type { SchemaColumn } from '$lib/types';
	import { PlusIcon, TrashIcon, CheckIcon } from '@lucide/svelte';
	let {
		columns: initialColumns,
		onconfirm,
		oncancel,
		pending = false,
	}: {
		columns: SchemaColumn[];
		onconfirm: (columns: SchemaColumn[]) => void;
		oncancel: () => void;
		pending?: boolean;
	} = $props();
	let columns = $state<SchemaColumn[]>(
		untrack(() => initialColumns.map((column) => ({ ...column })))
	);
	let previousColumns = untrack(() => initialColumns);
	$effect(() => {
		if (initialColumns !== previousColumns) {
			previousColumns = initialColumns;
			columns = initialColumns.map((column) => ({ ...column }));
		}
	});
	let validation = $derived.by(() => {
		if (!columns.length) return 'Add at least one column.';
		const names = columns.map((column) => column.name.trim().toLowerCase());
		if (names.some((name) => !name)) return 'Give every column a name.';
		if (new Set(names).size !== names.length) return 'Column names must be unique.';
		return '';
	});
	function confirm() {
		if (!validation && !pending)
			onconfirm(columns.map((column) => ({ ...column, name: column.name.trim() })));
	}
</script>

<div class="schema-editor">
	<header>
		<h2>Proposed Schema</h2>
		<p>Review the columns before searching. You can change names, types and descriptions.</p>
	</header>
	<div class="columns-list">
		{#each columns as column, i}
			<div class="column-row">
				<label
					>Name<input
						bind:value={column.name}
						placeholder="Column name"
						aria-label={`Column ${i + 1} name`}
						disabled={pending}
					/></label
				>
				<label
					>Type<select
						bind:value={column.type}
						aria-label={`Column ${i + 1} type`}
						disabled={pending}
						>{#each ['text', 'number', 'url', 'date', 'boolean'] as type}<option value={type}
								>{type}</option
							>{/each}</select
					></label
				>
				<label class="description"
					>Description<input
						bind:value={column.description}
						placeholder="Description"
						aria-label={`Column ${i + 1} description`}
						disabled={pending}
					/></label
				>
				<label class="required"
					><input
						type="checkbox"
						bind:checked={column.required}
						disabled={pending}
					/>Required</label
				>
				<button
					class="icon-button"
					onclick={() => {
						columns = columns.filter((_, index) => index !== i);
					}}
					disabled={pending}
					aria-label={`Remove column ${i + 1}`}><TrashIcon size={16} /></button
				>
			</div>
		{/each}
		{#if validation}<p class="validation" role="status">{validation}</p>{/if}
	</div>
	<footer>
		<button
			class="button"
			disabled={pending}
			onclick={() => {
				columns = [...columns, { name: '', type: 'text', description: '', required: false }];
			}}><PlusIcon size={16} />Add Column</button
		>
		<div>
			<button class="button danger" onclick={oncancel}>Cancel Run</button><button
				class="button primary"
				disabled={!!validation || pending}
				onclick={confirm}
				><CheckIcon size={16} />{pending ? 'Confirming…' : 'Confirm Schema'}</button
			>
		</div>
	</footer>
</div>

<style>
	.schema-editor {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
		background: var(--app-panel);
		border: 1px solid var(--app-border);
		border-radius: 12px;
		overflow: hidden;
	}
	header {
		padding: 16px 20px;
		flex-shrink: 0;
		border-bottom: 1px solid var(--app-border);
	}
	h2 {
		font-size: 17px;
		font-weight: 650;
	}
	p {
		color: var(--app-muted);
		margin-top: 4px;
		font-size: 13px;
	}
	.columns-list {
		min-height: 0;
		flex: 1;
		overflow: auto;
		padding: 16px;
		scrollbar-gutter: stable;
		container-type: inline-size;
	}
	.column-row {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		gap: 8px;
		margin-bottom: 12px;
		padding-bottom: 12px;
		border-bottom: 1px solid var(--app-border);
	}
	label {
		display: flex;
		flex-direction: column;
		flex: 1 1 110px;
		min-width: 0;
		font-size: 12px;
		gap: 4px;
		color: var(--app-muted);
	}
	.description {
		flex: 2 1 160px;
	}
	.required {
		flex: 0 0 auto;
		flex-direction: row;
		align-items: center;
		height: 36px;
	}
	input:not([type='checkbox']),
	select {
		width: 100%;
		height: 36px;
		padding: 7px 10px;
		border: 1px solid var(--app-border);
		border-radius: 8px;
		background: var(--app-bg);
		color: var(--app-text);
	}
	footer {
		padding: 12px 16px;
		display: flex;
		justify-content: space-between;
		gap: 8px;
		flex-wrap: wrap;
		flex-shrink: 0;
		border-top: 1px solid var(--app-border);
	}
	footer div {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}
	.validation {
		color: var(--color-warning-500);
	}
</style>
