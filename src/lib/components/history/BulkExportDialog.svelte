<script lang="ts">
	import { t } from '$lib/i18n';
	import Select from '$lib/components/common/Select.svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { FolderOpenIcon } from '@lucide/svelte';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import { exportRuns } from '$lib/api/tauri';
	import { errorText } from '$lib/utils/errors';

	let { runIds, onclose }: { runIds: string[]; onclose: () => void } = $props();
	let format = $state<'csv' | 'json' | 'xlsx'>('csv');
	let exporting = $state(false);
	let error = $state('');
	let result = $state<{ dir: string; count: number } | null>(null);

	async function handleExport() {
		if (exporting) return;
		error = '';
		exporting = true;
		try {
			const dir = await open({ directory: true, title: t('bulk.chooseFolder') });
			if (!dir || Array.isArray(dir)) return;
			const written = await exportRuns(runIds, dir, format);
			result = { dir, count: written.length };
		} catch (reason) {
			error = errorText(reason);
		} finally {
			exporting = false;
		}
	}
</script>

<Dialog title={t('bulk.title')} busy={exporting} {onclose}>
	{#if result}
		<p role="status">
			{t('bulk.exported', { count: result.count })} <span class="path">{result.dir}</span>
		</p>
	{:else}
		<p>
			{t('bulk.description', { count: runIds.length })}
		</p>
		<label class="field"
			>{t('bulk.format')}<Select
				bind:value={format}
				disabled={exporting}
				options={[
					{ value: 'csv', label: 'CSV' },
					{ value: 'json', label: 'JSON' },
					{ value: 'xlsx', label: t('export.excel') },
				]}
			/></label

		>
		<p class="note">{t('bulk.markdownNote')}</p>
	{/if}
	{#if error}<ErrorNotice {error} context="export" />{/if}
	{#snippet footer()}
		{#if result}<button class="button primary" onclick={onclose}>{t('export.done')}</button>
		{:else}<button class="button" disabled={exporting} onclick={onclose}>{t('export.cancel')}</button
			><button class="button primary" disabled={exporting} onclick={handleExport}
				><FolderOpenIcon size={16} />{exporting ? t('export.exporting') : t('bulk.export')}</button
			>{/if}
	{/snippet}
</Dialog>

<style>
	.field {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-top: 12px;
		color: var(--app-muted);
		font-size: var(--app-text-md);
	}
	.note {
		margin-top: 8px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.path {
		overflow-wrap: anywhere;
		font-weight: 600;
	}
</style>
