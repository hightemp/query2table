<script lang="ts">
	import { onMount } from 'svelte';
	import { getAppPaths, copyAppPath, openAppFolder, type AppPaths, type AppLocation } from '$lib/api/tauri';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import { errorText } from '$lib/utils/errors';

	let paths = $state<AppPaths | null>(null);
	let loading = $state(true);
	let error = $state('');
	let feedback = $state('');
	let busy = $state<string | null>(null);
	let active = true;
	const locations: { location: AppLocation; key: keyof AppPaths; label: string; description: string }[] = [
		{ location: 'data', key: 'data_dir', label: 'Application data', description: 'Settings, history, results, and sources are stored in data.db here.' },
		{ location: 'database', key: 'database_file', label: 'Settings database', description: 'The SQLite file containing saved application data.' },
		{ location: 'logs', key: 'log_dir', label: 'Application logs', description: 'Diagnostic log files.' },
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
		return () => { active = false; };
	});

	async function act(location: AppLocation, label: string, action: 'copy' | 'open') {
		busy = `${location}-${action}`;
		error = '';
		feedback = '';
		try {
			if (action === 'copy') await copyAppPath(location);
			else await openAppFolder(location);
			if (active) feedback = action === 'copy' ? `${label} path copied.` : `${label} folder opened.`;
		} catch (e) {
			if (active) error = errorText(e);
		} finally {
			if (active) busy = null;
		}
	}
</script>

<section class="app-files" aria-labelledby="app-files-title">
	<h2 id="app-files-title">Application files</h2>
	<p>Settings are updated automatically at startup. New options receive defaults; saved values are kept.</p>
	{#if loading}<p role="status">Loading file locations…</p>{/if}
	{#if paths}
		{#each locations as item}
			<div class="file-location">
				<label for={`app-path-${item.location}`}>{item.label}</label>
				<p class="description">{item.description}</p>
				<div class="path-controls">
					<input id={`app-path-${item.location}`} type="text" readonly value={paths[item.key]} onclick={(e) => e.currentTarget.select()} />
					<button type="button" disabled={busy !== null} aria-label={`Copy ${item.label} path`} onclick={() => act(item.location, item.label, 'copy')}>
						{busy === `${item.location}-copy` ? 'Copying…' : 'Copy path'}
					</button>
					<button type="button" disabled={busy !== null} aria-label={`Open ${item.label} folder`} onclick={() => act(item.location, item.label, 'open')}>
						{busy === `${item.location}-open` ? 'Opening…' : 'Open folder'}
					</button>
				</div>
			</div>
		{/each}
	{/if}
	{#if feedback}<p role="status">{feedback}</p>{/if}
	{#if error}
		<ErrorNotice {error} context="app_files" />
		{#if !paths && !loading}<button type="button" onclick={load}>Retry loading paths</button>{/if}
	{/if}
</section>

<style>
	.app-files { margin-bottom: 32px; padding: 20px; border: 1px solid var(--color-surface-300-700); border-radius: 12px; background: var(--color-surface-100-900); }
	h2 { margin: 0 0 4px; font-size: 1.2rem; font-weight: 600; }
	p { margin: 4px 0 12px; color: var(--color-surface-600-400); font-size: 0.9rem; }
	.file-location { margin-top: 16px; }
	label { font-weight: 500; }
	.description { font-size: 0.82rem; margin-bottom: 6px; }
	.path-controls { display: flex; flex-wrap: wrap; gap: 8px; }
	input { flex: 1 1 250px; min-width: 0; font-family: monospace; }
	input, button { padding: 8px 12px; border: 1px solid var(--color-surface-300-700); border-radius: 6px; background: var(--color-surface-200-800); color: inherit; }
	button { cursor: pointer; }
	button:disabled { opacity: 0.5; cursor: wait; }
	input:focus, button:focus-visible { outline: 2px solid var(--color-primary-500); outline-offset: 2px; }
</style>
