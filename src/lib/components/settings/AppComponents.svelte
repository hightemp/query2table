<script lang="ts">
	import { DownloadIcon, RefreshCwIcon, Trash2Icon } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { t } from '$lib/i18n';
	import { checkComponentUpdates, getComponents, installComponent, onComponentProgress, removeComponent } from '$lib/api/tauri';
	import type { ComponentName, ComponentProgress, ComponentsInfo, ComponentStatus, ComponentUpdate } from '$lib/types';
	import { formatBytes } from '$lib/utils/usage';

	let info = $state<ComponentsInfo | null>(null);
	let updates = $state<ComponentUpdate[] | null>(null);
	let busy = $state<ComponentName | null>(null);
	let checking = $state(false);
	let progress = $state<ComponentProgress | null>(null);
	let error = $state<string | null>(null);

	async function load() {
		try {
			info = await getComponents();
		} catch (e) {
			error = String(e);
		}
	}

	onMount(() => {
		load();
		const stop = onComponentProgress((p) => {
			if (p.component === busy) progress = p;
		});
		return () => {
			stop.then((unlisten) => unlisten());
		};
	});

	function name(status: ComponentStatus): string {
		if (status.component === 'ytdlp') return 'yt-dlp';
		return status.bundled ? t('components.quickjs') : 'Deno';
	}

	function updateOf(component: ComponentName): ComponentUpdate | undefined {
		return updates?.find((u) => u.component === component);
	}

	async function install(component: ComponentName) {
		busy = component;
		progress = null;
		error = null;
		try {
			await installComponent(component);
			updates = updates?.map((u) => (u.component === component ? { ...u, current: u.latest, update_available: false } : u)) ?? null;
		} catch (e) {
			error = t('components.failed', { error: String(e) });
		} finally {
			busy = null;
			progress = null;
			await load();
		}
	}

	async function remove(component: ComponentName) {
		error = null;
		try {
			await removeComponent(component);
		} catch (e) {
			error = t('components.failed', { error: String(e) });
		}
		await load();
	}

	async function check() {
		checking = true;
		error = null;
		try {
			updates = await checkComponentUpdates();
		} catch (e) {
			error = t('components.checkFailed', { error: String(e) });
		} finally {
			checking = false;
		}
	}

	function progressText(p: ComponentProgress): string {
		if (p.stage === 'verify') return t('components.verifying');
		if (p.stage === 'install') return t('components.installing');
		const percent = p.total ? Math.floor((p.downloaded / p.total) * 100) : null;
		return percent === null
			? t('components.downloadingSize', { size: formatBytes(p.downloaded) })
			: t('components.downloading', { percent });
	}
</script>

<div class="components">
	<p class="intro">{t('components.description')}</p>
	{#if info && !info.supported}
		<p class="failed">{t('components.unsupported')}</p>
	{:else if info}
		<ul class="list">
			{#each info.components as status (status.component)}
				{@const update = updateOf(status.component)}
				<li class="row">
					<div class="name">
						<span class="title">{name(status)}</span>
						<span class="state" role="status">
							{#if busy === status.component}{progress ? progressText(progress) : t('components.starting')}
							{:else if status.bundled}{t('components.bundled')}
							{:else if status.installed}{t('components.version', { version: status.version ?? '—' })}{#if update?.update_available}
									· {t('components.updateAvailable', { version: update.latest })}{:else if update}
									· {t('components.upToDate')}{/if}
							{:else}{t('components.notInstalled')}{/if}
						</span>
						{#if busy === status.component && progress?.stage === 'download' && progress.total}
							{@const percent = Math.floor((progress.downloaded / progress.total) * 100)}
							<div class="bar" role="progressbar" aria-label={name(status)} aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent}>
								<span style:width="{percent}%"></span>
							</div>
						{/if}
					</div>
					{#if !status.bundled}
						<div class="actions">
							{#if !status.installed}
								<button class="button sm" disabled={busy !== null} onclick={() => install(status.component)}
									><DownloadIcon size={14} />{t('components.install')}</button
								>
							{:else}
								{#if update?.update_available}
									<button class="button sm" disabled={busy !== null} onclick={() => install(status.component)}
										><DownloadIcon size={14} />{t('components.update')}</button
									>
								{/if}
								<button class="button sm ghost" disabled={busy !== null} onclick={() => remove(status.component)}
									><Trash2Icon size={14} />{t('components.remove')}</button
								>
							{/if}
						</div>
					{/if}
				</li>
			{/each}
		</ul>
		<div>
			<button class="button sm" disabled={checking || busy !== null} onclick={check}
				><RefreshCwIcon size={14} />{checking ? t('components.checking') : t('components.check')}</button
			>
		</div>
	{/if}
	{#if error}<p class="failed" role="alert">{error}</p>{/if}
</div>

<style>
	.components {
		display: grid;
		gap: 10px;
		margin-bottom: 12px;
	}
	.intro {
		margin: 0;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.list {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 8px;
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 8px 16px;
		padding: 10px 12px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		background: var(--app-panel);
	}
	.name {
		display: grid;
		gap: 2px;
		min-width: 0;
		flex: 1 1 220px;
	}
	.title {
		font-weight: 600;
	}
	.state {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		overflow-wrap: anywhere;
	}
	.bar {
		width: 100%;
		max-width: 320px;
		height: 6px;
		margin-top: 4px;
		overflow: hidden;
		border-radius: var(--app-radius-pill);
		background: var(--app-subtle);
	}
	.bar span {
		display: block;
		height: 100%;
		background: var(--app-accent);
		transition: width 0.2s ease;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.failed {
		margin: 0;
		color: var(--app-danger);
		font-size: var(--app-text-sm);
		overflow-wrap: anywhere;
	}
</style>
