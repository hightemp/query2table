<script lang="ts">
	import type { SchemaColumn } from '$lib/types';
	import { PlusIcon, TrashIcon, CheckIcon, XIcon } from '@lucide/svelte';

	interface Props {
		columns: SchemaColumn[];
		onconfirm: (columns: SchemaColumn[]) => void;
		oncancel: () => void;
	}

	let { columns: initialColumns, onconfirm, oncancel }: Props = $props();

	let columns = $state<SchemaColumn[]>(initialColumns.map((c) => ({ ...c })));

	// Keep local state in sync when prop changes (e.g. late-arriving event)
	$effect(() => {
		if (initialColumns.length > 0 && columns.length === 0) {
			columns = initialColumns.map((c) => ({ ...c }));
		}
	});

	function addColumn() {
		columns = [
			...columns,
			{ name: '', type: 'text', description: '', required: false },
		];
	}

	function removeColumn(index: number) {
		columns = columns.filter((_, i) => i !== index);
	}

	function handleConfirm() {
		const valid = columns.filter((c) => c.name.trim());
		if (valid.length === 0) return;
		onconfirm(valid);
	}
</script>

<div class="schema-editor">
	<h2>Proposed Schema</h2>
	<p class="hint">Review and adjust the columns for your results table, then confirm to proceed.</p>

	<div class="columns-list">
		{#each columns as col, i}
			<div class="column-row">
				<input
					class="col-input col-name"
					bind:value={col.name}
					placeholder="Column name"
				/>
				<select class="col-input col-type" bind:value={col.type}>
					<option value="text">Text</option>
					<option value="number">Number</option>
					<option value="url">URL</option>
					<option value="date">Date</option>
					<option value="boolean">Boolean</option>
				</select>
				<input
					class="col-input col-desc"
					bind:value={col.description}
					placeholder="Description"
				/>
				<label class="col-required">
					<input type="checkbox" bind:checked={col.required} />
					Req
				</label>
				<button class="btn-icon-sm" onclick={() => removeColumn(i)} aria-label="Remove column">
					<TrashIcon size={14} />
				</button>
			</div>
		{/each}
	</div>

	<div class="schema-actions">
		<button class="btn-secondary" onclick={addColumn}>
			<PlusIcon size={16} />
			Add Column
		</button>
		<div class="schema-actions-right">
			<button class="btn-ghost" onclick={oncancel}>
				<XIcon size={16} />
				Cancel Run
			</button>
			<button class="btn-primary" onclick={handleConfirm} disabled={columns.filter(c => c.name.trim()).length === 0}>
				<CheckIcon size={16} />
				Confirm Schema
			</button>
		</div>
	</div>
</div>

<style>
	.schema-editor {
		border: 2px solid var(--color-primary-500);
		border-radius: 12px;
		padding: 20px;
		background: var(--color-surface-100-900);
	}

	h2 {
		font-size: 1.2rem;
		font-weight: 700;
		margin-bottom: 4px;
	}

	.hint {
		color: var(--color-surface-600-400);
		font-size: 0.9rem;
		margin-bottom: 16px;
	}

	.columns-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 16px;
	}

	.column-row {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.col-input {
		padding: 6px 10px;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 6px;
		background: var(--color-surface-200-800);
		color: inherit;
		font-size: 0.9rem;
	}

	.col-name {
		flex: 2;
	}

	.col-type {
		flex: 1;
		min-width: 90px;
	}

	.col-desc {
		flex: 3;
	}

	.col-required {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 0.8rem;
		white-space: nowrap;
		cursor: pointer;
	}

	.btn-icon-sm {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 4px;
		border: none;
		background: transparent;
		border-radius: 4px;
		cursor: pointer;
		color: var(--color-error-500);
	}

	.btn-icon-sm:hover {
		background: var(--color-surface-200-800);
	}

	.schema-actions {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.schema-actions-right {
		display: flex;
		gap: 8px;
	}

	.btn-secondary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		background: var(--color-surface-200-800);
		border: 1px solid var(--color-surface-300-700);
		border-radius: 8px;
		cursor: pointer;
		font-size: 0.9rem;
		color: inherit;
	}

	.btn-secondary:hover {
		background: var(--color-surface-300-700);
	}

	.btn-primary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		background: var(--color-primary-500);
		color: white;
		border: none;
		border-radius: 8px;
		font-size: 0.9rem;
		font-weight: 600;
		cursor: pointer;
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--color-primary-600);
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-ghost {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		background: transparent;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 8px;
		cursor: pointer;
		font-size: 0.9rem;
		color: var(--color-error-500);
	}

	.btn-ghost:hover {
		background: var(--color-error-50, rgba(255, 0, 0, 0.05));
	}
</style>
