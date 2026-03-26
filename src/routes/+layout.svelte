<script lang="ts">
	import '../app.css';
	import Sidebar from '$lib/components/layout/Sidebar.svelte';
	import LogPanel from '$lib/components/layout/LogPanel.svelte';
	import { settings } from '$lib/stores/settings';
	import { onMount } from 'svelte';
	import { onLogEvent } from '$lib/api/tauri';
	import { addLog } from '$lib/stores/logs';
	import type { LogEntry } from '$lib/types';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	onMount(() => {
		settings.load();

		let unlisten: (() => void) | undefined;
		onLogEvent((entry) => {
			addLog(entry as LogEntry);
		}).then((fn) => {
			unlisten = fn;
		});

		return () => {
			unlisten?.();
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
		overflow-y: auto;
		padding: 24px;
	}
</style>
