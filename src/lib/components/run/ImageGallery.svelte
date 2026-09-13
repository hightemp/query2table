<script lang="ts">
	import type { ImageResult } from '$lib/types';
	import ImageCard from './ImageCard.svelte';
	import { Grid2x2, List, ChevronLeft, ChevronRight } from '@lucide/svelte';
	import { proxyImage } from '$lib/api/tauri';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import { debugUi } from '$lib/utils/diagnostics';
	let { images }: { images: ImageResult[] } = $props();
	let viewMode = $state<'grid' | 'list'>('grid');
	let previewIndex = $state<number | null>(null);
	let previewImage = $derived(previewIndex !== null ? images[previewIndex] : null);
	let source = $state('');
	let loading = $state(false);
	let failed = $state(false);
	let retry = $state(0);
	$effect(() => {
		const selected = previewImage;
		void retry;
		let current = true;
		source = '';
		failed = false;
		if (!selected) {
			loading = false;
			return;
		}
		loading = true;
		void (async () => {
			for (const url of [
				...new Set([selected.image_url, selected.thumbnail_url].filter(Boolean)),
			]) {
				try {
					const data = await proxyImage(url);
					if (!current) return;
					if (data) {
						source = data;
						loading = false;
						return;
					}
				} catch {
					if (!current) return;
				}
			}
			if (current) {
				source = selected.thumbnail_url || selected.image_url;
				failed = !source;
				loading = false;
				debugUi('image_preview_using_fallback');
			}
		})();
		return () => {
			current = false;
		};
	});
	function move(delta: number) {
		if (previewIndex !== null)
			previewIndex = Math.max(0, Math.min(images.length - 1, previewIndex + delta));
	}
</script>

<svelte:window
	onkeydown={(event) => {
		if (previewIndex !== null && ['ArrowLeft', 'ArrowRight'].includes(event.key)) {
			event.preventDefault();
			move(event.key === 'ArrowLeft' ? -1 : 1);
		}
	}}
/>
<div class="gallery-wrapper">
	<header>
		<span>{images.length} {images.length === 1 ? 'image' : 'images'}</span>
		<div>
			<button
				class="icon-button"
				aria-label="Grid view"
				aria-pressed={viewMode === 'grid'}
				onclick={() => {
					viewMode = 'grid';
				}}><Grid2x2 size={16} /></button
			><button
				class="icon-button"
				aria-label="List view"
				aria-pressed={viewMode === 'list'}
				onclick={() => {
					viewMode = 'list';
				}}><List size={16} /></button
			>
		</div>
	</header>
	<div class="gallery-scroll">
		{#if !images.length}<div class="empty-state">No images found yet.</div>{:else}<div
				class:grid={viewMode === 'grid'}
				class:list={viewMode === 'list'}
			>
				{#each images as image, index (image.id)}<ImageCard
						{image}
						compact={viewMode === 'list'}
						onclick={() => {
							previewIndex = index;
						}}
					/>{/each}
			</div>{/if}
	</div>
</div>
{#if previewImage}
	<Dialog
		title={previewImage.title || 'Image preview'}
		variant="image"
		onclose={() => {
			previewIndex = null;
		}}
	>
		<div class="preview-body">
			{#if loading}<p role="status">Loading image…</p>{:else if failed}<div
					class="empty-state"
					role="status"
				>
					<p>This image could not be displayed. Try again or open the original.</p>
					<button
						class="button"
						onclick={() => {
							retry++;
						}}>Retry image</button
					>
				</div>{:else}<img
					src={source}
					alt={previewImage.title || 'Preview'}
					onerror={() => {
						failed = true;
						debugUi('image_preview_decode_failed');
					}}
				/>{/if}
		</div>
		{#snippet footer()}
			<div class="preview-links">
				<ExternalLink
					href={previewImage.image_url}
					label="Original image"
				/>{#if previewImage.source_url}<ExternalLink
						href={previewImage.source_url}
						label="Source page"
					/>{/if}
			</div>
			<span class="muted">{(previewIndex ?? 0) + 1} / {images.length}</span>
			<button
				class="icon-button"
				aria-label="Previous image"
				disabled={previewIndex === 0}
				onclick={() => move(-1)}><ChevronLeft size={18} /></button
			><button
				class="icon-button"
				aria-label="Next image"
				disabled={previewIndex === images.length - 1}
				onclick={() => move(1)}><ChevronRight size={18} /></button
			>
		{/snippet}
	</Dialog>
{/if}

<style>
	.gallery-wrapper {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
	}
	header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding-bottom: 12px;
		color: var(--app-muted);
		font-size: 13px;
		flex-shrink: 0;
	}
	header div {
		display: flex;
		gap: 4px;
	}
	[aria-pressed='true'] {
		color: var(--app-accent);
		border-color: var(--app-accent);
	}
	.gallery-scroll {
		flex: 1;
		min-height: 0;
		overflow: auto;
		scrollbar-gutter: stable;
		padding: 0 12px 16px 0;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 16px;
	}
	.list {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.preview-body {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 180px;
		background: var(--app-bg);
		border-radius: 8px;
	}
	.preview-body img {
		max-width: 100%;
		max-height: 65vh;
		object-fit: contain;
	}
	.preview-links {
		margin-right: auto;
		display: flex;
		gap: 12px;
		flex-wrap: wrap;
		font-size: 13px;
	}
</style>
