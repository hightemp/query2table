<script lang="ts">
	import { tooltip } from '$lib/actions/tooltip';
	import { t } from '$lib/i18n';
	import Select from '$lib/components/common/Select.svelte';
	import Checkbox from '$lib/components/common/Checkbox.svelte';

	import { tick } from 'svelte';
	import {
		logs,
		logFilter,
		logPanelOpen,
		logPanelHeight,
		clearLogs,
		logTime,
		formatLogs,
	} from '$lib/stores/logs';
	import { runState } from '$lib/stores/run';
	import { copyText, openAppFolder } from '$lib/api/tauri';
	import { toast } from '$lib/stores/toasts';
	import { modKey } from '$lib/utils/shortcuts';
	import {
		ChevronDownIcon,
		ChevronUpIcon,
		TrashIcon,
		CopyIcon,
		FolderOpenIcon,
		SearchIcon,
	} from '@lucide/svelte';
	import type { LogLevel } from '$lib/types';

	const levels: (LogLevel | 'ALL')[] = ['ALL', 'DEBUG', 'INFO', 'WARN', 'ERROR'];
	const MIN_HEIGHT = 96;
	const STEP = 24;

	let search = $state('');
	let currentRunOnly = $state(false);
	let body = $state<HTMLDivElement>();
	let panel = $state<HTMLDivElement>();
	let following = true;
	let viewportHeight = $state(typeof window === 'undefined' ? 800 : window.innerHeight);

	let runId = $derived($runState.runId);
	let filteredLogs = $derived.by(() => {
		const needle = search.trim().toLowerCase();
		return $logs.filter(
			(entry) =>
				($logFilter === 'ALL' || entry.level === $logFilter) &&
				(!currentRunOnly || !runId || entry.run_id === runId) &&
				(!needle || entry.message.toLowerCase().includes(needle))
		);
	});
	let errorCount = $derived($logs.filter((entry) => entry.level === 'ERROR').length);
	let warnCount = $derived($logs.filter((entry) => entry.level === 'WARN').length);

	function clampHeight(value: number) {
		const max = Math.max(MIN_HEIGHT, Math.round(viewportHeight * 0.6));
		return Math.min(max, Math.max(MIN_HEIGHT, Math.round(value)));
	}
	let height = $derived($logPanelHeight === null ? null : clampHeight($logPanelHeight));

	function togglePanel() {
		logPanelOpen.update((v) => !v);
	}

	// Keep the newest entry in view unless the reader has scrolled up.
	function handleScroll() {
		if (body) following = body.scrollHeight - body.scrollTop - body.clientHeight < 24;
	}
	$effect(() => {
		void filteredLogs;
		const element = body;
		if (!element || !following) return;
		void tick().then(() => {
			element.scrollTop = element.scrollHeight;
		});
	});

	function startResize(event: PointerEvent) {
		if (!panel) return;
		event.preventDefault();
		const handle = event.currentTarget as HTMLElement;
		handle.setPointerCapture(event.pointerId);
		const startY = event.clientY;
		const startHeight = panel.getBoundingClientRect().height;
		function move(moveEvent: PointerEvent) {
			logPanelHeight.set(clampHeight(startHeight + startY - moveEvent.clientY));
		}
		function stop() {
			handle.removeEventListener('pointermove', move);
			handle.removeEventListener('pointerup', stop);
			handle.removeEventListener('pointercancel', stop);
		}
		handle.addEventListener('pointermove', move);
		handle.addEventListener('pointerup', stop);
		handle.addEventListener('pointercancel', stop);
	}
	function resizeWithKeys(event: KeyboardEvent) {
		const current = panel?.getBoundingClientRect().height ?? MIN_HEIGHT;
		if (event.key === 'ArrowUp' || event.key === 'ArrowDown') {
			event.preventDefault();
			logPanelHeight.set(clampHeight(current + (event.key === 'ArrowUp' ? STEP : -STEP)));
		} else if (event.key === 'Home') {
			event.preventDefault();
			logPanelHeight.set(null);
		}
	}

	async function copyLogs() {
		try {
			await copyText(formatLogs(filteredLogs));
			toast(t('logs.copied', { count: filteredLogs.length }), 'success');
		} catch {
			toast(t('logs.copyFailed'), 'error');
		}
	}
	async function openLogFolder() {
		try {
			await openAppFolder('logs');
		} catch {
			toast(t('logs.openFolderFailed'), 'error');
		}
	}
</script>

<svelte:window bind:innerHeight={viewportHeight} />
<div
	class="log-panel"
	class:open={$logPanelOpen}
	bind:this={panel}
	style={$logPanelOpen && height !== null ? `height:${height}px` : undefined}
