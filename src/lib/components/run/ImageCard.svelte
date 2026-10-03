<script lang="ts">
	import { ImageOffIcon, CheckIcon } from '@lucide/svelte';
	import type { ImageView } from '$lib/utils/images';
	import { isMenuKey, menuPointFor } from '$lib/components/common/ContextMenu.svelte';
	let {
		view,
		selected = false,
		selecting = false,
		showRelevance = false,
		onpreview,
		ontoggle,
		onratio,
		onmenu,
	}: {
		view: ImageView;
		selected?: boolean;
		/** Some image is selected, so checkboxes stay visible. */
		selecting?: boolean;
		showRelevance?: boolean;
		onpreview: () => void;
		ontoggle: () => void;
		/** Reports the real proportions when the saved size is unknown. */
		onratio?: (ratio: number) => void;
		/** Opens the image's context menu at a point on screen. */
		onmenu?: (point: { x: number; y: number }) => void;
	} = $props();

	// Thumbnail first; the original when the thumbnail fails; a placeholder after that.
	let attempt = $state(0);
	let sources = $derived(
		[...new Set([view.image.thumbnail_url, view.original].filter(Boolean))] as string[]
	);
	let source = $derived(sources[attempt] ?? null);
	$effect(() => {
		void view.image.id;
		attempt = 0;
	});
	let size = $derived(
		view.image.width && view.image.height ? `${view.image.width} × ${view.image.height}` : null
	);
</script>

<div
	class="tile"
	class:selected
	class:selecting
	style={`--ratio:${view.ratio ?? 1.5}`}
	role="listitem"
	oncontextmenu={(event) => {
		if (!onmenu) return;
		event.preventDefault();
		onmenu({ x: event.clientX, y: event.clientY });
	}}
>
	<button
		class="preview"
		onclick={onpreview}
		aria-label={`Preview ${view.title}`}
		aria-haspopup="menu"
		onkeydown={(event) => {
			if (onmenu && isMenuKey(event)) {
				event.preventDefault();
				onmenu(menuPointFor(event.currentTarget));
			}
		}}
	>
		{#if source}<img
				src={source}
				alt=""
				loading="lazy"
				onload={(event) => {
					const img = event.currentTarget as HTMLImageElement;
					if (!view.ratio && img.naturalWidth && img.naturalHeight)
						onratio?.(img.naturalWidth / img.naturalHeight);
				}}
				onerror={() => {
					attempt += 1;
				}}
			/>{:else}<span class="unavailable"
				><ImageOffIcon size={22} /><span>{view.domain || 'Preview unavailable'}</span></span
			>{/if}
		<span class="caption">
			<span class="title">{view.title}</span>
			<span class="meta"
				>{[view.domain, size, showRelevance && view.image.relevance_score !== null
						? `${Math.round(view.image.relevance_score * 100)}% match`
						: null]
					.filter(Boolean)
					.join(' · ')}</span
			>
		</span>
	</button>
	<label class="check" title={selected ? 'Deselect' : 'Select'}>
		<input
			type="checkbox"
			checked={selected}
			onchange={ontoggle}
			aria-label={`Select ${view.title}`}
		/><span aria-hidden="true"><CheckIcon size={14} /></span>
	</label>
</div>

<style>
	.tile {
		position: relative;
		flex: var(--ratio) 1 calc(var(--ratio) * var(--row-height, 200px));
		aspect-ratio: var(--ratio);
		min-width: 0;
		border-radius: var(--app-radius);
		overflow: hidden;
		background: var(--app-subtle);
	}
	.preview {
		display: block;
		width: 100%;
		height: 100%;
		padding: 0;
	}
	img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.unavailable {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 6px;
		height: 100%;
		padding: 8px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		overflow-wrap: anywhere;
	}
	.caption {
		position: absolute;
		inset: auto 0 0 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 28px 10px 8px;
		background: linear-gradient(transparent, rgb(0 0 0 / 75%));
		color: #fff;
		text-align: left;
		opacity: 0;
		transition: opacity 0.15s;
		pointer-events: none;
	}
	.tile:hover .caption,
	.tile:focus-within .caption {
		opacity: 1;
	}
	.title {
		font-size: var(--app-text-md);
		font-weight: 600;
		line-height: 1.3;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
	.meta {
		font-size: var(--app-text-xs);
		opacity: 0.85;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.check {
		position: absolute;
		top: 8px;
		left: 8px;
		display: grid;
		place-items: center;
		width: 24px;
		height: 24px;
		border: 2px solid #fff;
		border-radius: var(--app-radius-sm);
		background: rgb(0 0 0 / 35%);
		color: #fff;
		cursor: pointer;
		opacity: 0;
		transition: opacity 0.15s;
	}
	.check input {
		position: absolute;
		opacity: 0;
		inset: 0;
		margin: 0;
		cursor: pointer;
	}
	.check span {
		display: none;
	}
	.tile:hover .check,
	.tile:focus-within .check,
	.selecting .check {
		opacity: 1;
	}
	.check:has(input:focus-visible) {
		outline: 2px solid var(--app-accent);
		outline-offset: 2px;
	}
	.selected .check {
		background: var(--app-accent-solid);
		border-color: var(--app-accent-solid);
	}
	.selected .check span {
		display: block;
	}
	.selected {
		outline: 3px solid var(--app-accent-solid);
		outline-offset: -3px;
	}
	@media (prefers-reduced-motion: reduce) {
		.caption,
		.check {
			transition: none;
		}
	}
</style>
