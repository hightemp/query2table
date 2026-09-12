<script lang="ts">
	import '../app.css';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import LogPanel from '$lib/components/layout/LogPanel.svelte';
	import { settings } from '$lib/stores/settings';
	import { onMount } from 'svelte';
	import { onLogEvent, onRunLogEntry } from '$lib/api/tauri';
	import { addLog } from '$lib/stores/logs';
	import { currentTheme, loadTheme } from '$lib/stores/ui';
	import type { LogEntry } from '$lib/types';
	import type { Snippet } from 'svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import { errorText } from '$lib/utils/errors';

	let { children }: { children: Snippet } = $props();
	let settingsError = $state('');
	let eventError = $state('');

	async function loadSettings() {
		settingsError = '';
		try { await settings.load(); }
		catch (error) { settingsError = errorText(error); }
	}

	$effect(() => {
		const theme = $currentTheme;
		if (theme === 'dark') {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
	});

	onMount(() => {
		void loadSettings();
		loadTheme();

		const unlisteners: (() => void)[] = [];
		let disposed = false;
		function registered(unsubscribe: () => void) {
			if (disposed) unsubscribe();
			else unlisteners.push(unsubscribe);
		}
		function failed(error: unknown) { if (!disposed) eventError = errorText(error); }

		onLogEvent((entry) => {
			addLog(entry as LogEntry);
		}).then(registered).catch(failed);

		onRunLogEntry((e) => {
			addLog({
				timestamp: new Date().toISOString(),
				level: e.level as 'DEBUG' | 'INFO' | 'WARN' | 'ERROR',
				message: `[${e.role}] ${e.message}`,
			});
		}).then(registered).catch(failed);

		return () => {
			disposed = true;
			for (const fn of unlisteners) fn();
		};
	});
</script>

<div class="app-shell">
	<Sidebar />
	<div class="app-main">
		<main class="app-content">
			{#if settingsError}
				<ErrorNotice error={settingsError} context="settings_load" />
				<button onclick={loadSettings}>Retry loading settings</button>
			{/if}
			{#if eventError}<ErrorNotice error={eventError} />{/if}
			{@render children()}
		</main>
		<LogPanel />
	</div>
</div>

<style>
	.app-shell {
		display: flex;
		height: 100vh;
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
		padding: 24px;
		display: flex;
		flex-direction: column;
		min-height: 0;
	}
</style>
