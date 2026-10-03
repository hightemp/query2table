<script lang="ts">
	import type { LinkResult } from '$lib/types';
	import LinkCard from './LinkCard.svelte';
	import EmptyState from '$lib/components/common/EmptyState.svelte';

	let { links }: { links: LinkResult[] } = $props();
</script>

{#if links.length === 0}
	<EmptyState>No relevant links yet.</EmptyState>
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
		min-height: 0;
		flex: 1;
		scrollbar-gutter: stable;
		padding-right: 12px;
		padding-bottom: 16px;
	}

	.link-count {
		font-size: var(--app-text-md);
		color: var(--app-muted);
		margin-bottom: 2px;
	}
</style>
