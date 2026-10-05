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
		MonitorIcon,
	} from '@lucide/svelte';
	import {
		sidebarCollapsed,
		themePreference,
		setTheme,
		nextTheme,
		type ThemePreference,
	} from '$lib/stores/ui';
	import { toast } from '$lib/stores/toasts';
	import { modKey } from '$lib/utils/shortcuts';
	import { t, type MessageKey } from '$lib/i18n';

	const nav: { href: string; label: MessageKey; icon: typeof SearchIcon }[] = [
		{ href: '/', label: 'nav.query', icon: SearchIcon },
		{ href: '/history', label: 'nav.history', icon: HistoryIcon },
		{ href: '/settings', label: 'nav.settings', icon: SettingsIcon },
	];
	const themes: { value: ThemePreference; label: MessageKey; icon: typeof SunIcon }[] = [
		{ value: 'light', label: 'theme.light', icon: SunIcon },
		{ value: 'dark', label: 'theme.dark', icon: MoonIcon },
		{ value: 'system', label: 'theme.system', icon: MonitorIcon },
	];
	let savingTheme = $state(false);
	let current = $derived(themes.find((theme) => theme.value === $themePreference) ?? themes[2]);
	let upcoming = $derived(themes.find((theme) => theme.value === nextTheme($themePreference))!);

	async function changeTheme(next: ThemePreference) {
		if (savingTheme) return;
		savingTheme = true;
		try {
			await setTheme(next);
		} catch {
			toast(t('theme.saveFailed'), 'error');
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
		<button
			class="icon-button ghost"
			onclick={toggleSidebar}
			aria-label={t('sidebar.toggle')}
			aria-expanded={!$sidebarCollapsed}
			title={t($sidebarCollapsed ? 'sidebar.expand' : 'sidebar.collapse', { shortcut: `${modKey}+B` })}
		>
			{#if $sidebarCollapsed}
				<PanelLeftOpenIcon size={20} />
			{:else}
				<PanelLeftCloseIcon size={20} />
			{/if}
		</button>
	</div>

	<nav class="sidebar-nav">
		{#each nav as item (item.href)}
			{@const active = $page.url.pathname === item.href}
			<a
				href={item.href}
				aria-label={t(item.label)}
				title={$sidebarCollapsed ? t(item.label) : undefined}
				aria-current={active ? 'page' : undefined}
				class="nav-item"
				class:active
			>
				<item.icon size={20} />
				{#if !$sidebarCollapsed}<span>{t(item.label)}</span>{/if}
			</a>
		{/each}
	</nav>

	<div class="sidebar-footer">
		{#if $sidebarCollapsed}
			<button
				class="icon-button ghost"
				onclick={() => changeTheme(upcoming.value)}
				disabled={savingTheme}
				aria-label={t('theme.current', { current: t(current.label), next: t(upcoming.label) })}
				title={t('theme.current', { current: t(current.label), next: t(upcoming.label) })}
			>
				<current.icon size={20} />
			</button>
		{:else}
			<div class="theme-switch" role="group" aria-label={t('theme.group')}>
				{#each themes as theme (theme.value)}
					<button
						class:active={$themePreference === theme.value}
						aria-pressed={$themePreference === theme.value}
						disabled={savingTheme}
						aria-label={t(theme.label)}
						title={theme.value === 'system' ? t('theme.followSystem') : t('theme.named', { name: t(theme.label) })}
						onclick={() => changeTheme(theme.value)}
						><theme.icon size={15} />{#if $themePreference === theme.value}<span>{t(theme.label)}</span
							>{/if}</button
					>
				{/each}
			</div>
		{/if}
	</div>
</aside>

<style>
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

	/* Fits the 32px content box of the collapsed sidebar. */
	.icon-button {
		width: 32px;
		min-height: 32px;
		padding: 6px;
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
		font-size: var(--app-text-lg);
		white-space: nowrap;
		overflow: hidden;
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
		border-radius: var(--app-radius);
		text-decoration: none;
		color: var(--app-text);
		font-size: var(--app-text-base);
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
		padding: 12px;
	}

	.collapsed .sidebar-footer {
		justify-content: center;
	}

	.theme-switch {
		display: flex;
		width: 100%;
		padding: 3px;
		gap: 2px;
		border-radius: var(--app-radius);
		background: var(--app-subtle);
	}

	/* Only the chosen theme is named, so longer translations still fit. */
	.theme-switch button {
		flex: 0 0 34px;
		min-width: 0;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 4px;
		min-height: 28px;
		padding: 4px 2px;
		border-radius: var(--app-radius-sm);
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}

	.theme-switch button:hover:not(.active) {
		color: var(--app-text);
	}

	.theme-switch button.active {
		flex: 1 1 auto;
		background: var(--app-panel);
		color: var(--app-text);
		font-weight: 600;
		box-shadow: 0 1px 2px rgb(0 0 0 / 12%);
	}

	.theme-switch button:disabled {
		opacity: 1;
		cursor: wait;
	}
</style>
