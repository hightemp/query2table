<script lang="ts">
	import { t } from '$lib/i18n';
	import { tick } from 'svelte';
	import type { LinkResult } from '$lib/types';
	import LinkCard from './LinkCard.svelte';
	import SiteIcon from './SiteIcon.svelte';
	import EmptyState from '$lib/components/common/EmptyState.svelte';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import type { MenuItem } from '$lib/components/common/ContextMenu.svelte';
	import { menuPointFor } from '$lib/components/common/ContextMenu.svelte';
	import {
		SearchIcon,
		ExternalLinkIcon,
		CopyIcon,
		LinkIcon,
		EyeIcon,
		EyeOffIcon,
		SquareCheckIcon,
		SquareIcon,
		XIcon,
		ChevronDownIcon,
		ClipboardCopyIcon,
	} from '@lucide/svelte';
	import { setLinkHidden, setLinkVisited } from '$lib/api/tauri';
	import { showContextMenu } from '$lib/stores/contextMenu';
	import { toast } from '$lib/stores/toasts';
	import { copyWithToast, markdownLink, openLink } from '$lib/utils/linkMenu';
	import { persisted } from '$lib/utils/storage';
	import { errorText } from '$lib/utils/errors';
	import {
		COPY_FORMATS,
		copyFormatLabel,
		describeLinks,
		filterLinks,
		formatLinks,
		groupBySite,
		siteCounts,
		type CopyFormat,
		type LinkSort,
		type LinkView,
	} from '$lib/utils/linkResults';

	let { links }: { links: LinkResult[] } = $props();

	const OPEN_CONFIRM_LIMIT = 10;
	const groupBySiteSetting = persisted<boolean>('q2t-links-group', false, (v) =>
		typeof v === 'boolean' ? v : null
	);
	let search = $state('');
	let domain = $state('');
	let sort = $state<LinkSort>('relevance');
	let showHidden = $state(false);
	let showLow = $state(false);
	let selectedIds = $state<string[]>([]);
	let confirmOpen = $state<LinkView[] | null>(null);
	// Review state changed in this session, applied over the run's links.
	let overrides = $state<Record<string, Partial<LinkResult>>>({});

	let merged = $derived(links.map((link) => ({ ...link, ...overrides[link.id] })));
	let views = $derived(describeLinks(merged));
	let result = $derived(filterLinks(views, { search, domain, sort, showHidden }));
	let sites = $derived(siteCounts(views.filter((view) => !view.link.hidden)));
	let groups = $derived($groupBySiteSetting ? groupBySite(result.main) : null);
	let selected = $derived(views.filter((view) => selectedIds.includes(view.link.id)));
	let shown = $derived([...result.main, ...(showLow ? result.low : [])]);
	let filtered = $derived(!!search.trim() || !!domain);
	let relevantCount = $derived(views.filter((view) => !view.link.low_relevance && !view.link.hidden).length);
	$effect(() => {
		const ids = new Set(links.map((link) => link.id));
		if (selectedIds.some((id) => !ids.has(id))) selectedIds = selectedIds.filter((id) => ids.has(id));
	});

	// Links that arrive while the list is open are highlighted briefly.
	let known: Set<string> | null = null;
	let fresh = $state<string[]>([]);
	$effect(() => {
		const ids = links.map((link) => link.id);
		if (!known) {
			known = new Set(ids);
			return;
		}
		const added = ids.filter((id) => !known!.has(id));
		if (!added.length) return;
		for (const id of added) known.add(id);
		fresh = [...fresh, ...added];
		setTimeout(() => (fresh = fresh.filter((id) => !added.includes(id))), 2000);
	});

	// Keep the link being read in place when new links are sorted in above it.
	let scroller = $state<HTMLDivElement>();
	let anchor: { id: string; top: number } | null = null;
	$effect.pre(() => {
		void links.length;
		const element = scroller;
		if (!element || element.scrollTop <= 0) {
			anchor = null;
			return;
		}
		const boundary = element.getBoundingClientRect().top;
		const first = [...element.querySelectorAll<HTMLElement>('[data-link-id]')].find(
			(row) => row.getBoundingClientRect().bottom > boundary
		);
		anchor = first ? { id: first.dataset.linkId!, top: first.getBoundingClientRect().top } : null;
	});
	$effect(() => {
		void links.length;
		const saved = anchor;
		const element = scroller;
		if (!saved || !element) return;
		void tick().then(() => {
			const row = element.querySelector<HTMLElement>(`[data-link-id="${CSS.escape(saved.id)}"]`);
			if (row) element.scrollTop += row.getBoundingClientRect().top - saved.top;
		});
	});

	function update(id: string, change: Partial<LinkResult>) {
		overrides = { ...overrides, [id]: { ...overrides[id], ...change } };
	}
	async function markVisited(view: LinkView, visited: boolean) {
		const previous = view.link.visited_at ?? null;
		update(view.link.id, { visited_at: visited ? (previous ?? Date.now() / 1000) : null });
		try {
			await setLinkVisited(view.link.id, visited);
		} catch (error) {
			update(view.link.id, { visited_at: previous });
			toast(t('links.visitedFailed', { error: errorText(error) }), 'error');
		}
	}
	async function setHidden(list: LinkView[], hidden: boolean) {
		for (const view of list) update(view.link.id, { hidden });
		selectedIds = selectedIds.filter((id) => !list.some((view) => view.link.id === id));
		const failed: LinkView[] = [];
		for (const view of list) {
			try {
				await setLinkHidden(view.link.id, hidden);
			} catch {
				failed.push(view);
				update(view.link.id, { hidden: !hidden });
			}
		}
		if (failed.length) {
			toast(t(hidden ? 'links.hideFailed' : 'links.showFailed', { count: failed.length }), 'error');
			return;
		}
		if (hidden)
			toast(
				t('links.hidden', { count: list.length }),
				'info',
				{ label: t('common.undo'), run: () => setHidden(list, false) }
			);
	}
	async function open(list: LinkView[]) {
		confirmOpen = null;
		for (const view of list) if (await openLink(view.link.url)) void markVisited(view, true);
	}
	function requestOpen(list: LinkView[]) {
		if (list.length > OPEN_CONFIRM_LIMIT) confirmOpen = list;
		else void open(list);
	}
	function toggle(id: string) {
		selectedIds = selectedIds.includes(id)
			? selectedIds.filter((item) => item !== id)
			: [...selectedIds, id];
	}
	function copyMenu(point: { x: number; y: number }, list: LinkView[]) {
		showContextMenu(
			point,
			t('links.copyCount', { count: list.length }),
			COPY_FORMATS.map((format: CopyFormat) => ({
				label: copyFormatLabel(format),
				icon: format === 'markdown' ? LinkIcon : CopyIcon,
				action: () =>
					copyWithToast(
						formatLinks(list, format),
						t('images.linksCopied', { count: list.length })
					),
			}))
		);
	}
	function itemsFor(view: LinkView): MenuItem[] {
		const isSelected = selectedIds.includes(view.link.id);
		return [
			{ label: t('links.open'), icon: ExternalLinkIcon, action: () => open([view]) },
			{
				label: t('links.copy'),
				icon: CopyIcon,
				separator: true,
				action: () => copyWithToast(view.link.url, t('links.copied')),
			},
			{
				label: t('links.copyMarkdown'),
				icon: LinkIcon,
				action: () => copyWithToast(markdownLink(view.link.title, view.link.url), t('links.markdownCopied')),
			},
			{
				label: view.link.visited_at ? t('links.markNotVisited') : t('links.markVisited'),
				icon: view.link.visited_at ? EyeOffIcon : EyeIcon,
				separator: true,
				action: () => markVisited(view, !view.link.visited_at),
			},
			{
				label: isSelected ? t('images.deselect') : t('images.select'),
				icon: isSelected ? SquareIcon : SquareCheckIcon,
				action: () => toggle(view.link.id),
			},
			view.link.hidden
				? { label: t('links.showAgain'), icon: EyeIcon, action: () => setHidden([view], false) }
				: { label: t('links.hide'), icon: EyeOffIcon, action: () => setHidden([view], true) },
		];
	}
