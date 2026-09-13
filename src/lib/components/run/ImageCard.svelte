<script lang="ts">
	import type { ImageResult } from '$lib/types';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	let {
		image,
		compact = false,
		onclick,
	}: { image: ImageResult; compact?: boolean; onclick?: () => void } = $props();
	let failed = $state(false);
	$effect(() => {
		void image.thumbnail_url;
		failed = false;
	});
</script>

<article class="image-card" class:compact>
	<button class="preview-button" {onclick} aria-label={`Preview ${image.title || 'image'}`}>
		{#if failed}<span class="placeholder">Preview unavailable</span>{:else}<img
				src={image.thumbnail_url || image.image_url}
				alt=""
				loading="lazy"
				onerror={() => {
					failed = true;
				}}
			/>{/if}
	</button>
	<div class="image-info">
		<button class="image-title" {onclick}>{image.title || 'Untitled image'}</button>
		<div class="metadata">
			{#if image.width && image.height}<span>{image.width} × {image.height}</span
				>{/if}{#if image.relevance_score !== null}<span
					>Relevance {Math.round(image.relevance_score * 100)}%</span
				>{/if}
		</div>
		<div class="image-links">
			<ExternalLink
				href={image.image_url}
				label="Original image"
			/>{#if image.source_url}<ExternalLink href={image.source_url} label="Source page" />{/if}
		</div>
	</div>
</article>

<style>
	.image-card {
		min-width: 0;
		background: var(--app-panel);
		border: 1px solid var(--app-border);
		border-radius: 10px;
		overflow: hidden;
	}
	.preview-button {
		display: block;
		width: 100%;
		border: 0;
		padding: 0;
		background: var(--app-subtle);
		aspect-ratio: 16 / 10;
	}
	img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.placeholder {
		color: var(--app-muted);
		font-size: 13px;
	}
	.image-info {
		padding: 12px;
		min-width: 0;
	}
	.image-title {
		display: block;
		border: 0;
		background: transparent;
		color: var(--app-text);
		text-align: left;
		padding: 0;
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	.metadata,
	.image-links {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 12px;
		font-size: 12px;
		margin-top: 8px;
		color: var(--app-muted);
	}
	.compact {
		display: flex;
		align-items: center;
	}
	.compact .preview-button {
		flex: 0 0 130px;
		width: 130px;
	}
</style>
