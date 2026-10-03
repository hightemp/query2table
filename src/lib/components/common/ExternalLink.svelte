<script lang="ts">
	import { openExternal } from '$lib/utils/links';
	import { toast } from '$lib/stores/toasts';
	import type { Snippet } from 'svelte';
	let { href, children, label }: { href: string; children?: Snippet; label?: string } = $props();
	async function open(event: MouseEvent) {
		event.preventDefault();
		event.stopPropagation();
		try {
			await openExternal(href);
		} catch {
			toast('Could not open this link. Copy its address and open it in your browser.', 'error');
		}
	}
</script>

<a {href} title={href} onclick={open}
	>{#if children}{@render children()}{:else}{label ?? href}{/if}</a
>

<style>
	a {
		color: var(--app-accent);
		text-decoration: underline;
		text-underline-offset: 3px;
		overflow-wrap: anywhere;
	}
</style>
