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
		try {
			await settings.load();
		} catch (error) {
			settingsError = errorText(error);
		}
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

<div class="app-shell">
	<Sidebar />
	<div class="app-main">
		<main class="app-content">
			{#if settingsError || eventError}<div class="app-alerts">
					{#if settingsError}
						<ErrorNotice error={settingsError} context="settings_load" />
						<button class="button" onclick={loadSettings}>Retry loading settings</button>
					{/if}
					{#if eventError}<ErrorNotice error={eventError} />{/if}
				</div>{/if}
			{@render children()}
		</main>
		<LogPanel />
	</div>
</div>

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
