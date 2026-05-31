<script lang="ts">
	import type { LinkResult } from '$lib/types';
	import LinkCard from './LinkCard.svelte';

	let { links }: { links: LinkResult[] } = $props();
</script>

{#if links.length === 0}
	<div class="link-empty">No relevant links yet.</div>
{:else}
	<div class="link-list">
		<div class="link-count">{links.length} relevant {links.length === 1 ? 'link' : 'links'}</div>
		{#each links as link (link.id)}
			<LinkCard {link} />
		{/each}
	</div>
{/if}

<style>
	.link-list {
		display: flex;
		flex-direction: column;
		gap: 10px;
		overflow-y: auto;
		padding-bottom: 16px;
	}

	.link-count {
		font-size: 0.85rem;
		font-weight: 600;
		color: var(--color-surface-600-400);
		margin-bottom: 2px;
	}

	.link-empty {
		padding: 24px;
		text-align: center;
		color: var(--color-surface-500);
		font-size: 0.9rem;
	}
</style>
