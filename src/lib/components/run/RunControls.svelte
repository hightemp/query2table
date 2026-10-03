<script lang="ts">
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
			<button class="button sm" onclick={onpause} aria-label="Pause" disabled={!!pending}>
				<PauseIcon size={16} />
				{pending === 'pause' ? 'Pausing…' : 'Pause'}
			</button>
		{:else if status === 'paused'}
			<button class="button sm" onclick={onresume} aria-label="Resume" disabled={!!pending}>
				<PlayIcon size={16} />
				{pending === 'resume' ? 'Resuming…' : 'Resume'}
			</button>
		{/if}
		<button
			class="button sm danger outline"
			onclick={oncancel}
			aria-label="Cancel"
			disabled={pending === 'cancel'}
		>
			<XCircleIcon size={16} />
			{pending === 'cancel' ? 'Cancelling…' : 'Cancel'}
		</button>
	{/if}

	{#if isFinished}
		{#if onedit}
			<button class="button sm" onclick={onedit} title="Change the query and run it again">
				<PencilIcon size={16} />
				Edit query
			</button>
		{/if}
		<button class="button sm" onclick={onreset} aria-label="New query">
			<RotateCcwIcon size={16} />
			New query
		</button>
		{#if showExport && onexport}
			<button class="button sm accent" onclick={onexport} aria-label="Export">
				<DownloadIcon size={16} />
				Export
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
