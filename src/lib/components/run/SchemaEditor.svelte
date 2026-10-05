<script lang="ts">
	import { t, type MessageKey } from '$lib/i18n';
	import { untrack } from 'svelte';
	import type { SchemaColumn } from '$lib/types';
	import { PlusIcon, TrashIcon, CheckIcon } from '@lucide/svelte';
	let {
		columns: initialColumns,
		onconfirm,
		pending = false,
	}: {
		columns: SchemaColumn[];
		onconfirm: (columns: SchemaColumn[]) => void;
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
		if (!columns.length) return t('schema.needColumn');
		const names = columns.map((column) => column.name.trim().toLowerCase());
		if (names.some((name) => !name)) return t('schema.needNames');
		if (new Set(names).size !== names.length) return t('schema.uniqueNames');
		return '';
	});
	function confirm() {
		if (!validation && !pending)
			onconfirm(columns.map((column) => ({ ...column, name: column.name.trim() })));
	}
</script>

<div class="schema-editor">
	<header>
		<h2>{t('schema.title')}</h2>
		<p>{t('schema.subtitle')}</p>
	</header>
	<div class="columns-list">
		{#each columns as column, i}
			<div class="column-row">
				<label
					>{t('schema.name')}<input
						class="input"
						bind:value={column.name}
						placeholder={t('schema.namePlaceholder')}
						aria-label={t('schema.columnName', { n: i + 1 })}
						disabled={pending}
					/></label
				>
				<label
					>{t('schema.type')}<select
						class="input"
						bind:value={column.type}
						aria-label={t('schema.columnType', { n: i + 1 })}
						disabled={pending}
						>{#each ['text', 'number', 'url', 'date', 'boolean'] as type}<option value={type}
								>{t(`schema.type.${type}` as MessageKey)}</option
							>{/each}</select
					></label
				>
				<label class="description"
					>{t('schema.description')}<input
						class="input"
						bind:value={column.description}
						placeholder={t('schema.description')}
						aria-label={t('schema.columnDescription', { n: i + 1 })}
						disabled={pending}
					/></label
				>
				<label class="required"
					><input
						type="checkbox"
						bind:checked={column.required}
						disabled={pending}
					/>{t('schema.required')}</label
				>
				<button
					class="icon-button danger"
					onclick={() => {
						columns = columns.filter((_, index) => index !== i);
					}}
					disabled={pending}
					aria-label={t('schema.remove', { n: i + 1 })}><TrashIcon size={16} /></button
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
			}}><PlusIcon size={16} />{t('schema.add')}</button
		>
		<div>
			<button
				class="button primary"
				disabled={!!validation || pending}
				onclick={confirm}
				><CheckIcon size={16} />{pending ? t('schema.confirming') : t('schema.confirm')}</button
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
		border-radius: var(--app-radius-lg);
		overflow: hidden;
	}
	header {
		padding: 16px 20px;
		flex-shrink: 0;
		border-bottom: 1px solid var(--app-border);
	}
	h2 {
		font-size: var(--app-text-xl);
		font-weight: 650;
	}
	p {
		color: var(--app-muted);
		margin-top: 4px;
		font-size: var(--app-text-md);
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
		font-size: var(--app-text-sm);
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
		color: var(--app-warning);
	}
</style>
