<script lang="ts">
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
			const dir = await open({ directory: true, title: 'Choose a folder for the exported runs' });
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

<Dialog title="Export runs" busy={exporting} {onclose}>
	{#if result}
		<p role="status">
			Exported {result.count}
			{result.count === 1 ? 'file' : 'files'} to <span class="path">{result.dir}</span>
		</p>
	{:else}
		<p>
			{runIds.length === 1 ? 'The selected run is' : `${runIds.length} selected runs are`} saved one file
			each, named after the query and date.
		</p>
		<label class="field"
			>Format for tables, links and images<select class="input" bind:value={format} disabled={exporting}>
				<option value="csv">CSV</option>
				<option value="json">JSON</option>
				<option value="xlsx">Excel</option>
			</select></label
		>
		<p class="note">Research conversations are saved as Markdown.</p>
	{/if}
	{#if error}<ErrorNotice {error} context="export" />{/if}
	{#snippet footer()}
		{#if result}<button class="button primary" onclick={onclose}>Done</button>
		{:else}<button class="button" disabled={exporting} onclick={onclose}>Cancel</button
			><button class="button primary" disabled={exporting} onclick={handleExport}
				><FolderOpenIcon size={16} />{exporting ? 'Exporting…' : 'Choose folder and export'}</button
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