>
	{#if $logPanelOpen}
		<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
		<div
			class="resize-handle"
			role="separator"
			aria-orientation="horizontal"
			aria-label={t('logs.resize')}
			aria-valuemin={MIN_HEIGHT}
			aria-valuenow={height ?? undefined}
			tabindex="0"
			use:tooltip={t('logs.resizeHint')}
			onpointerdown={startResize}
			onkeydown={resizeWithKeys}
			ondblclick={() => logPanelHeight.set(null)}
		></div>
	{/if}
	<div class="log-header">
		<button
			class="log-toggle"
			onclick={togglePanel}
			aria-expanded={$logPanelOpen}
			aria-controls="app-log-body"
			use:tooltip={t($logPanelOpen ? 'logs.hide' : 'logs.show', { shortcut: `${modKey}+J` })}
		>
			{#if $logPanelOpen}
				<ChevronDownIcon size={16} />
			{:else}
				<ChevronUpIcon size={16} />
			{/if}
			<span>{t('logs.title', { count: $logs.length })}</span>
		</button>
		{#if errorCount}<span class="count error" use:tooltip={t('logs.errors')}>{t('logs.errorCount', { count: errorCount })}</span>{/if}
		{#if warnCount}<span class="count warn" use:tooltip={t('logs.warnings')}>{t('logs.warningCount', { count: warnCount })}</span>{/if}

		{#if $logPanelOpen}
			<div class="log-controls">
				<label class="log-search"
					><SearchIcon size={14} /><input
						class="input sm"
						type="search"
						placeholder={t('logs.filterPlaceholder')}
						aria-label={t('logs.filter')}
						bind:value={search}
					/></label
				>
				{#if runId}<span class="run-only"
						><Checkbox bind:checked={currentRunOnly}>{t('logs.currentRun')}</Checkbox></span
					>{/if}
				<Select
					label={t('logs.level')}
					bind:value={$logFilter}
					size="sm"
					class="log-filter"
					options={levels.map((level) => ({ value: level, label: level === 'ALL' ? t('logs.levelAll') : level }))}
				/>

				<button
					class="icon-button ghost sm"
					onclick={copyLogs}
					disabled={!filteredLogs.length}
					aria-label={t('logs.copy')}
					use:tooltip={t('logs.copy')}><CopyIcon size={14} /></button
				>
				<button
					class="icon-button ghost sm"
					onclick={openLogFolder}
					aria-label={t('logs.openFolder')}
					use:tooltip={t('logs.openFolder')}><FolderOpenIcon size={14} /></button
				>
				<button
					class="icon-button ghost sm"
					onclick={clearLogs}
					aria-label={t('logs.clear')}
					use:tooltip={t('logs.clear')}><TrashIcon size={14} /></button
				>
			</div>
		{/if}
	</div>

	{#if $logPanelOpen}
		<div class="log-body" id="app-log-body" bind:this={body} onscroll={handleScroll}>
			{#each filteredLogs as log}
				<div class="log-entry">
					<span class="log-time" use:tooltip={log.timestamp}>{logTime(log.timestamp)}</span>
					<span class="log-level {log.level.toLowerCase()}">{log.level}</span>
					<span class="log-message">{log.message}</span>
				</div>
			{:else}
				<div class="log-empty">
					{$logs.length ? t('logs.noMatch') : t('logs.empty')}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.log-panel {
		position: relative;
		border-top: 1px solid var(--app-border);
		background: var(--app-bg);
		flex-shrink: 0;
	}

	.log-panel.open {
		height: clamp(120px, 25vh, 240px);
		display: flex;
		flex-direction: column;
	}

	.resize-handle {
		position: absolute;
		top: -4px;
		left: 0;
		right: 0;
		height: 8px;
		cursor: ns-resize;
		z-index: 2;
		touch-action: none;
	}
	.resize-handle:hover,
	.resize-handle:focus-visible {
		outline: none;
		background: linear-gradient(
			transparent 3px,
			var(--app-accent) 3px,
			var(--app-accent) 5px,
			transparent 5px
		);
	}

	.log-header {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 4px 10px;
		padding: 6px 12px;
		background: var(--app-panel);
	}

	.log-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--app-text-md);
		font-weight: 600;
	}

	.count {
		font-size: var(--app-text-xs);
		font-weight: 600;
		padding: 1px 7px;
		border-radius: var(--app-radius-pill);
	}
	.count.error {
		color: var(--app-danger);
		background: color-mix(in srgb, var(--app-danger) 14%, transparent);
	}
	.count.warn {
		color: var(--app-warning);
		background: color-mix(in srgb, var(--app-warning) 14%, transparent);
	}

	.log-controls {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-left: auto;
	}

	.log-search {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--app-muted);
	}
	.log-search input {
		width: 180px;
	}

	.run-only {
		display: flex;
		align-items: center;
		gap: 5px;
		font-size: var(--app-text-sm);
		color: var(--app-muted);
		white-space: nowrap;
	}

	:global(.select.log-filter) {
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
		color: var(--app-muted);
	}
	.log-level.info {
		color: var(--app-accent);
	}
	.log-level.warn {
		color: var(--app-warning);
	}
	.log-level.error {
		color: var(--app-danger);
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
