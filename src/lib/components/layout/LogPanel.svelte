<script lang="ts">
	import { logs, logFilter, logPanelOpen, clearLogs } from '$lib/stores/logs';
	import { ChevronDownIcon, ChevronUpIcon, TrashIcon } from '@lucide/svelte';
	import type { LogLevel } from '$lib/types';

	const levels: (LogLevel | 'ALL')[] = ['ALL', 'DEBUG', 'INFO', 'WARN', 'ERROR'];

	let filteredLogs = $derived(
		$logFilter === 'ALL' ? $logs : $logs.filter((l) => l.level === $logFilter)
	);

	function togglePanel() {
		logPanelOpen.update((v) => !v);
	}

	function levelColor(level: string): string {
		switch (level) {
			case 'ERROR':
				return 'var(--app-danger)';
			case 'WARN':
				return 'var(--app-warning)';
			case 'INFO':
				return 'var(--app-accent)';
			case 'DEBUG':
				return 'var(--app-muted)';
			default:
				return 'inherit';
		}
	}
</script>

<div class="log-panel" class:open={$logPanelOpen}>
	<div class="log-header">
		<button
			class="log-toggle"
			onclick={togglePanel}
			aria-expanded={$logPanelOpen}
			aria-controls="app-log-body"
		>
			{#if $logPanelOpen}
				<ChevronDownIcon size={16} />
			{:else}
				<ChevronUpIcon size={16} />
			{/if}
			<span>Logs ({$logs.length})</span>
		</button>

		{#if $logPanelOpen}
			<div class="log-controls">
				<select aria-label="Log level" bind:value={$logFilter} class="input sm log-filter">
					{#each levels as level}
						<option value={level}>{level}</option>
					{/each}
				</select>
				<button class="icon-button ghost sm" onclick={clearLogs} aria-label="Clear logs">
					<TrashIcon size={14} />
				</button>
			</div>
		{/if}
	</div>

	{#if $logPanelOpen}
		<div class="log-body" id="app-log-body">
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
		border-top: 1px solid var(--app-border);
		background: var(--app-bg);
		flex-shrink: 0;
	}

	.log-panel.open {
		height: clamp(120px, 25vh, 240px);
		display: flex;
		flex-direction: column;
	}

	.log-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 12px;
		background: var(--app-panel);
	}

	.log-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		border: none;
		background: transparent;
		cursor: pointer;
		font-size: var(--app-text-md);
		font-weight: 600;
		color: inherit;
	}

	.log-controls {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.log-filter {
		width: auto;
	}

	.log-body {
		flex: 1;
		min-height: 0;
		scrollbar-gutter: stable;
		overflow-y: auto;
		padding: 4px 12px;
		font-family: var(--app-font-mono);
		font-size: var(--app-text-sm);
	}

	.log-entry {
		display: flex;
		gap: 8px;
		padding: 2px 0;
		border-bottom: 1px solid var(--app-subtle);
	}

	.log-time {
		color: var(--app-muted);
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
		color: var(--app-muted);
		text-align: center;
		padding: 20px;
	}
</style>
