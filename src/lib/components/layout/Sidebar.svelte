<script lang="ts">
	import { page } from '$app/stores';
	import {
		SearchIcon,
		HistoryIcon,
		SettingsIcon,
		PanelLeftCloseIcon,
		PanelLeftOpenIcon,
		SunIcon,
		MoonIcon,
	} from '@lucide/svelte';
	import { sidebarCollapsed, currentTheme, toggleTheme } from '$lib/stores/ui';
	let themeError = $state(false);
	let savingTheme = $state(false);
	async function changeTheme() {
		if (savingTheme) return;
		savingTheme = true;
		themeError = false;
		try {
			await toggleTheme();
		} catch {
			themeError = true;
		} finally {
			savingTheme = false;
		}
	}

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
		<a
			href="/"
			aria-label="Query"
			title="Query"
			aria-current={$page.url.pathname === '/' ? 'page' : undefined}
			class="nav-item"
			class:active={$page.url.pathname === '/'}
		>
			<SearchIcon size={20} />
			{#if !$sidebarCollapsed}<span>Query</span>{/if}
		</a>
		<a
			href="/history"
			aria-label="History"
			title="History"
			aria-current={$page.url.pathname === '/history' ? 'page' : undefined}
			class="nav-item"
			class:active={$page.url.pathname === '/history'}
		>
			<HistoryIcon size={20} />
			{#if !$sidebarCollapsed}<span>History</span>{/if}
		</a>
		<a
			href="/settings"
			aria-label="Settings"
			title="Settings"
			aria-current={$page.url.pathname === '/settings' ? 'page' : undefined}
			class="nav-item"
			class:active={$page.url.pathname === '/settings'}
		>
			<SettingsIcon size={20} />
			{#if !$sidebarCollapsed}<span>Settings</span>{/if}
		</a>
	</nav>

	<div class="sidebar-footer">
		<button class="btn-icon" onclick={changeTheme} disabled={savingTheme} aria-label="Toggle theme">
			{#if $currentTheme === 'dark'}
				<SunIcon size={20} />
			{:else}
				<MoonIcon size={20} />
			{/if}
		</button>
		{#if !$sidebarCollapsed}
			<span class="theme-label">{$currentTheme === 'dark' ? 'Light mode' : 'Dark mode'}</span>
		{/if}
	</div>
	{#if themeError}<p class="theme-error" role="alert">
			Could not save the theme. Try the theme button again.
		</p>{/if}
</aside>

<style>
	.theme-error {
		margin: 0;
		padding: 8px;
		color: var(--color-error-500);
		overflow-wrap: anywhere;
		font-size: 12px;
	}
	.sidebar {
		display: flex;
		flex-direction: column;
		width: 220px;
		height: 100%;
		min-height: 0;
		background: var(--app-panel);
		border-right: 1px solid var(--app-border);
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
		background: var(--app-subtle);
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
		color: var(--app-text);
		font-size: 0.95rem;
		transition: background 0.15s;
	}

	.nav-item:hover {
		background: var(--app-subtle);
	}

	.nav-item.active {
		background: color-mix(in srgb, var(--app-accent) 14%, transparent);
		color: var(--app-accent);
		font-weight: 650;
	}

	.collapsed .sidebar-header {
		justify-content: center;
	}

	.collapsed .nav-item {
		justify-content: center;
		padding: 10px;
	}

	.sidebar-footer {
		margin-top: auto;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 12px;
	}

	.theme-label {
		font-size: 0.85rem;
		white-space: nowrap;
		opacity: 0.7;
	}
</style>
