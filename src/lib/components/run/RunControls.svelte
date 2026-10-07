<script lang="ts">
	import { tooltip } from '$lib/actions/tooltip';
	import { t } from '$lib/i18n';
	import {
		PauseIcon,
		PlayIcon,
		XCircleIcon,
		RotateCcwIcon,
		DownloadIcon,
		PencilIcon,
	} from '@lucide/svelte';
	import type { RunControl } from '$lib/stores/run';
	import Badge from '$lib/components/common/Badge.svelte';
	import { statusLabel, statusTone } from '$lib/utils/status';

	interface Props {
		status: string;
		pending?: RunControl | null;
		onpause: () => void;
		onresume: () => void;
		oncancel: () => void;
		onreset: () => void;
		onedit?: () => void;
		onexport?: () => void;
		showExport?: boolean;
	}

	let {
		status,
		pending = null,
		onpause,
		onresume,
		oncancel,
		onreset,
		onedit,
		onexport,
		showExport = false,
	}: Props = $props();

	let isActive = $derived(
		status === 'running' ||
			status === 'paused' ||
			status === 'pending' ||
			status === 'schema_review'
	);
	let isFinished = $derived(
		status === 'completed' || status === 'failed' || status === 'cancelled'
	);
</script>

<div class="run-controls">
	<Badge tone={statusTone(status)}>{statusLabel(status)}</Badge>

	{#if isActive}
		{#if status === 'running' || status === 'pending' || status === 'schema_review'}
			<button class="button sm" onclick={onpause} aria-label={t('controls.pause')} disabled={!!pending}>
				<PauseIcon size={16} />
				{pending === 'pause' ? t('controls.pausing') : t('controls.pause')}
			</button>
		{:else if status === 'paused'}
			<button class="button sm" onclick={onresume} aria-label={t('controls.resume')} disabled={!!pending}>
				<PlayIcon size={16} />
				{pending === 'resume' ? t('controls.resuming') : t('controls.resume')}
			</button>
		{/if}
		<button
			class="button sm danger outline"
			onclick={oncancel}
			aria-label={t('controls.cancel')}
			disabled={pending === 'cancel'}
		>
			<XCircleIcon size={16} />
			{pending === 'cancel' ? t('controls.cancelling') : t('controls.cancel')}
		</button>
	{/if}

	{#if isFinished}
		{#if onedit}
			<button class="button sm" onclick={onedit} use:tooltip={t('controls.editQueryHint')}>
				<PencilIcon size={16} />
				{t('controls.editQuery')}
			</button>
		{/if}
		<button class="button sm" onclick={onreset} aria-label={t('controls.newQuery')}>
			<RotateCcwIcon size={16} />
			{t('controls.newQuery')}
		</button>
		{#if showExport && onexport}
			<button class="button sm accent" onclick={onexport} aria-label={t('controls.export')}>
				<DownloadIcon size={16} />
				{t('controls.export')}
			</button>
		{/if}
	{/if}
</div>

<style>
	.run-controls {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 8px;
	}
	.button:disabled {
		cursor: wait;
	}
</style>
