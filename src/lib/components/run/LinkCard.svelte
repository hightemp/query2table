<script lang="ts">
	import type { LinkResult } from '$lib/types';
	import { ExternalLinkIcon } from '@lucide/svelte';

	let { link }: { link: LinkResult } = $props();

	let scorePercent = $derived(
		link.relevance_score != null ? Math.round(link.relevance_score * 100) : null
	);

	let hostname = $derived.by(() => {
		try {
			return new URL(link.url).hostname.replace(/^www\./, '');
		} catch {
			return link.url;
		}
	});
</script>

<div class="link-card">
	<div class="link-head">
		<a class="link-title" href={link.url} target="_blank" rel="noopener noreferrer">
			{link.title || link.url}
			<ExternalLinkIcon size={14} />
		</a>
		{#if scorePercent != null}
			<span class="score" class:high={scorePercent >= 80} class:mid={scorePercent >= 50 && scorePercent < 80}>
				{scorePercent}%
			</span>
		{/if}
	</div>
	<a class="link-url" href={link.url} target="_blank" rel="noopener noreferrer">{hostname}</a>
	{#if link.description}
		<p class="link-desc">{link.description}</p>
	{/if}
</div>

<style>
	.link-card {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 14px 16px;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 8px;
		background: var(--color-surface-100-900);
	}

	.link-head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 12px;
	}

	.link-title {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 1rem;
		font-weight: 600;
		color: var(--color-primary-500);
		text-decoration: none;
		line-height: 1.3;
	}

	.link-title:hover {
		text-decoration: underline;
	}

	.score {
		flex-shrink: 0;
		font-size: 0.78rem;
		font-weight: 700;
		padding: 2px 8px;
		border-radius: 999px;
		background: var(--color-surface-200-800);
		color: var(--color-surface-600-400);
	}

	.score.mid {
		background: rgba(234, 179, 8, 0.15);
		color: rgb(202, 138, 4);
	}

	.score.high {
		background: rgba(34, 197, 94, 0.15);
		color: rgb(22, 163, 74);
	}

	.link-url {
		font-size: 0.8rem;
		color: var(--color-surface-500);
		text-decoration: none;
	}

	.link-url:hover {
		text-decoration: underline;
	}

	.link-desc {
		margin: 0;
		font-size: 0.9rem;
		line-height: 1.45;
		color: var(--color-surface-700-300);
	}
</style>
