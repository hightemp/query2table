<script lang="ts" module>
	import type { Component } from 'svelte';

	export interface MenuItem {
		label: string;
		icon?: Component<{ size?: number }>;
		action: () => void | Promise<void>;
		disabled?: boolean;
		/** Draws a divider above the item. */
		separator?: boolean;
	}

	/** Position for a menu opened from the keyboard: just below the focused element. */
	export function menuPointFor(element: Element): { x: number; y: number } {
		const rect = element.getBoundingClientRect();
		return { x: rect.left + 8, y: rect.top + Math.min(rect.height, 32) };
	}

	/** True for the keys that open a context menu on the focused element. */
	export function isMenuKey(event: KeyboardEvent): boolean {
		return event.key === 'ContextMenu' || (event.shiftKey && event.key === 'F10');
	}
</script>

<script lang="ts">
	import { onMount, tick } from 'svelte';
	let {
		x,
		y,
		items,
		label,
		onclose,
	}: { x: number; y: number; items: MenuItem[]; label: string; onclose: () => void } = $props();

	let menu: HTMLDivElement;
	let left = $state(0);
	let top = $state(0);
	const previous = document.activeElement as HTMLElement | null;

	function buttons() {
		return [...menu.querySelectorAll<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')];
	}
	function close(restoreFocus = true) {
		onclose();
		if (restoreFocus && previous?.isConnected) previous.focus();
	}
	async function run(item: MenuItem) {
		if (item.disabled) return;
		close();
		await item.action();
	}
	function handleKey(event: KeyboardEvent) {
		const list = buttons();
		const index = list.indexOf(document.activeElement as HTMLButtonElement);
		if (event.key === 'Escape') {
			event.preventDefault();
			close();
		} else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			const step = event.key === 'ArrowDown' ? 1 : -1;
			list[(index + step + list.length) % list.length]?.focus();
		} else if (event.key === 'Home' || event.key === 'End') {
			event.preventDefault();
			list[event.key === 'Home' ? 0 : list.length - 1]?.focus();
		} else if (event.key === 'Tab') {
			event.preventDefault();
			close();
		}
	}

	onMount(() => {
		// Keep the menu inside the window.
		const rect = menu.getBoundingClientRect();
		left = Math.max(8, Math.min(x, window.innerWidth - rect.width - 8));
		top = Math.max(8, y + rect.height > window.innerHeight - 8 ? y - rect.height : y);
		void tick().then(() => buttons()[0]?.focus());
		const dismiss = (event: Event) => {
			if (!menu.contains(event.target as Node)) close(event.type !== 'pointerdown');
		};
		const closeQuietly = () => close(false);
		window.addEventListener('pointerdown', dismiss, true);
		window.addEventListener('scroll', closeQuietly, true);
		window.addEventListener('resize', closeQuietly);
		window.addEventListener('blur', closeQuietly);
		return () => {
			window.removeEventListener('pointerdown', dismiss, true);
			window.removeEventListener('scroll', closeQuietly, true);
			window.removeEventListener('resize', closeQuietly);
			window.removeEventListener('blur', closeQuietly);
		};
	});
</script>

<div
	class="context-menu"
	role="menu"
	aria-label={label}
	tabindex="-1"
	bind:this={menu}
	style={`left:${left}px;top:${top}px`}
	onkeydown={handleKey}
	oncontextmenu={(event) => event.preventDefault()}
>
	{#each items as item}
		{#if item.separator}<div class="separator" role="separator"></div>{/if}
		<button
			role="menuitem"
			tabindex="-1"
			disabled={item.disabled}
			onclick={() => run(item)}
			>{#if item.icon}<item.icon size={15} />{:else}<span class="no-icon"></span>{/if}<span
				>{item.label}</span
			></button
		>
	{/each}
</div>

<style>
	.context-menu {
		position: fixed;
		z-index: 200;
		min-width: 200px;
		max-width: 320px;
		padding: 4px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		background: var(--app-panel);
		box-shadow: var(--app-shadow-popover);
		font-size: var(--app-text-md);
	}
	.context-menu:focus {
		outline: none;
	}
	button {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 6px 10px;
		border-radius: var(--app-radius-sm);
		color: var(--app-text);
		text-align: left;
		white-space: nowrap;
	}
	button span:last-child {
		overflow: hidden;
		text-overflow: ellipsis;
	}
	button:hover:not(:disabled),
	button:focus-visible {
		outline: none;
		background: color-mix(in srgb, var(--app-accent) 14%, transparent);
	}
	button :global(svg),
	.no-icon {
		width: 15px;
		flex-shrink: 0;
		color: var(--app-muted);
	}
	.separator {
		height: 1px;
		margin: 4px 6px;
		background: var(--app-border);
	}
</style>
