<script lang="ts">
	import { tooltip } from '$lib/actions/tooltip';
	import { t } from '$lib/i18n';
	import Checkbox from '$lib/components/common/Checkbox.svelte';
	import { CopyIcon, EllipsisIcon, EyeIcon } from '@lucide/svelte';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import type { MenuItem } from '$lib/components/common/ContextMenu.svelte';
	import { menuPointFor } from '$lib/components/common/ContextMenu.svelte';
	import SiteIcon from './SiteIcon.svelte';
	import { tierLabel, type LinkView } from '$lib/utils/linkResults';
	import { copyWithToast } from '$lib/utils/linkMenu';

	let {
		view,
		selected = false,
		selecting = false,
		fresh = false,
		onopen,
		ontoggle,
		onunhide,
		onmenu,
		menuItems,
	}: {
		view: LinkView;
		selected?: boolean;
		selecting?: boolean;
		/** Arrived during the live run; highlighted briefly. */
		fresh?: boolean;
		onopen: () => void;
		ontoggle: () => void;
		onunhide: () => void;
		onmenu: (point: { x: number; y: number }) => void;
		menuItems: () => MenuItem[];
	} = $props();

	let expanded = $state(false);
	let link = $derived(view.link);
	let visited = $derived(!!link.visited_at);
	let percent = $derived(
		link.relevance_score === null ? null : Math.round(link.relevance_score * 100)
	);
	let tierTitle = $derived(
		[percent === null ? null : t('links.matchPercent', { percent }), link.reason].filter(Boolean).join(' — ')
	);
	let longText = $derived(link.description.length > 160 || !!link.reason);
</script>

<article
	class="link-card"
	class:selected
	class:selecting
	class:visited
	class:hidden-link={link.hidden}
	class:low={link.low_relevance}
	class:fresh
	oncontextmenu={(event) => {
		event.preventDefault();
		onmenu({ x: event.clientX, y: event.clientY });
	}}
>
	<span class="check"
		><Checkbox checked={selected} onchange={ontoggle} label={t('links.selectOne', { title: link.title || link.url })} /></span
	>

	<div class="body">
		<div class="site-line">
			<SiteIcon domain={view.domain} host={view.host} />
			<span class="site" use:tooltip={link.url}
				>{view.domain}{#if view.path}<span class="path">{` › ${view.path}`}</span>{/if}</span
			>
			{#if visited}<span class="visited-mark" use:tooltip={t('links.openedBefore')}
					><EyeIcon size={12} />{t('links.visited')}</span
				>{/if}
			{#if view.tier}<span class="tier {view.tier}" use:tooltip={tierTitle}>{tierLabel(view.tier)}</span
				>{/if}
		</div>
		<h3>
			<ExternalLink
				href={link.url}
				label={link.title || link.url}
				class="title-link"
				{onopen}
				{menuItems}
			/>
		</h3>
		{#if link.description || link.reason}
			{#if longText}
				<button
					class="description"
					class:expanded
					aria-expanded={expanded}
					onclick={() => (expanded = !expanded)}
					use:tooltip={expanded ? t('common.showLess') : t('common.showMore')}
				>
					<span class="text">{link.description}</span>
					{#if expanded && link.reason}<span class="reason"
							><strong>{t('links.why')}</strong> {link.reason}</span
						>{/if}
				</button>
			{:else}
				<p class="description">{link.description}</p>
			{/if}
		{/if}
		{#if link.hidden}<p class="hidden-note">
				{t('links.hiddenNote')} <button class="button ghost sm" onclick={onunhide}
					>{t('links.showAgain')}</button
				>
			</p>{/if}
	</div>
	<div class="actions">
		<button
			class="icon-button ghost sm"
			aria-label={t('links.copyLinkOf', { title: link.title || link.url })}
			use:tooltip={t('links.copy')}
			onclick={() => copyWithToast(link.url, t('links.copied'))}><CopyIcon size={15} /></button
		>
		<button
			class="icon-button ghost sm"
			aria-label={t('links.moreActionsFor', { title: link.title || link.url })}
			aria-haspopup="menu"
			use:tooltip={t('links.moreActions')}
			onclick={(event) => onmenu(menuPointFor(event.currentTarget))}
			><EllipsisIcon size={15} /></button
		>
	</div>
</article>

<style>
	.link-card {
		position: relative;
		display: flex;
		gap: 10px;
		padding: 12px 8px 12px 6px;
		border-bottom: 1px solid var(--app-border);
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.link-card:hover,
	.link-card:focus-within {
		background: color-mix(in srgb, var(--app-accent) 4%, transparent);
	}
	.link-card.selected {
		background: color-mix(in srgb, var(--app-accent) 9%, transparent);
	}
	.link-card.fresh {
		animation: arrive 2s ease-out;
	}
	@keyframes arrive {
		from {
			background: color-mix(in srgb, var(--app-accent) 18%, transparent);
		}
	}
	.check {
		display: flex;
		padding-top: 2px;
		opacity: 0;
	}
	.link-card:hover .check,
	.link-card:focus-within .check,
	.selecting .check,
	.selected .check {
		opacity: 1;
	}
	.body {
		flex: 1;
		min-width: 0;
	}
	.site-line {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
		font-size: var(--app-text-sm);
		color: var(--app-muted);
	}
	.site {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--app-text);
	}
	.path {
		color: var(--app-muted);
	}
	.visited-mark {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		flex-shrink: 0;
	}
	.tier {
		flex-shrink: 0;
		margin-left: auto;
		padding: 0 8px;
		border-radius: var(--app-radius-pill);
		font-size: var(--app-text-xs);
		font-weight: 600;
		line-height: 1.7;
		cursor: help;
	}
	.tier.best {
		color: var(--app-success);
		background: color-mix(in srgb, var(--app-success) 14%, transparent);
	}
	.tier.good {
		color: var(--app-accent);
		background: color-mix(in srgb, var(--app-accent) 14%, transparent);
	}
	.tier.partial {
		color: var(--app-muted);
		background: var(--app-subtle);
	}
	h3 {
		margin: 3px 0 2px;
		font-size: var(--app-text-lg);
		font-weight: 600;
		line-height: 1.35;
	}
	h3 :global(.title-link) {
		text-decoration: none;
	}
	h3 :global(.title-link:hover) {
		text-decoration: underline;
	}
	.visited h3 :global(.title-link) {
		color: color-mix(in srgb, var(--app-accent) 55%, var(--app-muted));
	}
	.description {
		display: block;
		width: 100%;
		margin: 0;
		padding: 0;
		color: var(--app-text);
		text-align: left;
		font-size: var(--app-text-md);
		line-height: 1.55;
	}
	button.description {
		cursor: pointer;
	}
	.description .text {
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
	.description.expanded .text {
		display: block;
	}
	.reason {
		display: block;
		margin-top: 6px;
		color: var(--app-muted);
	}
	.hidden-link .body,
	.low .body {
		opacity: 0.7;
	}
	.hidden-note {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 6px;
		color: var(--app-warning);
		font-size: var(--app-text-sm);
	}
	.actions {
		display: flex;
		align-items: flex-start;
		gap: 2px;
		opacity: 0;
	}
	.link-card:hover .actions,
	.link-card:focus-within .actions {
		opacity: 1;
	}
	@media (prefers-reduced-motion: reduce) {
		.link-card.fresh {
			animation: none;
		}
	}
</style>
