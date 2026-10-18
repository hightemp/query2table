<script lang="ts">
	import { DownloadIcon, RotateCwIcon, ShieldAlertIcon, TriangleAlertIcon } from '@lucide/svelte';
	import { t, formatDateTime } from '$lib/i18n';
	import Dialog from './Dialog.svelte';
	import Markdown from './Markdown.svelte';
	import { runState } from '$lib/stores/run';
	import { closeUpdateDialog, installFoundUpdate, restartNow, skipVersion, updates } from '$lib/stores/updates';

	let update = $derived($updates);
	let info = $derived(update.info);
	let busy = $derived(update.status === 'downloading');
	let running = $derived($runState.status === 'running' || $runState.status === 'pending');
	let packageInstall = $derived(info?.installer === 'deb' || info?.installer === 'rpm');
	let percent = $derived(
		update.progress?.total ? Math.min(100, Math.round((update.progress.downloaded / update.progress.total) * 100)) : null
	);
	function megabytes(bytes: number) {
		return (bytes / 1024 / 1024).toFixed(1);
	}
</script>

{#if info}
	<Dialog title={t('updates.title', { version: info.version })} onclose={closeUpdateDialog} {busy}>
		<p class="versions">
			{t('updates.versions', { current: info.current_version, version: info.version })}{info.date
				? ` · ${formatDateTime(info.date, { dateStyle: 'medium' })}`
				: ''}
		</p>
		<div class="notes" aria-label={t('updates.changes')} role="region">
			{#if info.notes}<Markdown content={info.notes} />{:else}<p class="muted">{t('updates.noNotes')}</p>{/if}
		</div>

		{#if running && (update.status === 'available' || update.status === 'error')}
			<p class="note warning"><TriangleAlertIcon size={15} />{t('updates.runWarning')}</p>
		{/if}
		{#if packageInstall && update.status !== 'restart'}
			<p class="note"><ShieldAlertIcon size={15} />{t('updates.adminNote')}</p>
		{/if}

		{#if update.status === 'downloading'}
			<div class="progress" role="status">
				<div
					class="bar"
					role="progressbar"
					aria-label={t('updates.downloading')}
					aria-valuemin={0}
					aria-valuemax={100}
					aria-valuenow={percent ?? undefined}
				>
					<span style:width={percent === null ? '35%' : `${percent}%`} class:indeterminate={percent === null}></span>
				</div>
				<span
					>{update.progress?.total
						? t('updates.downloaded', { done: megabytes(update.progress.downloaded), total: megabytes(update.progress.total) })
						: t('updates.downloading')}</span
				>
			</div>
		{:else if update.status === 'restart'}
			<p class="note success" role="status">{t('updates.installed')}</p>
		{:else if update.status === 'installer'}
			<p class="note success" role="status">{t('updates.installerOpen')}</p>
		{:else if update.status === 'permission'}
			<p class="note" role="status">{t('updates.permission')}</p>
		{:else if update.status === 'error' && update.error}
			<div class="update-error" class:signature={update.error.kind === 'signature'} role="alert">
				<strong
					>{update.error.kind === 'signature'
						? t('updates.signatureFailed')
						: update.error.kind === 'download'
							? t('updates.downloadFailed')
							: update.error.kind === 'check'
								? t('updates.checkFailed')
								: t('updates.installFailed')}</strong
				>
				{#if update.error.kind === 'signature'}<p>{t('updates.signatureHint')}</p>{/if}
				<details>
					<summary>{t('common.technicalDetails')}</summary>
					<code>{update.error.message}</code>
				</details>
			</div>
		{/if}

		{#snippet footer()}
			{#if update.status === 'restart'}
				<button class="button" onclick={closeUpdateDialog}>{t('updates.later')}</button>
				<button class="button primary" onclick={restartNow}><RotateCwIcon size={15} />{t('updates.restartNow')}</button>
			{:else if update.status === 'installer'}
				<button class="button" onclick={closeUpdateDialog}>{t('common.close')}</button>
			{:else}
				<button class="button ghost skip" disabled={busy} onclick={() => info && skipVersion(info.version)}
					>{t('updates.skip')}</button
				>
				<button class="button" disabled={busy} onclick={closeUpdateDialog}>{t('updates.later')}</button>
				<button class="button primary" disabled={busy} onclick={installFoundUpdate}
					><DownloadIcon size={15} />{update.status === 'error' || update.status === 'permission'
						? t('updates.tryAgain')
						: t('updates.install')}</button
				>
			{/if}
		{/snippet}
	</Dialog>
{/if}

<style>
	.versions {
		margin: 0 0 12px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.notes {
		max-height: min(45vh, 420px);
		overflow: auto;
		padding: 4px 12px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		background: var(--app-bg);
	}
	.muted {
		color: var(--app-muted);
	}
	.note {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		margin: 12px 0 0;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.note :global(svg) {
		flex-shrink: 0;
		margin-top: 2px;
	}
	.note.warning {
		color: var(--app-warning, var(--app-text));
	}
	.note.success {
		color: var(--app-success, var(--app-text));
	}
	.progress {
		display: grid;
		gap: 6px;
		margin-top: 12px;
		font-size: var(--app-text-sm);
		color: var(--app-muted);
	}
	.bar {
		height: 6px;
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
	.bar span.indeterminate {
		animation: slide 1.2s ease-in-out infinite;
	}
	@keyframes slide {
		from {
			transform: translateX(-100%);
		}
		to {
			transform: translateX(300%);
		}
	}
	.update-error {
		margin-top: 12px;
		padding: 10px 12px;
		border: 1px solid var(--app-danger);
		border-radius: var(--app-radius);
		color: var(--app-text);
		font-size: var(--app-text-sm);
	}
	.update-error strong {
		color: var(--app-danger);
	}
	.update-error p {
		margin: 6px 0 0;
	}
	.update-error details {
		margin-top: 6px;
	}
	.update-error code {
		display: block;
		margin-top: 4px;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		font: var(--app-text-xs) / 1.5 var(--app-font-mono);
	}
	.skip {
		margin-right: auto;
	}
</style>
