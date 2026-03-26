<script lang="ts">
	import { logs, logFilter, logPanelOpen, clearLogs } from '$lib/stores/logs';
	import { ChevronDownIcon, ChevronUpIcon, TrashIcon } from '@lucide/svelte';
	import type { LogLevel } from '$lib/types';

	const levels: (LogLevel | 'ALL')[] = ['ALL', 'DEBUG', 'INFO', 'WARN', 'ERROR'];

	$: filteredLogs =
		$logFilter === 'ALL'
			? $logs
			: $logs.filter((l) => l.level === $logFilter);

	function togglePanel() {
		logPanelOpen.update((v) => !v);
	}

	function levelColor(level: string): string {
		switch (level) {
			case 'ERROR':
				return 'var(--color-error-500)';
			case 'WARN':
				return 'var(--color-warning-500)';
			case 'INFO':
				return 'var(--color-primary-500)';
			case 'DEBUG':
				return 'var(--color-surface-500)';
			default:
				return 'inherit';
		}
	}
</script>

<div class="log-panel" class:open={$logPanelOpen}>
	<div class="log-header">
		<button class="log-toggle" onclick={togglePanel}>
			{#if $logPanelOpen}
				<ChevronDownIcon size={16} />
			{:else}
				<ChevronUpIcon size={16} />
			{/if}
			<span>Logs ({$logs.length})</span>
		</button>

		{#if $logPanelOpen}
			<div class="log-controls">
				<select bind:value={$logFilter} class="log-filter">
					{#each levels as level}
						<option value={level}>{level}</option>
					{/each}
				</select>
				<button class="btn-icon-sm" onclick={clearLogs} aria-label="Clear logs">
					<TrashIcon size={14} />
				</button>
			</div>
		{/if}
	</div>

	{#if $logPanelOpen}
		<div class="log-body">
			{#each filteredLogs as log}
				<div class="log-entry">
					<span class="log-time">{log.timestamp.substring(11, 19)}</span>
					<span class="log-level" style="color: {levelColor(log.level)}">{log.level}</span>
					<span class="log-message">{log.message}</span>
				</div>
			{:else}
				<div class="log-empty">No log entries</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.log-panel {
		border-top: 1px solid var(--color-surface-300);
		background: var(--color-surface-50);
		flex-shrink: 0;
	}

	.log-panel.open {
		height: 200px;
		display: flex;
		flex-direction: column;
	}

	.log-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 12px;
		background: var(--color-surface-100);
	}

	.log-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		border: none;
		background: transparent;
		cursor: pointer;
		font-size: 0.85rem;
		font-weight: 600;
		color: inherit;
	}

	.log-controls {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.log-filter {
		font-size: 0.8rem;
		padding: 2px 6px;
		border: 1px solid var(--color-surface-300);
		border-radius: 4px;
		background: var(--color-surface-50);
		color: inherit;
	}

	.btn-icon-sm {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 4px;
		border: none;
		background: transparent;
		border-radius: 4px;
		cursor: pointer;
		color: inherit;
	}

	.btn-icon-sm:hover {
		background: var(--color-surface-200);
	}

	.log-body {
		flex: 1;
		overflow-y: auto;
		padding: 4px 12px;
		font-family: monospace;
		font-size: 0.8rem;
	}

	.log-entry {
		display: flex;
		gap: 8px;
		padding: 2px 0;
		border-bottom: 1px solid var(--color-surface-200);
	}

	.log-time {
		color: var(--color-surface-500);
		flex-shrink: 0;
	}

	.log-level {
		font-weight: 600;
		width: 50px;
		flex-shrink: 0;
	}

	.log-message {
		word-break: break-word;
	}

	.log-empty {
		color: var(--color-surface-400);
		text-align: center;
		padding: 20px;
	}
</style>
