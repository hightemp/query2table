<script lang="ts">
	import { RefreshCwIcon } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { appVersion } from '$lib/api/tauri';
	import { checkForUpdates, openUpdateDialog, updates } from '$lib/stores/updates';

	let version = $state('');
	onMount(() => {
		appVersion().then(
			(found) => (version = found),
			() => {}
		);
	});
	let update = $derived($updates);
	let checking = $derived(update.status === 'checking');
</script>

<div class="updates">
	<p class="version">{t('updates.currentVersion', { version: version || update.info?.current_version || '—' })}</p>
	<div class="row">
		<button class="button sm" disabled={checking || update.status === 'downloading'} onclick={() => checkForUpdates(true)}
			><RefreshCwIcon size={14} />{checking ? t('updates.checking') : t('updates.check')}</button
		>
		<span class="status" role="status">
			{#if update.status === 'none'}{t('updates.upToDate')}
			{:else if update.info && ['available', 'downloading', 'restart', 'installer', 'permission'].includes(update.status)}
				<button class="link" onclick={openUpdateDialog}>{t('updates.notice', { version: update.info.version })}</button>
			{:else if update.status === 'error' && update.error?.kind === 'check'}<span class="failed"
					>{t('updates.checkFailed')}: {update.error.message}</span
				>{/if}
		</span>
	</div>
</div>

<style>
	.updates {
		display: grid;
		gap: 8px;
		margin-bottom: 12px;
	}
	.version {
		margin: 0;
		font-weight: 600;
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px;
	}
	.status {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		overflow-wrap: anywhere;
	}
	.failed {
		color: var(--app-danger);
	}
	.link {
		padding: 0;
		border: 0;
		background: none;
		color: var(--app-accent);
		text-decoration: underline;
		text-underline-offset: 3px;
		cursor: pointer;
	}
</style>
