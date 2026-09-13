<script lang="ts">
	import type { LinkResult } from '$lib/types';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import CopyButton from '$lib/components/common/CopyButton.svelte';
	import { urlLabel } from '$lib/utils/values';
	let { link }: { link: LinkResult } = $props();
</script>

<article class="link-card">
	<header>
		<h3><ExternalLink href={link.url} label={link.title || link.url} /></h3>
		<CopyButton text={link.url} label="Copy link URL" />
	</header>
	<div class="metadata">
		<span title={link.url}>{urlLabel(link.url)}</span>{#if link.relevance_score !== null}<span
				>Relevance {Math.round(link.relevance_score * 100)}%</span
			>{/if}
	</div>
	{#if link.description}<p>{link.description}</p>{/if}
</article>

<style>
	.link-card {
		padding: 16px;
		border: 1px solid var(--app-border);
		border-radius: 10px;
		background: var(--app-panel);
		min-width: 0;
		overflow-wrap: anywhere;
	}
	header {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		align-items: flex-start;
	}
	h3 {
		min-width: 0;
		font-size: 15px;
		font-weight: 600;
		margin: 0;
	}
	.metadata {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 16px;
		color: var(--app-muted);
		font-size: 12px;
		margin: 4px 0 8px;
	}
	p {
		margin: 0;
		white-space: pre-wrap;
		line-height: 1.6;
	}
</style>