</script>

{#snippet row(view: LinkView)}
	<div data-link-id={view.link.id}>
		<LinkCard
			{view}
			selected={selectedIds.includes(view.link.id)}
			selecting={selected.length > 0}
			fresh={fresh.includes(view.link.id)}
			onopen={() => markVisited(view, true)}
			ontoggle={() => toggle(view.link.id)}
			onunhide={() => setHidden([view], false)}
			onmenu={(point) => showContextMenu(point, t('common.actionsFor', { name: view.link.title || view.link.url }), itemsFor(view))}
			menuItems={() => itemsFor(view)}
		/>
	</div>
{/snippet}

<div class="link-results">
	{#if links.length}
		<div class="toolbar">
			<label class="search"
				><SearchIcon size={16} /><input
					class="input sm"
					type="search"
					placeholder={t('links.searchPlaceholder')}
					aria-label={t('links.search')}
					bind:value={search}
				/></label
			>
			<select class="input sm" aria-label={t('links.sort')} bind:value={sort}>
				<option value="relevance">{t('images.sortBest')}</option>
				<option value="site">{t('links.sortSite')}</option>
				<option value="found">{t('images.sortFound')}</option>
			</select>
			{#if sites.length > 1}<select class="input sm" aria-label={t('links.site')} bind:value={domain}>
					<option value="">{t('images.allSites')}</option>
					{#each sites as [name, count]}<option value={name}>{name} ({count})</option>{/each}
				</select>{/if}
			<label class="toggle"
				><input type="checkbox" bind:checked={$groupBySiteSetting} />{t('links.groupBySite')}</label
			>
			{#if result.hiddenCount}<label class="toggle"
					><input type="checkbox" bind:checked={showHidden} />{t('links.showHidden', { count: result.hiddenCount })}</label
				>{/if}
			<span class="count link-count" role="status"
				>{filtered
					? t('links.relevantOf', { shown: result.main.length, count: relevantCount })
					: t('links.relevant', { count: relevantCount })}</span
			>
			<button
				class="button ghost sm copy-all"
				disabled={!shown.length}
				aria-haspopup="menu"
				onclick={(event) => copyMenu(menuPointFor(event.currentTarget), shown)}
				><ClipboardCopyIcon size={15} />{t('links.copyAll')}<ChevronDownIcon size={14} /></button
			>
		</div>
		{#if selected.length}
			<div class="selection-bar" role="region" aria-label={t('links.selected')}>
				<strong>{t('common.selectedCount', { count: selected.length })}</strong>
				{#if shown.some((view) => !selectedIds.includes(view.link.id))}<button
						class="button ghost sm"
						onclick={() => (selectedIds = [...new Set([...selectedIds, ...shown.map((view) => view.link.id)])])}
						>{t('images.selectShown')}</button
					>{/if}
				<span class="spacer"></span>
				<button
					class="button sm"
					aria-haspopup="menu"
					onclick={(event) => copyMenu(menuPointFor(event.currentTarget), selected)}
					><CopyIcon size={14} />{t('links.copyShort')}<ChevronDownIcon size={14} /></button
				>
				<button class="button sm" onclick={() => requestOpen(selected)}
					><ExternalLinkIcon size={14} />{t('links.openInBrowser')}</button
				>
				<button class="button sm" onclick={() => setHidden(selected, true)}
					><EyeOffIcon size={14} />{t('links.hideShort')}</button
				>
				<button class="icon-button ghost sm" aria-label={t('common.clearSelection')} onclick={() => (selectedIds = [])}
					><XIcon size={14} /></button
				>
			</div>
		{/if}
	{/if}
	<div class="link-list" bind:this={scroller}>
		{#if !links.length}<EmptyState>{t('links.empty')}</EmptyState>
		{:else if !result.main.length && !result.low.length}<EmptyState>
				{t('links.noMatch')}
				{#snippet action()}<button
						class="button sm"
						onclick={() => {
							search = '';
							domain = '';
						}}>{t('common.resetFilters')}</button
					>{/snippet}
			</EmptyState>
		{:else}
			{#if groups}
				{#each groups as group (group.domain)}
					<section class="site-group" aria-label={group.domain}>
						<h3>
							<SiteIcon domain={group.domain} host={group.views[0].host} />
							{group.domain}<span>{group.views.length}</span>
						</h3>
						{#each group.views as view (view.link.id)}{@render row(view)}{/each}
					</section>
				{/each}
			{:else}
				{#each result.main as view (view.link.id)}{@render row(view)}{/each}
			{/if}
			{#if !result.main.length}<p class="no-relevant">
					{t('links.noneRelevant')}
				</p>{/if}
			{#if result.low.length}
				<button class="show-low" aria-expanded={showLow} onclick={() => (showLow = !showLow)}>
					<ChevronDownIcon size={15} />
					{t(showLow ? 'links.hideLow' : 'links.showLow', { count: result.low.length })}
				</button>
				{#if showLow}
					<div class="low-list" aria-label={t('links.lowRegion')} role="region">
						{#each result.low as view (view.link.id)}{@render row(view)}{/each}
					</div>
				{/if}
			{/if}
		{/if}
	</div>
</div>

{#if confirmOpen}
	{@const list = confirmOpen}
	<Dialog title={t('links.openConfirm', { count: list.length })} onclose={() => (confirmOpen = null)}>
		<p>{t('links.openConfirmText')}</p>
		{#snippet footer()}
			<button class="button" onclick={() => (confirmOpen = null)}>{t('export.cancel')}</button>
			<button class="button primary" onclick={() => open(list)}>{t('links.openCount', { count: list.length })}</button>
		{/snippet}
	</Dialog>
{/if}

<style>
	.link-results {
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
		padding-bottom: 8px;
		flex-shrink: 0;
	}
	.toolbar select {
		width: auto;
		max-width: 170px;
	}
	.search {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--app-muted);
	}
	.search input {
		width: min(240px, 26vw);
	}
	.toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--app-text-md);
		white-space: nowrap;
	}
	.count {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.copy-all {
		margin-left: auto;
	}
	.selection-bar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		margin-bottom: 8px;
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
	.link-list {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		overflow-anchor: none;
		scrollbar-gutter: stable;
		padding: 0 12px 16px 0;
		border-top: 1px solid var(--app-border);
	}
	.site-group h3 {
		position: sticky;
		top: 0;
		z-index: 1;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 6px;
		border-bottom: 1px solid var(--app-border);
		background: var(--app-bg);
		font-size: var(--app-text-md);
		font-weight: 650;
	}
	.site-group h3 span {
		color: var(--app-muted);
		font-weight: 400;
	}
	.no-relevant {
		padding: 16px 6px;
		color: var(--app-muted);
		font-size: var(--app-text-md);
	}
	.show-low {
		display: flex;
		align-items: center;
		gap: 6px;
		margin: 12px 0 4px;
		padding: 6px 8px;
		border-radius: var(--app-radius-sm);
		color: var(--app-muted);
		font-size: var(--app-text-md);
	}
	.show-low:hover {
		background: var(--app-subtle);
		color: var(--app-text);
	}
	.show-low[aria-expanded='true'] :global(svg) {
		transform: rotate(180deg);
	}
</style>
