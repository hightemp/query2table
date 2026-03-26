<script lang="ts">
	import { PauseIcon, PlayIcon, XCircleIcon, RotateCcwIcon } from '@lucide/svelte';

	interface Props {
		status: string;
		onpause: () => void;
		onresume: () => void;
		oncancel: () => void;
		onreset: () => void;
	}

	let { status, onpause, onresume, oncancel, onreset }: Props = $props();

	let isActive = $derived(
		status === 'running' || status === 'paused' || status === 'pending' || status === 'schema_review'
	);
	let isFinished = $derived(
		status === 'completed' || status === 'failed' || status === 'cancelled'
	);
</script>

<div class="run-controls">
	<span class="status-badge" class:running={status === 'running'} class:paused={status === 'paused'} class:completed={status === 'completed'} class:failed={status === 'failed'} class:cancelled={status === 'cancelled'}>
		{status}
	</span>

	{#if isActive}
		{#if status === 'running'}
			<button class="ctrl-btn" onclick={onpause} aria-label="Pause">
				<PauseIcon size={16} />
				Pause
			</button>
		{:else if status === 'paused'}
			<button class="ctrl-btn" onclick={onresume} aria-label="Resume">
				<PlayIcon size={16} />
				Resume
			</button>
		{/if}
		<button class="ctrl-btn ctrl-cancel" onclick={oncancel} aria-label="Cancel">
			<XCircleIcon size={16} />
			Cancel
		</button>
	{/if}

	{#if isFinished}
		<button class="ctrl-btn" onclick={onreset} aria-label="New query">
			<RotateCcwIcon size={16} />
			New Query
		</button>
	{/if}
</div>

<style>
	.run-controls {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.status-badge {
		padding: 4px 10px;
		border-radius: 12px;
		font-size: 0.8rem;
		font-weight: 600;
		text-transform: capitalize;
		background: var(--color-surface-200);
		color: var(--color-surface-600);
	}

	.status-badge.running {
		background: var(--color-primary-100, rgba(59, 130, 246, 0.15));
		color: var(--color-primary-500);
	}

	.status-badge.paused {
		background: var(--color-warning-100, rgba(245, 158, 11, 0.15));
		color: var(--color-warning-500);
	}

	.status-badge.completed {
		background: rgba(34, 197, 94, 0.15);
		color: var(--color-success-500, #22c55e);
	}

	.status-badge.failed {
		background: rgba(239, 68, 68, 0.15);
		color: var(--color-error-500);
	}

	.status-badge.cancelled {
		background: var(--color-surface-200);
		color: var(--color-surface-500);
	}

	.ctrl-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px 12px;
		border: 1px solid var(--color-surface-300);
		border-radius: 6px;
		background: var(--color-surface-50);
		cursor: pointer;
		font-size: 0.85rem;
		color: inherit;
	}

	.ctrl-btn:hover {
		background: var(--color-surface-200);
	}

	.ctrl-cancel {
		color: var(--color-error-500);
		border-color: var(--color-error-500);
	}

	.ctrl-cancel:hover {
		background: rgba(239, 68, 68, 0.1);
	}
</style>
