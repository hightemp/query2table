<script lang="ts">
	import type { ImageResult } from '$lib/types';
	import ImageCard from './ImageCard.svelte';
	import { Grid2x2, List, Image, X, ExternalLink, ChevronLeft, ChevronRight } from '@lucide/svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';

	interface Props {
		images: ImageResult[];
	}

	let { images }: Props = $props();

	let viewMode = $state<'grid' | 'list'>('grid');

	// Preview modal state
	let previewIndex = $state<number | null>(null);
	let previewImage = $derived(previewIndex !== null ? images[previewIndex] : null);
	let previewImgError = $state(false);

	function openPreview(index: number) {
		previewIndex = index;
		previewImgError = false;
	}

	function closePreview() {
		previewIndex = null;
	}

	function prevImage() {
		if (previewIndex !== null && previewIndex > 0) {
			previewIndex--;
			previewImgError = false;
		}
	}

	function nextImage() {
		if (previewIndex !== null && previewIndex < images.length - 1) {
			previewIndex++;
			previewImgError = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (previewIndex === null) return;
		if (e.key === 'Escape') closePreview();
		else if (e.key === 'ArrowLeft') prevImage();
		else if (e.key === 'ArrowRight') nextImage();
	}

	function handleOpenUrl(url: string) {
		if (url) openUrl(url);
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="gallery-wrapper">
	<div class="gallery-header">
		<span class="count">
			<Image size={16} />
			{images.length} image{images.length !== 1 ? 's' : ''}
		</span>
		<div class="view-toggle">
			<button
				class="toggle-btn"
				class:active={viewMode === 'grid'}
				onclick={() => (viewMode = 'grid')}
				title="Grid view"
			>
				<Grid2x2 size={16} />
			</button>
			<button
				class="toggle-btn"
				class:active={viewMode === 'list'}
				onclick={() => (viewMode = 'list')}
				title="List view"
			>
				<List size={16} />
			</button>
		</div>
	</div>

	{#if images.length === 0}
		<div class="empty">No images found yet.</div>
	{:else if viewMode === 'grid'}
		<div class="grid">
			{#each images as image, i (image.id)}
				<ImageCard {image} onclick={() => openPreview(i)} />
			{/each}
		</div>
	{:else}
		<div class="list">
			{#each images as image, i (image.id)}
				<ImageCard {image} compact onclick={() => openPreview(i)} />
			{/each}
		</div>
	{/if}
</div>

<!-- Preview Modal -->
{#if previewImage}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="preview-overlay" onclick={closePreview}>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="preview-container" onclick={(e) => e.stopPropagation()}>
			<div class="preview-header">
				<span class="preview-title" title={previewImage.title}>
					{previewImage.title || 'Untitled'}
				</span>
				<div class="preview-actions">
					<button class="preview-btn" onclick={() => handleOpenUrl(previewImage!.image_url)} title="Open full image">
						<ExternalLink size={16} />
						Image
					</button>
					{#if previewImage.source_url}
						<button class="preview-btn" onclick={() => handleOpenUrl(previewImage!.source_url)} title="Open source page">
							<ExternalLink size={16} />
							Source
						</button>
					{/if}
					<button class="preview-close" onclick={closePreview} title="Close (Esc)">
						<X size={20} />
					</button>
				</div>
			</div>

			<div class="preview-body">
				{#if previewIndex !== null && previewIndex > 0}
					<button class="nav-btn nav-prev" onclick={prevImage} title="Previous (←)">
						<ChevronLeft size={28} />
					</button>
				{/if}

				<div class="preview-image-wrapper">
					{#if !previewImgError}
						<img
							src={previewImage.image_url}
							alt={previewImage.title}
							class="preview-img"
							onerror={() => { previewImgError = true; }}
						/>
					{:else}
						<div class="preview-img-error">Failed to load image</div>
					{/if}
				</div>

				{#if previewIndex !== null && previewIndex < images.length - 1}
					<button class="nav-btn nav-next" onclick={nextImage} title="Next (→)">
						<ChevronRight size={28} />
					</button>
				{/if}
			</div>

			<div class="preview-footer">
				{#if previewImage.width && previewImage.height}
					<span class="preview-dim">{previewImage.width}×{previewImage.height}</span>
				{/if}
				{#if previewImage.relevance_score !== null}
					<span class="preview-score">Score: {(previewImage.relevance_score * 100).toFixed(0)}%</span>
				{/if}
				<span class="preview-counter">
					{(previewIndex ?? 0) + 1} / {images.length}
				</span>
			</div>
		</div>
	</div>
{/if}

<style>
	.gallery-wrapper {
		display: flex;
		flex-direction: column;
		gap: 12px;
		min-height: 0;
		flex: 1;
	}

	.gallery-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex-shrink: 0;
	}

	.count {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 0.9rem;
		color: var(--color-surface-600-400);
	}

	.view-toggle {
		display: flex;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 6px;
		overflow: hidden;
	}

	.toggle-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 6px 10px;
		border: none;
		background: var(--color-surface-100-900);
		cursor: pointer;
		color: var(--color-surface-500);
		transition: all 0.15s;
	}

	.toggle-btn:not(:last-child) {
		border-right: 1px solid var(--color-surface-300-700);
	}

	.toggle-btn.active {
		background: var(--color-primary-500);
		color: white;
	}

	.empty {
		text-align: center;
		padding: 40px 16px;
		color: var(--color-surface-500);
		font-size: 0.9rem;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 12px;
		overflow-y: auto;
		min-height: 0;
		flex: 1;
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		overflow-y: auto;
		min-height: 0;
		flex: 1;
	}

	/* Preview Modal */
	.preview-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.85);
		z-index: 1000;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.preview-container {
		display: flex;
		flex-direction: column;
		width: 90vw;
		height: 90vh;
		max-width: 1200px;
		background: var(--color-surface-100-900);
		border-radius: 12px;
		overflow: hidden;
	}

	.preview-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 16px;
		border-bottom: 1px solid var(--color-surface-300-700);
		flex-shrink: 0;
		gap: 12px;
	}

	.preview-title {
		font-size: 0.9rem;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		min-width: 0;
		flex: 1;
	}

	.preview-actions {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-shrink: 0;
	}

	.preview-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 6px 10px;
		font-size: 0.8rem;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 6px;
		background: var(--color-surface-200-800);
		color: var(--color-surface-700-300);
		cursor: pointer;
		transition: background 0.15s;
	}

	.preview-btn:hover {
		background: var(--color-surface-300-700);
	}

	.preview-close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		padding: 0;
		border: none;
		border-radius: 6px;
		background: transparent;
		color: var(--color-surface-500);
		cursor: pointer;
		transition: background 0.15s, color 0.15s;
	}

	.preview-close:hover {
		background: var(--color-surface-200-800);
		color: var(--color-surface-700-300);
	}

	.preview-body {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 0;
		position: relative;
		padding: 8px;
		gap: 8px;
	}

	.preview-image-wrapper {
		flex: 1;
		min-width: 0;
		min-height: 0;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.preview-img {
		max-width: 100%;
		max-height: 100%;
		object-fit: contain;
		border-radius: 4px;
	}

	.preview-img-error {
		color: var(--color-surface-500);
		font-size: 0.9rem;
	}

	.nav-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border: none;
		border-radius: 50%;
		background: var(--color-surface-200-800);
		color: var(--color-surface-700-300);
		cursor: pointer;
		flex-shrink: 0;
		transition: background 0.15s;
	}

	.nav-btn:hover {
		background: var(--color-surface-300-700);
	}

	.preview-footer {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 8px 16px;
		border-top: 1px solid var(--color-surface-300-700);
		font-size: 0.8rem;
		color: var(--color-surface-500);
		flex-shrink: 0;
	}

	.preview-counter {
		margin-left: auto;
	}
</style>
