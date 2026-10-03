<script lang="ts">
	import type { ImageResult } from '$lib/types';
	import ImageCard from './ImageCard.svelte';
	import {
		Grid2x2,
		List,
		ChevronLeft,
		ChevronRight,
		SearchIcon,
		CopyIcon,
		DownloadIcon,
		ExternalLinkIcon,
		ImageOffIcon,
		XIcon,
	} from '@lucide/svelte';
	import { open, save } from '@tauri-apps/plugin-dialog';
	import { proxyImage, copyText, saveImage, saveImages } from '$lib/api/tauri';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import EmptyState from '$lib/components/common/EmptyState.svelte';
	import { toast } from '$lib/stores/toasts';
	import { errorText } from '$lib/utils/errors';
	import { debugUi } from '$lib/utils/diagnostics';
	import { persisted } from '$lib/utils/storage';
	import {
		describeImage,
		domainCounts,
		filterImages,
		imageFileName,
		relevanceVaries,
		urlExtension,
		type ImageSort,
		type ImageView,
	} from '$lib/utils/images';

	let { images }: { images: ImageResult[] } = $props();

	const viewMode = persisted<'grid' | 'list'>('q2t-image-view', 'grid', (v) =>
		v === 'grid' || v === 'list' ? v : null
	);
	let search = $state('');
	let domain = $state('');
	let minSide = $state(0);
	let sort = $state<ImageSort>('relevance');
	let selectedIds = $state<string[]>([]);
	let measured = $state<Record<string, number>>({});
	let saving = $state(false);

	let views = $derived(
		images.map((image, index) => {
			const view = describeImage(image, index);
			return view.ratio || !measured[image.id] ? view : { ...view, ratio: measured[image.id] };
		})
	);
	let shown = $derived(filterImages(views, { search, domain, minSide, sort }));
	let domains = $derived(domainCounts(views));
	let showRelevance = $derived(relevanceVaries(views));
	let filtered = $derived(!!search.trim() || !!domain || minSide > 0);
	let selected = $derived(views.filter((view) => selectedIds.includes(view.image.id)));
	// Drop selections of images that are no longer part of the run.
	$effect(() => {
		const ids = new Set(images.map((image) => image.id));
		if (selectedIds.some((id) => !ids.has(id)))
			selectedIds = selectedIds.filter((id) => ids.has(id));
	});

	function toggle(id: string) {
		selectedIds = selectedIds.includes(id)
			? selectedIds.filter((item) => item !== id)
			: [...selectedIds, id];
	}
	function selectShown() {
		selectedIds = [...new Set([...selectedIds, ...shown.map((view) => view.image.id)])];
	}
	function resetFilters() {
		search = '';
		domain = '';
		minSide = 0;
	}

	// Preview follows the displayed order, so arrows step through the filtered images.
	let previewId = $state<string | null>(null);
	let previewIndex = $derived(shown.findIndex((view) => view.image.id === previewId));
	let preview = $derived(previewIndex >= 0 ? shown[previewIndex] : null);
	let source = $state('');
	let fromThumbnail = $state(false);
	let loading = $state(false);
	let failed = $state(false);
	let retry = $state(0);
	$effect(() => {
		const view = preview;
		void retry;
		let current = true;
		source = '';
		failed = false;
		fromThumbnail = false;
		if (!view) {
			loading = false;
			return;
		}
		loading = true;
		void (async () => {
			const candidates = [...new Set([view.original, view.image.thumbnail_url].filter(Boolean))];
			for (const url of candidates as string[]) {
				try {
					const data = await proxyImage(url);
					if (!current) return;
					if (data) {
						source = data;
						fromThumbnail = url !== view.original;
						loading = false;
						return;
					}
				} catch {
					if (!current) return;
				}
			}
			if (current) {
				source = view.image.thumbnail_url || view.original || '';
				fromThumbnail = source !== view.original;
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
		if (previewIndex < 0) return;
		const next = shown[Math.max(0, Math.min(shown.length - 1, previewIndex + delta))];
		if (next) previewId = next.image.id;
	}

	async function copyLink(view: ImageView) {
		const url = view.original ?? view.page ?? view.image.thumbnail_url;
		try {
			await copyText(url);
			toast(view.original ? 'Image link copied.' : 'Page link copied.', 'success');
		} catch {
			toast('Could not copy the link.', 'error');
		}
	}
	async function copyLinks(list: ImageView[]) {
		try {
			await copyText(list.map((view) => view.original ?? view.page ?? view.image.thumbnail_url).join('\n'));
			toast(`Copied ${list.length} ${list.length === 1 ? 'link' : 'links'}.`, 'success');
		} catch {
			toast('Could not copy the links.', 'error');
		}
	}
	async function saveOne(view: ImageView) {
		if (saving) return;
		const extension = urlExtension(view.original) ?? 'jpg';
		try {
			const path = await save({
				defaultPath: `${imageFileName(view)}.${extension}`,
				filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif', 'avif', 'svg'] }],
			});
			if (!path) return;
			saving = true;
			const written = await saveImage(
				view.original ?? view.image.thumbnail_url,
				view.image.thumbnail_url || null,
				path
			);
			toast(`Saved ${written}`, 'success');
		} catch (error) {
			toast(`Could not save the image: ${errorText(error)}`, 'error');
		} finally {
			saving = false;
		}
	}
	async function saveSelected() {
		if (saving || !selected.length) return;
		try {
			const directory = await open({
				directory: true,
				title: `Choose a folder for ${selected.length} ${selected.length === 1 ? 'image' : 'images'}`,
			});
			if (!directory || Array.isArray(directory)) return;
			saving = true;
			const result = await saveImages(
				selected.map((view) => ({
					url: view.original ?? view.image.thumbnail_url,
					fallback_url: view.image.thumbnail_url || null,
					name: imageFileName(view),
				})),
				directory
			);
			if (result.failed.length)
				toast(
					`Saved ${result.saved.length} of ${selected.length} images. Not saved: ${result.failed
						.map(([name]) => name)
						.join(', ')}`,
					'error'
				);
			else toast(`Saved ${result.saved.length} images to ${directory}`, 'success');
		} catch (error) {
			toast(`Could not save the images: ${errorText(error)}`, 'error');
		} finally {
			saving = false;
		}
	}
	function size(view: ImageView) {
		return view.image.width && view.image.height
			? `${view.image.width} × ${view.image.height}`
			: '—';
	}
</script>

<svelte:window
	onkeydown={(event) => {
		if (preview && ['ArrowLeft', 'ArrowRight'].includes(event.key)) {
			event.preventDefault();
			move(event.key === 'ArrowLeft' ? -1 : 1);
		}
	}}
/>
<div class="gallery-wrapper">
	{#if images.length}
		<div class="toolbar">
			<label class="search"
				><SearchIcon size={16} /><input
					class="input sm"
					type="search"
					placeholder="Search titles and sites…"
					aria-label="Search images"
					bind:value={search}
				/></label
			>
			<select class="input sm" aria-label="Sort images" bind:value={sort}>
				<option value="relevance">Best match</option>
				<option value="size">Largest first</option>
				<option value="found">Found order</option>
			</select>
			{#if domains.length > 1}<select class="input sm" aria-label="Source site" bind:value={domain}>
					<option value="">All sites</option>
					{#each domains as [name, count]}<option value={name}>{name} ({count})</option>{/each}
				</select>{/if}
			<select class="input sm" aria-label="Minimum size" bind:value={minSide}>
				<option value={0}>Any size</option>
				<option value={1000}>≥ 1000 px</option>
				<option value={2000}>≥ 2000 px</option>
			</select>
			<span class="count" role="status"
				>{filtered ? `${shown.length} of ${images.length}` : images.length}
				{images.length === 1 ? 'image' : 'images'}</span
			>
			<div class="view-toggle">
				<button
					class="icon-button ghost sm"
					aria-label="Grid view"
					aria-pressed={$viewMode === 'grid'}
					onclick={() => viewMode.set('grid')}><Grid2x2 size={16} /></button
				><button
					class="icon-button ghost sm"
					aria-label="List view"
					aria-pressed={$viewMode === 'list'}
					onclick={() => viewMode.set('list')}><List size={16} /></button
				>
			</div>
		</div>
		{#if selected.length}
			<div class="selection-bar" role="region" aria-label="Selected images">
				<strong>{selected.length} selected</strong>
				{#if shown.some((view) => !selectedIds.includes(view.image.id))}<button
						class="button ghost sm"
						onclick={selectShown}>Select all shown</button
					>{/if}
				<span class="spacer"></span>
				<button class="button sm" onclick={() => copyLinks(selected)}
					><CopyIcon size={14} />Copy links</button
				>
				<button class="button sm primary" disabled={saving} onclick={saveSelected}
					><DownloadIcon size={14} />{saving ? 'Saving…' : 'Save to folder…'}</button
				>
				<button class="icon-button ghost sm" aria-label="Clear selection" onclick={() => (selectedIds = [])}
					><XIcon size={14} /></button
				>
			</div>
		{/if}
	{/if}
	<div class="gallery-scroll">
		{#if !images.length}<EmptyState>No images found yet.</EmptyState>
		{:else if !shown.length}<EmptyState>
				No images match these filters.
				{#snippet action()}<button class="button sm" onclick={resetFilters}>Reset filters</button
					>{/snippet}
			</EmptyState>
		{:else if $viewMode === 'grid'}
			<div class="justified" role="list" aria-label="Images">
				{#each shown as view (view.image.id)}<ImageCard
						{view}
						{showRelevance}
						selected={selectedIds.includes(view.image.id)}
						selecting={selected.length > 0}
						onpreview={() => (previewId = view.image.id)}
						ontoggle={() => toggle(view.image.id)}
						onratio={(ratio) => (measured[view.image.id] = Math.min(3, Math.max(0.4, ratio)))}
					/>{/each}
			</div>
		{:else}
			<table class="image-list">
				<thead>
					<tr>
						<th class="check-cell"><span class="sr-only">Selected</span></th>
						<th><span class="sr-only">Thumbnail</span></th>
						<th>Title</th>
						<th>Site</th>
						<th class="numeric">Size</th>
						{#if showRelevance}<th class="numeric">Match</th>{/if}
						<th><span class="sr-only">Actions</span></th>
					</tr>
				</thead>
				<tbody>
					{#each shown as view (view.image.id)}
						<tr class:selected={selectedIds.includes(view.image.id)}>
							<td class="check-cell"
								><input
									type="checkbox"
									checked={selectedIds.includes(view.image.id)}
									onchange={() => toggle(view.image.id)}
									aria-label={`Select ${view.title}`}
								/></td
							>
							<td class="thumb-cell"
								><button
									class="thumb"
									aria-label={`Preview ${view.title}`}
									onclick={() => (previewId = view.image.id)}
									>{#if view.image.thumbnail_url}<img
											src={view.image.thumbnail_url}
											alt=""
											loading="lazy"
										/>{:else}<ImageOffIcon size={18} />{/if}</button
								></td
							>
							<td class="title-cell" title={view.title}>{view.title}</td>
							<td class="muted">{view.domain}</td>
							<td class="numeric muted">{size(view)}</td>
							{#if showRelevance}<td class="numeric muted"
									>{view.image.relevance_score === null
										? '—'
										: `${Math.round(view.image.relevance_score * 100)}%`}</td
								>{/if}
							<td class="actions"
								><button
									class="icon-button ghost sm"
									aria-label={`Copy link of ${view.title}`}
									title="Copy link"
									onclick={() => copyLink(view)}><CopyIcon size={14} /></button
								>{#if view.page}<ExternalLink href={view.page}
										><ExternalLinkIcon size={14} /><span class="sr-only"
											>Open source page of {view.title}</span
										></ExternalLink
									>{/if}</td
							>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</div>
</div>
{#if preview}
	<Dialog
		title={preview.title}
		variant="image"
		onclose={() => {
			previewId = null;
		}}
	>
		<div class="preview-body">
			{#if loading}<p role="status">Loading image…</p>{:else if failed}<EmptyState role="status">
					This image could not be displayed. Try again or open the source page.
					{#snippet action()}<button
							class="button"
							onclick={() => {
								retry++;
							}}>Retry image</button
						>{/snippet}
				</EmptyState>{:else if source}<img
					src={source}
					alt={preview.title}
					onerror={() => {
						failed = true;
						debugUi('image_preview_decode_failed');
					}}
				/>{/if}
		</div>
		<div class="preview-meta">
			<span>{[preview.domain, size(preview)].filter((part) => part && part !== '—').join(' · ')}</span>
			{#if !loading && !failed && fromThumbnail}<span class="thumbnail-note"
					>{preview.original
						? 'The original could not be loaded; showing the search thumbnail.'
						: 'Showing the search thumbnail; the original file is not known for this result.'}</span
				>{/if}
		</div>
		{#snippet footer()}
			<div class="preview-links">
				{#if preview.page}<ExternalLink href={preview.page} label="Source page" />{/if}
				{#if preview.original}<ExternalLink href={preview.original} label="Open original" />{/if}
			</div>
			<button class="button sm" onclick={() => copyLink(preview!)}
				><CopyIcon size={14} />Copy link</button
			>
			<button class="button sm" disabled={saving} onclick={() => saveOne(preview!)}
				><DownloadIcon size={14} />{saving ? 'Saving…' : 'Save image…'}</button
			>
			<span class="muted">{previewIndex + 1} / {shown.length}</span>
			<button
				class="icon-button"
				aria-label="Previous image"
				disabled={previewIndex === 0}
				onclick={() => move(-1)}><ChevronLeft size={18} /></button
			><button
				class="icon-button"
				aria-label="Next image"
				disabled={previewIndex === shown.length - 1}
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
	.toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		padding-bottom: 10px;
		flex-shrink: 0;
	}
	.toolbar select {
		width: auto;
		max-width: 220px;
	}
	.search {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--app-muted);
	}
	.search input {
		width: min(260px, 30vw);
	}
	.count {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.view-toggle {
		display: flex;
		gap: 2px;
		margin-left: auto;
	}
	.selection-bar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		margin-bottom: 10px;
		padding: 6px 8px 6px 12px;
		border: 1px solid color-mix(in srgb, var(--app-accent) 40%, var(--app-border));
		border-radius: var(--app-radius);
		background: color-mix(in srgb, var(--app-accent) 8%, var(--app-panel));
		font-size: var(--app-text-md);
		flex-shrink: 0;
	}
	.selection-bar .spacer {
		flex: 1;
	}
	.gallery-scroll {
		flex: 1;
		min-height: 0;
		overflow: auto;
		scrollbar-gutter: stable;
		padding: 0 12px 16px 0;
		container-type: inline-size;
	}
	/* Justified rows: every image keeps its proportions and a row shares one height. */
	.justified {
		--row-height: 160px;
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.justified::after {
		content: '';
		flex-grow: 1000000;
	}
	/* Narrow windows fit more images per row instead of stretching one across the width. */
	@container (max-width: 760px) {
		.justified {
			--row-height: 120px;
		}
	}
	@container (min-width: 1300px) {
		.justified {
			--row-height: 200px;
		}
	}
	.image-list {
		width: 100%;
		border-collapse: collapse;
		font-size: var(--app-text-md);
	}
	.image-list th {
		position: sticky;
		top: 0;
		z-index: 1;
		padding: 8px;
		background: var(--app-bg);
		border-bottom: 1px solid var(--app-border);
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		font-weight: 600;
		text-align: left;
	}
	.image-list td {
		padding: 6px 8px;
		border-bottom: 1px solid var(--app-border);
		vertical-align: middle;
	}
	.image-list tr.selected td {
		background: color-mix(in srgb, var(--app-accent) 8%, var(--app-panel));
	}
	.image-list .numeric {
		text-align: right;
		white-space: nowrap;
		font-variant-numeric: tabular-nums;
	}
	.check-cell {
		width: 32px;
	}
	.thumb-cell {
		width: 96px;
	}
	.thumb {
		display: grid;
		place-items: center;
		width: 80px;
		height: 54px;
		padding: 0;
		border-radius: var(--app-radius-sm);
		background: var(--app-subtle);
		overflow: hidden;
		color: var(--app-muted);
	}
	.thumb img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.title-cell {
		max-width: 0;
		width: 50%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-weight: 550;
	}
	.actions {
		white-space: nowrap;
		text-align: right;
	}
	.actions :global(a) {
		display: inline-grid;
		place-items: center;
		width: 28px;
		height: 28px;
		vertical-align: middle;
	}
	.preview-body {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 180px;
		background: var(--app-bg);
		border-radius: var(--app-radius);
	}
	.preview-body img {
		max-width: 100%;
		max-height: 62vh;
		object-fit: contain;
	}
	.preview-meta {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 12px;
		margin-top: 10px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.thumbnail-note {
		color: var(--app-warning);
	}
	.preview-links {
		margin-right: auto;
		display: flex;
		gap: 12px;
		flex-wrap: wrap;
		font-size: var(--app-text-md);
	}
</style>
