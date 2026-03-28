<script lang="ts">
	import type { ImageResult } from '$lib/types';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { ExternalLink } from '@lucide/svelte';

	interface Props {
		image: ImageResult;
		compact?: boolean;
		onclick?: () => void;
	}

	let { image, compact = false, onclick }: Props = $props();

	let dimensions = $derived(
		image.width && image.height ? `${image.width}×${image.height}` : ''
	);

	let scorePercent = $derived(
		image.relevance_score !== null && image.relevance_score !== undefined
			? Math.round(image.relevance_score * 100)
			: null
	);

	let imgError = $state(false);

	function handleLink(e: MouseEvent, url: string) {
		e.stopPropagation();
		if (url) openUrl(url);
	}
</script>

{#if compact}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="image-card-compact" onclick={onclick}>
		{#if !imgError}
			<img
				src={image.thumbnail_url || image.image_url}
				alt={image.title}
				class="thumb-compact"
				loading="lazy"
				onerror={() => { imgError = true; }}
			/>
		{:else}
			<div class="thumb-compact thumb-placeholder">No img</div>
		{/if}
		<div class="info-compact">
			<span class="title-compact" title={image.title}>{image.title || 'Untitled'}</span>
			<div class="meta-compact">
				{#if dimensions}
					<span class="dim-compact">{dimensions}</span>
				{/if}
				{#if scorePercent !== null}
					<span class="score-compact">{scorePercent}%</span>
				{/if}
			</div>
			<div class="links-compact">
				<button class="link-btn" onclick={(e) => handleLink(e, image.image_url)}>
					<ExternalLink size={12} /> Image
				</button>
				{#if image.source_url}
					<button class="link-btn" onclick={(e) => handleLink(e, image.source_url)}>
						<ExternalLink size={12} /> Source
					</button>
				{/if}
			</div>
		</div>
	</div>
{:else}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="image-card" onclick={onclick}>
		<div class="thumb-wrapper">
			{#if !imgError}
				<img
					src={image.thumbnail_url || image.image_url}
					alt={image.title}
					class="thumb"
					loading="lazy"
					onerror={() => { imgError = true; }}
				/>
			{:else}
				<div class="thumb-placeholder">No image</div>
			{/if}
		</div>
		<div class="info">
			<span class="title" title={image.title}>{image.title || 'Untitled'}</span>
			<div class="meta-row">
				{#if dimensions}
					<span class="dim">{dimensions}</span>
				{/if}
				{#if scorePercent !== null}
					<span class="score">{scorePercent}%</span>
				{/if}
			</div>
			<div class="links">
				<button class="link-btn" onclick={(e) => handleLink(e, image.image_url)}>
					<ExternalLink size={12} /> Image
				</button>
				{#if image.source_url}
					<button class="link-btn" onclick={(e) => handleLink(e, image.source_url)}>
						<ExternalLink size={12} /> Source
					</button>
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.image-card {
		border: 1px solid var(--color-surface-300-700);
		border-radius: 8px;
		overflow: hidden;
		background: var(--color-surface-100-900);
		transition: box-shadow 0.15s, transform 0.15s;
		cursor: pointer;
		display: flex;
		flex-direction: column;
	}

	.image-card:hover {
		box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
		transform: translateY(-1px);
	}

	.thumb-wrapper {
		width: 100%;
		height: 200px;
		overflow: hidden;
		background: var(--color-surface-200-800);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.thumb {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.thumb-placeholder {
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 0.8rem;
		color: var(--color-surface-500);
		background: var(--color-surface-200-800);
	}

	.info {
		padding: 8px 10px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.title {
		font-size: 0.85rem;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.dim {
		font-size: 0.75rem;
		color: var(--color-surface-500);
	}

	.meta-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.score {
		font-size: 0.75rem;
		color: var(--color-primary-500);
		font-weight: 600;
	}

	.links {
		display: flex;
		gap: 8px;
		margin-top: 2px;
	}

	/* Compact (list) styles */
	.image-card-compact {
		display: flex;
		align-items: center;
		gap: 12px;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 8px;
		padding: 8px;
		background: var(--color-surface-100-900);
		cursor: pointer;
		transition: background 0.15s;
	}

	.image-card-compact:hover {
		background: var(--color-surface-200-800);
	}

	.thumb-compact {
		width: 100px;
		height: 75px;
		object-fit: cover;
		border-radius: 4px;
		flex-shrink: 0;
		background: var(--color-surface-200-800);
	}

	.info-compact {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}

	.title-compact {
		font-size: 0.85rem;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.dim-compact {
		font-size: 0.75rem;
		color: var(--color-surface-500);
	}

	.meta-compact {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.score-compact {
		font-size: 0.75rem;
		color: var(--color-primary-500);
		font-weight: 600;
	}

	.links-compact {
		display: flex;
		gap: 8px;
	}

	.link-btn {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		font-size: 0.75rem;
		color: var(--color-primary-500);
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
	}

	.link-btn:hover {
		text-decoration: underline;
	}
</style>
