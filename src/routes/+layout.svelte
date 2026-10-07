<script lang="ts">
	import '../app.css';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import LogPanel from '$lib/components/layout/LogPanel.svelte';
	import { settings } from '$lib/stores/settings';
	import { onMount } from 'svelte';
	import { onLogEvent, onRunLogEntry } from '$lib/api/tauri';
	import { addLog, logPanelOpen } from '$lib/stores/logs';
	import { appliedTheme, currentTheme, loadTheme, sidebarCollapsed, uiScale } from '$lib/stores/ui';
	import Toaster from '$lib/components/layout/Toaster.svelte';
	import ContextMenu from '$lib/components/common/ContextMenu.svelte';
	import { contextMenu, closeContextMenu } from '$lib/stores/contextMenu';
	import { hasMod } from '$lib/utils/shortcuts';
	import type { LogEntry } from '$lib/types';
	import type { Snippet } from 'svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import { errorText } from '$lib/utils/errors';
	import { loadLanguage, t } from '$lib/i18n';

	let { children }: { children: Snippet } = $props();
	let settingsError = $state('');
	let eventError = $state('');
	// Runs before the first render, so a cached language is used from the start.
	void loadLanguage().catch(() => {});

	async function loadSettings() {
		settingsError = '';
		try {
			await settings.load();
		} catch (error) {
			settingsError = errorText(error);
		}
	}

	$effect(() => {
		const root = document.documentElement;
		root.classList.toggle('dark', $currentTheme === 'dark');
		root.dataset.theme = $appliedTheme;
		root.dataset.scale = $uiScale;
	});

	// The webview's own menu offers browser actions (open in new window, save image, reload)
	// that do not work in the app. Keep it only where it helps: text fields and selected text.
	// In development, Shift+right-click still opens it for "Inspect element".
	function handleContextMenu(event: MouseEvent) {
		if (event.defaultPrevented) return;
		const target = event.target as Element | null;
		if (target?.closest('input, textarea, [contenteditable="true"]')) return;
		if (window.getSelection()?.toString().trim()) return;
		if (import.meta.env.DEV && event.shiftKey) return;
		event.preventDefault();
	}

	function handleShortcut(event: KeyboardEvent) {
		if (!hasMod(event) || event.shiftKey) return;
		const key = event.key.toLowerCase();
		if (key === 'b') {
			event.preventDefault();
			sidebarCollapsed.update((value) => !value);
		} else if (key === 'j') {
			event.preventDefault();
			logPanelOpen.update((value) => !value);
		}
	}

	onMount(() => {
		void loadSettings();
		void loadTheme().catch((error) => {
			settingsError = errorText(error);
		});

		const unlisteners: (() => void)[] = [];
		let disposed = false;
		function registered(unsubscribe: () => void) {
			if (disposed) unsubscribe();
			else unlisteners.push(unsubscribe);
		}
		function failed(error: unknown) {
			if (!disposed) eventError = errorText(error);
		}

		onLogEvent((entry) => {
			addLog(entry as LogEntry);
		})
			.then(registered)
			.catch(failed);

		onRunLogEntry((e) => {
			addLog({
				run_id: e.run_id,
				role: e.role,
				timestamp: new Date().toISOString(),
				level: e.level as 'DEBUG' | 'INFO' | 'WARN' | 'ERROR',
				message: `[${e.role}] ${e.message}`,
			});
		})
			.then(registered)
			.catch(failed);

		return () => {
			disposed = true;
			for (const fn of unlisteners) fn();
		};
	});
</script>

<svelte:window onkeydown={handleShortcut} oncontextmenu={handleContextMenu} />
<div class="app-shell">
	<Sidebar />
	<div class="app-main">
		<main class="app-content">
			{#if settingsError || eventError}<div class="app-alerts">
					{#if settingsError}
						<ErrorNotice error={settingsError} context="settings_load" />
						<button class="button" onclick={loadSettings}>{t('common.retryLoadingSettings')}</button>
					{/if}
					{#if eventError}<ErrorNotice error={eventError} />{/if}
				</div>{/if}
			{@render children()}
		</main>
		<LogPanel />
	</div>
</div>
<Toaster />
{#if $contextMenu}
	{#key $contextMenu}
		<ContextMenu
			x={$contextMenu.x}
			y={$contextMenu.y}
			label={$contextMenu.label}
			items={$contextMenu.items}
			onclose={closeContextMenu}
		/>
	{/key}
{/if}

<style>
	.app-alerts {
		max-height: 30vh;
		overflow: auto;
		flex-shrink: 0;
		margin-bottom: 12px;
		scrollbar-gutter: stable;
		padding-right: 12px;
	}
	.app-shell {
		display: flex;
		height: 100dvh;
		overflow: hidden;
	}

	.app-main {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
		overflow: hidden;
	}

	.app-content {
		flex: 1;
		overflow: hidden;
		padding: 20px;
		display: flex;
		flex-direction: column;
		min-height: 0;
	}
</style>
