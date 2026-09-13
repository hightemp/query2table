<script lang="ts">
	import { openExternal } from '$lib/utils/links';
	import type { Snippet } from 'svelte';
	let { href, children, label }: { href: string; children?: Snippet; label?: string } = $props();
	let error = $state(false);
	async function open(event: MouseEvent) {
		event.preventDefault();
		event.stopPropagation();
		error = false;
		try {
			await openExternal(href);
		} catch {
			error = true;
		}
	}
</script>

<a {href} title={href} onclick={open}
	>{#if children}{@render children()}{:else}{label ?? href}{/if}</a
>
{#if error}<span class="open-error" role="alert"
		>Could not open this link. Copy its address and open it in your browser.</span
	>{/if}

<style>
	a {
		color: var(--app-accent);
		text-decoration: underline;
		text-underline-offset: 3px;
		overflow-wrap: anywhere;
	}
	.open-error {
		display: block;
		font-size: 12px;
		color: var(--color-error-500);
		white-space: normal;
	}
</style>
