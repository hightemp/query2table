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

	let { children }: { children: Snippet } = $props();

	$effect(() => {
		const theme = $currentTheme;
		if (theme === 'dark') {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
	});

	onMount(() => {
		settings.load();
		loadTheme();

		const unlisteners: (() => void)[] = [];

		onLogEvent((entry) => {
			addLog(entry as LogEntry);
		}).then((fn) => unlisteners.push(fn));

		onRunLogEntry((e) => {
			addLog({
				timestamp: new Date().toISOString(),
				level: e.level as 'DEBUG' | 'INFO' | 'WARN' | 'ERROR',
				message: `[${e.role}] ${e.message}`,
			});
		}).then((fn) => unlisteners.push(fn));

		return () => {
			for (const fn of unlisteners) fn();
		};
	});
</script>

<div class="app-shell">
	<Sidebar />
	<div class="app-main">
		<main class="app-content">
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
