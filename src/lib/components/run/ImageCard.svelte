<script lang="ts">
	import type { ImageResult } from '$lib/types';
	import { openUrl } from '@tauri-apps/plugin-opener';

	interface Props {
		image: ImageResult;
		compact?: boolean;
	}

	let { image, compact = false }: Props = $props();

	function handleOpenUrl(url: string) {
		if (url) openUrl(url);
	}

	let dimensions = $derived(
		image.width && image.height ? `${image.width}×${image.height}` : ''
	);
</script>

{#if compact}
	<div class="image-card-compact">
		<img
			src={image.thumbnail_url}
			alt={image.title}
			class="thumb-compact"
			loading="lazy"
			onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
		/>
		<div class="info-compact">
			<span class="title-compact" title={image.title}>{image.title || 'Untitled'}</span>
			{#if dimensions}
				<span class="dim-compact">{dimensions}</span>
			{/if}
			<div class="links-compact">
				<button class="link-btn" onclick={() => handleOpenUrl(image.image_url)}>Image</button>
				{#if image.source_url}
					<button class="link-btn" onclick={() => handleOpenUrl(image.source_url)}>Source</button>
				{/if}
			</div>
		</div>
	</div>
{:else}
	<div class="image-card">
		<div class="thumb-wrapper">
			<img
				src={image.thumbnail_url}
				alt={image.title}
				class="thumb"
				loading="lazy"
				onerror={(e) => { (e.target as HTMLImageElement).style.display = 'none'; }}
			/>
		</div>
		<div class="info">
			<span class="title" title={image.title}>{image.title || 'Untitled'}</span>
			{#if dimensions}
				<span class="dim">{dimensions}</span>
			{/if}
			<div class="links">
				<button class="link-btn" onclick={() => handleOpenUrl(image.image_url)}>Open Image</button>
				{#if image.source_url}
					<button class="link-btn" onclick={() => handleOpenUrl(image.source_url)}>Source</button>
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
		transition: box-shadow 0.15s;
	}

	.image-card:hover {
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
	}

	.thumb-wrapper {
		width: 100%;
		aspect-ratio: 4 / 3;
		overflow: hidden;
		background: var(--color-surface-200-800);
	}

	.thumb {
		width: 100%;
		height: 100%;
		object-fit: cover;
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

	.links {
		display: flex;
		gap: 8px;
		margin-top: 4px;
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
	}

	.thumb-compact {
		width: 80px;
		height: 60px;
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

	.links-compact {
		display: flex;
		gap: 8px;
	}

	.link-btn {
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
