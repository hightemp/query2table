<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { MenuItem } from './ContextMenu.svelte';
	import { isMenuKey, menuPointFor } from './ContextMenu.svelte';
	import { showContextMenu } from '$lib/stores/contextMenu';
	import { linkMenuItems, openLink } from '$lib/utils/linkMenu';
	let {
		href,
		children,
		label,
		onopen,
		menuItems,
		class: className = '',
	}: {
		href: string;
		children?: Snippet;
		label?: string;
		/** Called after the link was opened in the browser. */
		onopen?: () => void;
		/** Replaces the default link menu (Open, Copy link, Copy as Markdown). */
		menuItems?: () => MenuItem[];
		class?: string;
	} = $props();
	async function open(event: MouseEvent) {
		event.preventDefault();
		event.stopPropagation();
		if (await openLink(href)) onopen?.();
	}
	function menu(point: { x: number; y: number }) {
		showContextMenu(point, `Actions for ${label ?? href}`, menuItems?.() ?? linkMenuItems(href, label));
	}
</script>

<a
	{href}
	title={href}
	class={className}
	onclick={open}
	aria-haspopup="menu"
	oncontextmenu={(event) => {
		event.preventDefault();
		event.stopPropagation();
		menu({ x: event.clientX, y: event.clientY });
	}}
	onkeydown={(event) => {
		if (isMenuKey(event)) {
			event.preventDefault();
			menu(menuPointFor(event.currentTarget));
		}
	}}>{#if children}{@render children()}{:else}{label ?? href}{/if}</a
>

<style>
	a {
		color: var(--app-accent);
		text-decoration: underline;
		text-underline-offset: 3px;
		overflow-wrap: anywhere;
	}
</style>
