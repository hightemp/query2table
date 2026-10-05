<script lang="ts">
	import { t } from '$lib/i18n';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import SiteIcon from './SiteIcon.svelte';
	import type { ResearchSource } from '$lib/utils/research';
	let { sources }: { sources: ResearchSource[] } = $props();
	function host(url: string) {
		try {
			return new URL(url).hostname;
		} catch {
			return '';
		}
	}
</script>

{#if !sources.length}
	<p class="empty">{t('research.noSources')}</p>
{:else}
	<ol class="sources">
		{#each sources as source, i (source.url)}
			<li class="source">
				<span class="number">{i + 1}</span>
				<SiteIcon domain={source.domain} host={host(source.url)} />
				<div class="body">
					<ExternalLink href={source.url} label={source.title} class="source-title" />
					<div class="meta">
						<span class="domain">{source.domain}</span>
						{#if source.cited}<span class="tag cited" title={t('research.citedHint')}>{t('research.cited')}</span>{/if}
						{#if source.read}<span class="tag" title={t('research.readHint')}>{t('research.read')}</span>{/if}
					</div>
				</div>
			</li>
		{/each}
	</ol>
{/if}

<style>
	.empty {
		color: var(--app-muted);
	}
	.sources {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.source {
		display: grid;
		grid-template-columns: 24px 16px minmax(0, 1fr);
		gap: 8px;
		align-items: start;
		padding: 10px 0;
		border-bottom: 1px solid var(--app-border);
	}
	.number {
		color: var(--app-muted);
		text-align: right;
		font-size: var(--app-text-sm);
		font-variant-numeric: tabular-nums;
	}
	.source :global(.site-icon),
	.source :global(.site-letter) {
		margin-top: 3px;
	}
	.body {
		min-width: 0;
	}
	.body :global(.source-title) {
		font-weight: 600;
		text-decoration: none;
	}
	.meta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		margin-top: 2px;
		font-size: var(--app-text-sm);
		color: var(--app-muted);
	}
	.tag {
		padding: 0 7px;
		border-radius: var(--app-radius-pill);
		background: var(--app-subtle);
		font-size: var(--app-text-xs);
		font-weight: 600;
		line-height: 1.7;
	}
	.tag.cited {
		color: var(--app-success);
		background: color-mix(in srgb, var(--app-success) 14%, transparent);
	}
</style>
