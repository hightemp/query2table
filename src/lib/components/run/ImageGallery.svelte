<script lang="ts">
	import type { ImageResult } from '$lib/types';
	import ImageCard from './ImageCard.svelte';
	import { Grid2x2, List, Image } from '@lucide/svelte';

	interface Props {
		images: ImageResult[];
	}

	let { images }: Props = $props();

	let viewMode = $state<'grid' | 'list'>('grid');
</script>

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
			{#each images as image (image.id)}
				<ImageCard {image} />
			{/each}
		</div>
	{:else}
		<div class="list">
			{#each images as image (image.id)}
				<ImageCard {image} compact />
			{/each}
		</div>
	{/if}
</div>

<style>
	.gallery-wrapper {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.gallery-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
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
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
</style>
