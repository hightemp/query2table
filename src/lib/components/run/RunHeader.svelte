<script lang="ts">
	import { t } from '$lib/i18n';
	import type { Snippet } from 'svelte';

	let {
		eyebrow,
		title,
		subtitle = null,
		level = 2,
		heading,
		meta,
		actions,
	}: {
		/** The kind of run, shown above the title. */
		eyebrow: string;
		title: string;
		/** Shown under the title, e.g. the query of a renamed run. */
		subtitle?: string | null;
		level?: 1 | 2;
		/** Replaces the title, e.g. with a rename field. */
		heading?: Snippet;
		meta?: Snippet;
		actions?: Snippet;
	} = $props();
	let expanded = $state(false);
</script>

<header class="run-header">
	<div class="run-query-display">
		<span class="eyebrow">{eyebrow}</span>
		{#if heading}{@render heading()}{:else}<svelte:element
				this={`h${level}`}
				class="run-title"
				class:expanded>{title}</svelte:element
			>{/if}
		{#if title.length > 90 && !heading}<button
				class="query-expand"
				onclick={() => (expanded = !expanded)}
				aria-expanded={expanded}>{expanded ? t('common.showLess') : t('header.showFull')}</button
			>{/if}
		{#if subtitle}<p class="subtitle">{subtitle}</p>{/if}
		{#if meta}<div class="meta">{@render meta()}</div>{/if}
	</div>
	{#if actions}<div class="actions">{@render actions()}</div>{/if}
</header>

<style>
	.run-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		flex-wrap: wrap;
		gap: 12px 20px;
		flex-shrink: 0;
		padding-bottom: 12px;
	}
	.run-query-display {
		flex: 1 1 240px;
		min-width: 0;
	}
	.eyebrow {
		color: var(--app-muted);
		font-size: var(--app-text-xs);
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}
	.run-title {
		font-size: var(--app-text-2xl);
		font-weight: 650;
		line-height: 1.35;
		overflow-wrap: anywhere;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
		margin: 4px 0 0;
	}
	.run-title.expanded {
		display: block;
		max-height: 120px;
		overflow: auto;
	}
	.query-expand {
		font-size: var(--app-text-sm);
		color: var(--app-accent);
		background: transparent;
		padding: 4px 0;
		border: 0;
	}
	.subtitle {
		margin-top: 2px;
		color: var(--app-muted);
		font-size: var(--app-text-md);
		overflow-wrap: anywhere;
	}
	.meta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 4px 12px;
		margin-top: 6px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}
</style>
