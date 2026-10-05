<script lang="ts">
	import { t, type MessageKey } from '$lib/i18n';
	import { onMount } from 'svelte';
	import {
		getAppPaths,
		copyAppPath,
		openAppFolder,
		type AppPaths,
		type AppLocation,
	} from '$lib/api/tauri';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import { errorText } from '$lib/utils/errors';

	let paths = $state<AppPaths | null>(null);
	let loading = $state(true);
	let error = $state('');
	let feedback = $state('');
	let busy = $state<string | null>(null);
	let active = true;
	const locations: {
		location: AppLocation;
		key: keyof AppPaths;
		label: MessageKey;
		description: MessageKey;
	}[] = [
		{
			location: 'data',
			key: 'data_dir',
			label: 'files.data',
			description: 'files.dataDescription',
		},
		{
			location: 'database',
			key: 'database_file',
			label: 'files.database',
			description: 'files.databaseDescription',
		},
		{
			location: 'logs',
			key: 'log_dir',
			label: 'files.logs',
			description: 'files.logsDescription',
		},
	];

	async function load() {
		loading = true;
		error = '';
		try {
			const result = await getAppPaths();
			if (active) paths = result;
		} catch (e) {
			if (active) error = errorText(e);
		} finally {
			if (active) loading = false;
		}
	}

	onMount(() => {
		void load();
		return () => {
			active = false;
		};
	});

	async function act(location: AppLocation, label: string, action: 'copy' | 'open') {
		busy = `${location}-${action}`;
		error = '';
		feedback = '';
		try {
			if (action === 'copy') await copyAppPath(location);
			else await openAppFolder(location);
			if (active)
				feedback = action === 'copy' ? t('files.copied', { name: label }) : t('files.opened', { name: label });
		} catch (e) {
			if (active) error = errorText(e);
		} finally {
			if (active) busy = null;
		}
	}
</script>

<section class="app-files" aria-labelledby="app-files-title">
	<h2 id="app-files-title">{t('files.title')}</h2>
	<p>
		{t('files.intro')}
	</p>
	{#if loading}<p role="status">{t('files.loading')}</p>{/if}
	{#if paths}
		{#each locations as item}
			<div class="file-location">
				<label for={`app-path-${item.location}`}>{t(item.label)}</label>
				<p class="description">{t(item.description)}</p>
				<div class="path-controls">
					<input
						id={`app-path-${item.location}`}
						class="input path"
						type="text"
						readonly
						value={paths[item.key]}
						title={paths[item.key]}
						onclick={(e) => e.currentTarget.select()}
					/>
					<button
						type="button"
						class="button"
						disabled={busy !== null}
						aria-label={t('files.copyPath', { name: t(item.label) })}
						onclick={() => act(item.location, t(item.label), 'copy')}
					>
						{busy === `${item.location}-copy` ? t('files.copying') : t('files.copyPathShort')}
					</button>
					<button
						type="button"
						class="button"
						disabled={busy !== null}
						aria-label={t('files.openFolder', { name: t(item.label) })}
						onclick={() => act(item.location, t(item.label), 'open')}
					>
						{busy === `${item.location}-open` ? t('files.opening') : t('files.openFolderShort')}
					</button>
				</div>
			</div>
		{/each}
	{/if}
	{#if feedback}<p role="status">{feedback}</p>{/if}
	{#if error}
		<ErrorNotice {error} context="app_files" />
		{#if !paths && !loading}<button type="button" class="button" onclick={load}
				>{t('files.retry')}</button
			>{/if}
	{/if}
</section>

<style>
	.app-files {
		margin-bottom: 32px;
		padding: 20px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius-lg);
		background: var(--app-panel);
	}
	h2 {
		margin: 0 0 4px;
		font-size: var(--app-text-xl);
		font-weight: 600;
	}
	p {
		margin: 4px 0 12px;
		color: var(--app-muted);
		font-size: var(--app-text-md);
	}
	.file-location {
		margin-top: 16px;
	}
	label {
		font-weight: 500;
	}
	.description {
		font-size: var(--app-text-sm);
		margin-bottom: 6px;
	}
	.path-controls {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.path {
		flex: 1 1 250px;
		width: auto;
		font-family: var(--app-font-mono);
		font-size: var(--app-text-md);
		text-overflow: ellipsis;
	}
	.button:disabled {
		cursor: wait;
	}
</style>
