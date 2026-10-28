<script lang="ts">
	import { tooltip } from '$lib/actions/tooltip';
	import { t } from '$lib/i18n';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import SiteIcon from './SiteIcon.svelte';
	import { FileTextIcon, PlayIcon } from '@lucide/svelte';
	import { showAttachment } from '$lib/stores/attachmentPreview';
	import { formatDuration, type ResearchSource } from '$lib/utils/research';
	let {
		sources,
		loadImage,
	}: {
		sources: ResearchSource[];
		/** The kept copy of a picture as a data URL, for video thumbnails. */
		loadImage?: (url: string) => Promise<string | null>;
	} = $props();
	let pages = $derived(sources.filter((source) => !source.video));
	let videos = $derived(sources.filter((source) => source.video));
	function host(url: string) {
		try {
			return new URL(url).hostname;
		} catch {
			return '';
		}
	}
</script>

{#snippet tags(source: ResearchSource)}
	{#if source.cited}<span class="tag cited" use:tooltip={t('research.citedHint')}>{t('research.cited')}</span>{/if}
	{#if source.read}<span class="tag" use:tooltip={t('research.readHint')}>{t('research.read')}</span>{/if}
{/snippet}

{#if !sources.length}
	<p class="empty">{t('research.noSources')}</p>
{:else}
	{#if pages.length}
		<ol class="sources">
			{#each pages as source, i (source.url)}
				<li class="source">
					<span class="number">{i + 1}</span>
					{#if source.file}<span class="file-icon"><FileTextIcon size={16} /></span>{:else}<SiteIcon domain={source.domain} host={host(source.url)} />{/if}
					<div class="body">
						{#if source.file}<button class="file-title" onclick={() => showAttachment(source.url)}>{source.title}</button
							>{:else}<ExternalLink href={source.url} label={source.title} class="source-title" />{/if}
						<div class="meta">
							<span class="domain">{source.domain}</span>
							{@render tags(source)}
							{#if source.images?.length}<span class="tag" use:tooltip={t('research.imageSourceHint')}
									>{t('research.imageSource', { count: source.images.length })}</span
								>{/if}
						</div>
					</div>
				</li>
			{/each}
		</ol>
	{/if}
	{#if videos.length}
		<h3 class="group">{t('research.videos')}</h3>
		<ol class="sources videos">
			{#each videos as source, i (source.url)}
				{@const thumbnail = source.video?.thumbnail && loadImage ? loadImage(source.video.thumbnail) : null}
				<li class="source video">
					<span class="number">{pages.length + i + 1}</span>
					<span class="thumb" aria-hidden="true">
						{#await thumbnail then src}{#if src}<img {src} alt="" />{/if}{/await}
						<span class="play"><PlayIcon size={14} /></span>
					</span>
					<div class="body">
						<ExternalLink href={source.url} label={source.title} class="source-title" />
						<div class="meta">
							<span class="domain">{[source.video?.channel, source.domain].filter(Boolean).join(' · ')}</span>
							{#if source.video?.duration}<span class="duration">{formatDuration(source.video.duration)}</span>{/if}
							{@render tags(source)}
						</div>
					</div>
				</li>
			{/each}
		</ol>
	{/if}
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
	.group {
		margin: 20px 0 4px;
		font-size: var(--app-text-sm);
		font-weight: 600;
		color: var(--app-muted);
	}
	.source {
		display: grid;
		grid-template-columns: 24px 16px minmax(0, 1fr);
		gap: 8px;
		align-items: start;
		padding: 10px 0;
		border-bottom: 1px solid var(--app-border);
	}
	.source.video {
		grid-template-columns: 24px 96px minmax(0, 1fr);
		gap: 10px;
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
	.thumb {
		position: relative;
		display: block;
		aspect-ratio: 16 / 9;
		overflow: hidden;
		border: 1px solid var(--app-border);
		border-radius: 6px;
		background: var(--app-subtle);
	}
	.thumb img {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.play {
		position: absolute;
		top: 50%;
		left: 50%;
		display: grid;
		place-items: center;
		width: 24px;
		height: 24px;
		border-radius: 50%;
		transform: translate(-50%, -50%);
		background: color-mix(in srgb, var(--app-bg) 82%, transparent);
		color: var(--app-text);
	}
	.body {
		min-width: 0;
	}
	.file-icon {
		display: flex;
		margin-top: 3px;
		color: var(--app-muted);
	}
	.file-title {
		padding: 0;
		border: 0;
		background: none;
		color: var(--app-accent);
		font-weight: 600;
		text-align: left;
	}
	.file-title:hover {
		text-decoration: underline;
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
	.duration {
		font-variant-numeric: tabular-nums;
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
