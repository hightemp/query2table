<script lang="ts">
	import { untrack } from 'svelte';
	import { save } from '@tauri-apps/plugin-dialog';
	import { DownloadIcon } from '@lucide/svelte';
	import { exportRun } from '$lib/api/tauri';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import { errorText } from '$lib/utils/errors';
	import { debugUi } from '$lib/utils/diagnostics';
	let {
		runId,
		runType = 'table',
		onclose,
	}: { runId: string; runType?: string; onclose: () => void } = $props();
	let format = $state<'csv' | 'json' | 'xlsx' | 'md'>(
		untrack(() => (runType === 'research' ? 'md' : 'csv'))
	);
	let exporting = $state(false);
	let error = $state('');
	let savedPath = $state('');
	let options = $derived(
		runType === 'research'
			? [{ value: 'md' as const, label: 'Markdown', description: 'Answer document with sources' }]
			: [
					{ value: 'csv' as const, label: 'CSV', description: 'For spreadsheets and data tools' },
					{
						value: 'json' as const,
						label: 'JSON',
						description: 'Structured data with original values',
					},
					{ value: 'xlsx' as const, label: 'Excel', description: 'An Excel workbook' },
				]
	);
	async function handleExport() {
		if (exporting) return;
		error = '';
		exporting = true;
		debugUi('export_started');
		try {
			const option = options.find((item) => item.value === format)!;
			const path = await save({
				defaultPath: `query2table-export.${format}`,
				filters: [{ name: option.label, extensions: [format] }],
			});
			if (!path) return;
			await exportRun(runId, format, path);
			savedPath = path;
			debugUi('export_finished');
		} catch (reason) {
			error = errorText(reason);
			debugUi('export_failed');
		} finally {
			exporting = false;
		}
	}
</script>

<Dialog title={savedPath ? 'Export complete' : 'Export Results'} busy={exporting} {onclose}>
	{#if savedPath}<div role="status">
			<p>Your results were saved.</p>
			<p class="saved-path">{savedPath}</p>
		</div>
	{:else}<fieldset disabled={exporting}>
			<legend>Format</legend>{#each options as option}<label
					class:selected={format === option.value}
					><input type="radio" name="format" value={option.value} bind:group={format} /><span
						><strong>{option.label}</strong><small>{option.description}</small></span
					></label
				>{/each}
		</fieldset>{/if}
	{#if error}<ErrorNotice {error} context="export" />{/if}
	{#snippet footer()}
		{#if savedPath}<button class="button primary" onclick={onclose}>Done</button>{:else}<button
				class="button"
				onclick={onclose}
				disabled={exporting}>Cancel</button
			><button class="button primary" onclick={handleExport} disabled={exporting}
				><DownloadIcon size={16} />{exporting ? 'Exporting…' : 'Export'}</button
			>{/if}
	{/snippet}
</Dialog>

<style>
	fieldset {
		margin: 0;
		padding: 0;
		border: 0;
	}
	legend {
		font-weight: 600;
		font-size: 13px;
		margin-bottom: 8px;
	}
	label {
		display: flex;
		gap: 12px;
		align-items: center;
		border: 1px solid var(--app-border);
		padding: 12px;
		margin-bottom: 8px;
		border-radius: 8px;
		cursor: pointer;
	}
	label.selected {
		border-color: var(--app-accent);
		background: color-mix(in srgb, var(--app-accent) 8%, var(--app-panel));
	}
	strong,
	small {
		display: block;
	}
	small {
		color: var(--app-muted);
		margin-top: 3px;
	}
	.saved-path {
		background: var(--app-subtle);
		padding: 12px;
		border-radius: 8px;
		margin-top: 12px;
		overflow-wrap: anywhere;
	}
</style>
