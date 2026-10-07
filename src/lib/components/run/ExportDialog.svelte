<script lang="ts">
	import { t } from '$lib/i18n';
	import { untrack } from 'svelte';
	import { save } from '@tauri-apps/plugin-dialog';
	import { DownloadIcon } from '@lucide/svelte';
	import { exportRun } from '$lib/api/tauri';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import Select from '$lib/components/common/Select.svelte';
	import Radio from '$lib/components/common/Radio.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import { errorText } from '$lib/utils/errors';
	import { debugUi } from '$lib/utils/diagnostics';
	let {
		runId,
		runType = 'table',
		turns = [],
		onclose,
	}: {
		runId: string;
		runType?: string;
		/** Research turns, to export one of them instead of the whole conversation. */
		turns?: { index: number; question: string }[];
		onclose: () => void;
	} = $props();
	let turnChoice = $state('all');
	let format = $state<'csv' | 'json' | 'xlsx' | 'md'>(
		untrack(() => (runType === 'research' ? 'md' : 'csv'))
	);
	let exporting = $state(false);
	let error = $state('');
	let savedPath = $state('');
	let options = $derived(
		runType === 'research'
			? [{ value: 'md' as const, label: t('export.markdown'), description: t('export.markdownDescription') }]
			: [
					{ value: 'csv' as const, label: 'CSV', description: t('export.csvDescription') },
					{
						value: 'json' as const,
						label: 'JSON',
						description: t('export.jsonDescription'),
					},
					{ value: 'xlsx' as const, label: t('export.excel'), description: t('export.excelDescription') },
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
			await exportRun(runId, format, path, turnChoice === 'all' ? null : Number(turnChoice));
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

<Dialog title={savedPath ? t('export.complete') : t('export.title')} busy={exporting} {onclose}>
	{#if savedPath}<div role="status">
			<p>{t('export.saved')}</p>
			<p class="saved-path">{savedPath}</p>
		</div>
	{:else}<fieldset disabled={exporting}>
			<legend>{t('export.format')}</legend>{#each options as option}<Radio
					name="format"
					value={option.value}
					checked={format === option.value}
					onchange={() => (format = option.value)}
					><strong>{option.label}</strong><small>{option.description}</small></Radio
				>{/each}
		</fieldset>
		{#if runType === 'research'}
			<label class="turn-choice"
				>{t('export.what')}<Select
					bind:value={turnChoice}
					disabled={exporting}
					options={[
						{ value: 'all', label: t('export.wholeConversation') },
						...turns.map((turn) => ({
							value: String(turn.index),
							label: t('export.question', { n: turn.index + 1, question: turn.question }),
						})),
					]}
				/></label

			>
		{/if}{/if}
	{#if error}<ErrorNotice {error} context="export" />{/if}
	{#snippet footer()}
		{#if savedPath}<button class="button primary" onclick={onclose}>{t('export.done')}</button>{:else}<button
				class="button"
				onclick={onclose}
				disabled={exporting}>{t('export.cancel')}</button
			><button class="button primary" onclick={handleExport} disabled={exporting}
				><DownloadIcon size={16} />{exporting ? t('export.exporting') : t('export.export')}</button
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
		font-size: var(--app-text-md);
		margin-bottom: 8px;
	}
	fieldset :global(.radio) {
		display: flex;
		gap: 12px;
		align-items: center;
		border: 1px solid var(--app-border);
		padding: 12px;
		margin-bottom: 8px;
		border-radius: var(--app-radius);
	}
	fieldset :global(.radio:has(input:checked)) {

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
	.turn-choice {
		display: flex;
		flex-direction: column;
		align-items: stretch;
		gap: 4px;
		margin-top: 12px;
		padding: 0;
		border: 0;
		cursor: default;
		font-size: var(--app-text-md);
		color: var(--app-muted);
	}
	.saved-path {
		background: var(--app-subtle);
		padding: 12px;
		border-radius: var(--app-radius);
		margin-top: 12px;
		overflow-wrap: anywhere;
	}
</style>
