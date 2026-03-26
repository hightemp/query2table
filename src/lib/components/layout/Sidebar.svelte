<script lang="ts">
	import { page } from '$app/stores';
	import { SearchIcon, HistoryIcon, SettingsIcon, PanelLeftCloseIcon, PanelLeftOpenIcon } from '@lucide/svelte';
	import { sidebarCollapsed } from '$lib/stores/ui';

	function toggleSidebar() {
		sidebarCollapsed.update((v) => !v);
	}
</script>

<aside class="sidebar" class:collapsed={$sidebarCollapsed}>
	<div class="sidebar-header">
		{#if !$sidebarCollapsed}
			<span class="sidebar-title">Query2Table</span>
		{/if}
		<button class="btn-icon" onclick={toggleSidebar} aria-label="Toggle sidebar">
			{#if $sidebarCollapsed}
				<PanelLeftOpenIcon size={20} />
			{:else}
				<PanelLeftCloseIcon size={20} />
			{/if}
		</button>
	</div>

	<nav class="sidebar-nav">
		<a href="/" class="nav-item" class:active={$page.url.pathname === '/'}>
			<SearchIcon size={20} />
			{#if !$sidebarCollapsed}<span>Query</span>{/if}
		</a>
		<a href="/history" class="nav-item" class:active={$page.url.pathname === '/history'}>
			<HistoryIcon size={20} />
			{#if !$sidebarCollapsed}<span>History</span>{/if}
		</a>
		<a href="/settings" class="nav-item" class:active={$page.url.pathname === '/settings'}>
			<SettingsIcon size={20} />
			{#if !$sidebarCollapsed}<span>Settings</span>{/if}
		</a>
	</nav>
</aside>

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		width: 220px;
		min-height: 100vh;
		background: var(--color-surface-100);
		border-right: 1px solid var(--color-surface-300);
		transition: width 0.2s ease;
		flex-shrink: 0;
	}

	.sidebar.collapsed {
		width: 56px;
	}

	.sidebar-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px;
		gap: 8px;
	}

	.sidebar-title {
		font-weight: 700;
		font-size: 1.1rem;
		white-space: nowrap;
		overflow: hidden;
	}

	.btn-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 6px;
		border: none;
		background: transparent;
		border-radius: 6px;
		cursor: pointer;
		color: inherit;
	}

	.btn-icon:hover {
		background: var(--color-surface-200);
	}

	.sidebar-nav {
		display: flex;
		flex-direction: column;
		padding: 8px;
		gap: 4px;
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 10px 12px;
		border-radius: 8px;
		text-decoration: none;
		color: var(--color-surface-900);
		font-size: 0.95rem;
		transition: background 0.15s;
	}

	.nav-item:hover {
		background: var(--color-surface-200);
	}

	.nav-item.active {
		background: var(--color-primary-500);
		color: white;
	}

	.collapsed .sidebar-header {
		justify-content: center;
	}

	.collapsed .nav-item {
		justify-content: center;
		padding: 10px;
	}
</style>
