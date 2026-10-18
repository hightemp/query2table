<script lang="ts">
	import FileChips from '$lib/components/common/FileChips.svelte';
	import type { AttachmentInfo, ResearchImage } from '$lib/types';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import { displayHost } from '$lib/utils/hosts';
	import { t, type MessageKey } from '$lib/i18n';
	import { tick } from 'svelte';
	import type { ResearchStep } from '$lib/types';
	import { CopyIcon, FileTextIcon, ListIcon, ChevronDownIcon, LinkIcon } from '@lucide/svelte';
	import Markdown from '$lib/components/common/Markdown.svelte';
	import { menuPointFor } from '$lib/components/common/ContextMenu.svelte';
	import ResearchSteps from './ResearchSteps.svelte';
	import ResearchSources from './ResearchSources.svelte';
	import { showContextMenu } from '$lib/stores/contextMenu';
	import { copyWithToast } from '$lib/utils/linkMenu';
	import { formatUsd } from '$lib/utils/stopConditions';
	import {
		answerHeadings,
		answerWithSources,
		readableAnswer,
		collectSources,
		markdownToText,
	} from '$lib/utils/research';

	let {
		answer,
		steps,
		running = false,
		status = 'completed',
		question = null,
		showQuestion = false,
		cost = null,
		files = [],
		images = null,
		loadImage,
	}: {
		answer: string | null;
		steps: ResearchStep[];
		running?: boolean;
		/** running, completed, cancelled or failed */
		status?: string;
		/** Names the turn; shown as its title when `showQuestion` is set. */
		question?: string | null;
		showQuestion?: boolean;
		/** Cost of this turn, shown in conversations. */
		cost?: number | null;
		/** Files attached with this question. */
		files?: AttachmentInfo[];
		/** Pictures kept with the run's answers; null while not known yet. */
		images?: ResearchImage[] | null;
		loadImage?: (url: string) => Promise<string | null>;
	} = $props();

	/** The picture shown full size. */
	let viewing = $state<{ url: string; src: string; image: ResearchImage; caption: string } | null>(null);
	async function showImage(url: string) {
		const image = images?.find((item) => item.url === url);
		const src = image && (await loadImage?.(url));
		if (!image || !src) return;
		const caption = answerImageCaption(url) || image.title;
		viewing = { url, src, image, caption };
	}
	function answerImageCaption(url: string): string {
		for (const match of (answer ?? '').matchAll(/!\[([^\]]*)\]\(\s*<?([^\s)>]+)/g)) if (match[2] === url) return match[1].trim();
		return '';
	}
	function pageHost(url: string | null): string {
		try {
			return url ? displayHost(new URL(url).hostname) : '';
		} catch {
			return '';
		}
	}

	type Tab = 'answer' | 'activity' | 'sources';
	const id = $props.id();
	let chosen = $state<Tab | null>(null);
	// Until the reader picks a tab, follow the run: activity while working, then the answer.
	let tab = $derived<Tab>(chosen ?? (answer ? 'answer' : 'activity'));
	$effect(() => {
		if (!answer && chosen === 'answer') chosen = null;
	});
	let sources = $derived(collectSources(steps, answer, images ?? []));
	let headings = $derived(answer ? answerHeadings(answer) : []);
	let showContents = $derived(headings.length >= 3);
	let activeHeading = $state<string | null>(null);
	let layout = $state<HTMLDivElement>();

	const tabs: { value: Tab; label: MessageKey }[] = [
		{ value: 'answer', label: 'research.tab.answer' },
		{ value: 'activity', label: 'research.tab.activity' },
		{ value: 'sources', label: 'research.tab.sources' },
	];
	function count(value: Tab) {
		return value === 'activity' ? steps.length : value === 'sources' ? sources.length : null;
	}
	let tabBar = $state<HTMLDivElement>();
	let panel = $state<HTMLDivElement>();
	function select(value: Tab) {
		if (value === 'answer' && !answer) return;
		chosen = value;
		void showFromStart();
	}
	/** Keeps a short tab tall enough for its tabs to stay at the top of the view. */
	let panelMinHeight = $state<number | null>(null);
	// A reader already past the tabs sees the new tab from its start, right under the tabs.
	async function showFromStart() {
		const element = tabBar?.closest<HTMLElement>('.research-view');
		if (!element || !tabBar) return;
		const viewTop = element.getBoundingClientRect().top;
		if (tabBar.getBoundingClientRect().top > viewTop + 1) {
			panelMinHeight = null;
			return;
		}
		panelMinHeight = element.clientHeight - tabBar.offsetHeight;
		await tick();
		if (!panel) return;
		element.scrollTop += panel.getBoundingClientRect().top - viewTop - tabBar.offsetHeight;
	}
	function handleTabKey(event: KeyboardEvent) {
		if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
		event.preventDefault();
		const enabled = tabs.filter((item) => item.value !== 'answer' || answer);
		const index = enabled.findIndex((item) => item.value === tab);
		const next =
			event.key === 'Home'
				? enabled[0]
				: event.key === 'End'
					? enabled[enabled.length - 1]
					: enabled[(index + (event.key === 'ArrowRight' ? 1 : -1) + enabled.length) % enabled.length];
		select(next.value);
		document.getElementById(`${id}-tab-${next.value}`)?.focus();
	}

	function scroller(): HTMLElement | null {
		return layout?.closest('.research-view') ?? null;
	}
	/** A heading chosen in the contents stays current until the reader scrolls. */
	let chosenHeading: string | null = null;
	function goTo(slug: string) {
		activeHeading = slug;
		chosenHeading = slug;
		document.getElementById(`${id}-h-${slug}`)?.scrollIntoView({ block: 'start' });
	}
	// Highlights the section being read.
	$effect(() => {
		const element = scroller();
		if (!element || !showContents || tab !== 'answer') return;
		const list = headings;
		const update = () => {
			if (chosenHeading) {
				activeHeading = chosenHeading;
				return;
			}
			const top = element.getBoundingClientRect().top + 80;
			let current = list[0]?.slug ?? null;
			for (const heading of list) {
				const target = document.getElementById(`${id}-h-${heading.slug}`);
				if (target && target.getBoundingClientRect().top <= top) current = heading.slug;
			}
			// Short last sections never reach the top; at the bottom the last heading is current.
			if (element.scrollTop > 0 && element.scrollTop + element.clientHeight >= element.scrollHeight - 2)
				current = list.at(-1)?.slug ?? current;
			activeHeading = current;
		};
		const release = () => (chosenHeading = null);
		update();
		element.addEventListener('scroll', update, { passive: true });
		for (const event of ['wheel', 'touchmove', 'keydown', 'pointerdown'] as const)
			element.addEventListener(event, release, { passive: true });
		return () => {
			element.removeEventListener('scroll', update);
			for (const event of ['wheel', 'touchmove', 'keydown', 'pointerdown'] as const)
				element.removeEventListener(event, release);
		};
	});

	function copyMenu(event: MouseEvent) {
		if (!answer) return;
		const raw = answer;
		// File citations become readable places in everything that leaves the app.
		const text = readableAnswer(raw, sources);
		showContextMenu(menuPointFor(event.currentTarget as Element), t('research.copyAnswer'), [
			{ label: t('research.copyMarkdown'), icon: CopyIcon, action: () => copyWithToast(text, t('research.copiedMarkdown')) },
			{ label: t('research.copyText'), icon: FileTextIcon, action: () => copyWithToast(markdownToText(text), t('research.copiedText')) },
			{
				label: t('research.copyWithSources'),
				icon: LinkIcon,
				action: () => copyWithToast(answerWithSources(raw, sources), t('research.copiedWithSources')),
			},
		]);
	}
	function contentsMenu(event: MouseEvent) {
		showContextMenu(
			menuPointFor(event.currentTarget as Element),
			t('research.contents'),
			headings.map((heading) => ({
				label: `${'  '.repeat(heading.depth - 1)}${heading.text}`,
				action: () => goTo(heading.slug),
			}))
		);
	}
</script>

<section class="turn" aria-label={question ?? t('research.turn')}>
	{#if showQuestion && question}<h2 class="question">{question}</h2>{/if}
	<FileChips {files} />
	{#if !answer && !running && (status === 'cancelled' || status === 'failed')}
		<p class="turn-note" class:failed={status === 'failed'}>
			{status === 'cancelled'
				? t('research.cancelledNote')
				: t('research.failedNote')}
		</p>
	{/if}
	<div class="tab-bar" bind:this={tabBar}>
		<div class="tabs" role="tablist" aria-label={t('research.results')} tabindex="-1" onkeydown={handleTabKey}>
			{#each tabs as item (item.value)}
				{@const n = count(item.value)}
				<button
					role="tab"
					id={`${id}-tab-${item.value}`}
					aria-selected={tab === item.value}
					aria-controls={`${id}-panel-${item.value}`}
					tabindex={tab === item.value ? 0 : -1}
					disabled={item.value === 'answer' && !answer}
					onclick={() => select(item.value)}
					>{t(item.label)}{#if n !== null}<span class="count">{n}</span>{/if}</button
				>
			{/each}
		</div>
		{#if tab === 'answer' && answer}
			<div class="tab-actions">
				{#if showContents}<button
						class="button ghost sm contents-button"
						aria-haspopup="menu"
						onclick={contentsMenu}><ListIcon size={15} />{t('research.contents')}<ChevronDownIcon size={14} /></button
					>{/if}
				<button class="button ghost sm" aria-haspopup="menu" onclick={copyMenu}
					><CopyIcon size={15} />{t('research.copyAnswer')}<ChevronDownIcon size={14} /></button
				>
			</div>
		{/if}
	</div>

	<div
		class="panel"
		bind:this={panel}
		style:min-height={panelMinHeight ? `${panelMinHeight}px` : null}
		role="tabpanel"
		id={`${id}-panel-${tab}`}
		aria-labelledby={`${id}-tab-${tab}`}
		tabindex="0"
	>
		{#if tab === 'answer' && answer}
			<div class="answer-layout" class:with-contents={showContents} bind:this={layout}>
				<div class="answer" aria-label={t('research.answer')} role="region">
					<Markdown content={answer} idPrefix={`${id}-h`} {images} {loadImage} onimage={showImage} />
				</div>
				{#if showContents}
					<nav class="contents" aria-label={t('research.contents')}>
						<p>{t('research.contents')}</p>
						<ul>
							{#each headings as heading (heading.slug)}
								<li class={`depth-${heading.depth}`}>
									<a
										href={`#${heading.slug}`}
										aria-current={activeHeading === heading.slug ? 'true' : undefined}
										onclick={(event) => {
											event.preventDefault();
											goTo(heading.slug);
										}}>{heading.text}</a
									>
								</li>
							{/each}
						</ul>
					</nav>
				{/if}
			</div>
		{:else if tab === 'sources'}
			<ResearchSources {sources} />
		{:else}
			<ResearchSteps {steps} {running} />
		{/if}
	</div>
	{#if viewing}
		<Dialog title={viewing.caption || t('research.image')} variant="image" onclose={() => (viewing = null)}>
			<div class="image-view"><img src={viewing.src} alt={viewing.caption} /></div>
			<p class="image-meta">
				{[pageHost(viewing.image.source_url), `${viewing.image.width} × ${viewing.image.height}`].filter(Boolean).join(' · ')}
			</p>
			{#snippet footer()}
				<div class="image-links">
					{#if viewing?.image.source_url}<ExternalLink href={viewing.image.source_url} label={t('images.sourcePage')} />{/if}
					{#if viewing}<ExternalLink href={viewing.url} label={t('images.openOriginal')} />{/if}
				</div>
			{/snippet}
		</Dialog>
	{/if}
	{#if cost !== null}<p class="turn-cost">{t('research.turnCost', { amount: formatUsd(cost) })}</p>{/if}
</section>

<style>
	.turn {
		margin-bottom: 24px;
	}
	.turn-note {
		margin-bottom: 12px;
		padding: 10px 12px;
		border-radius: var(--app-radius);
		background: var(--app-subtle);
		color: var(--app-muted);
		font-size: var(--app-text-md);
	}
	.turn-note.failed {
		color: var(--app-danger);
	}
	.image-view {
		display: flex;
		justify-content: center;
	}
	.image-view img {
		max-width: 100%;
		max-height: min(70vh, 900px);
		object-fit: contain;
		border-radius: var(--app-radius);
	}
	.image-meta {
		margin-top: 8px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.image-links {
		display: flex;
		flex-wrap: wrap;
		gap: 12px;
		margin-right: auto;
	}
	.turn-cost {
		margin-top: 8px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.question {
		margin: 0 0 12px;
		font-size: var(--app-text-2xl);
		font-weight: 650;
		line-height: 1.3;
	}
	.tab-bar {
		position: sticky;
		top: 0;
		z-index: 2;
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		justify-content: space-between;
		gap: 8px;
		border-bottom: 1px solid var(--app-border);
		background: var(--app-bg);
	}
	.tabs {
		display: flex;
		gap: 4px;
	}
	[role='tab'] {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 12px;
		border-bottom: 2px solid transparent;
		margin-bottom: -1px;
		color: var(--app-muted);
		font-weight: 600;
	}
	[role='tab']:hover:not(:disabled) {
		color: var(--app-text);
	}
	[role='tab'][aria-selected='true'] {
		color: var(--app-accent);
		border-bottom-color: var(--app-accent);
	}
	.count {
		padding: 0 7px;
		border-radius: var(--app-radius-pill);
		background: var(--app-subtle);
		font-size: var(--app-text-xs);
		line-height: 1.7;
	}
	.tab-actions {
		display: flex;
		gap: 4px;
		padding-bottom: 4px;
	}
	.panel {
		padding-top: 16px;
	}
	.panel:focus-visible {
		outline-offset: -2px;
	}
	.answer-layout {
		display: block;
	}
	.answer {
		min-width: 0;
	}
	.contents {
		display: none;
	}
	/* Wide windows: contents beside the answer; narrow ones use the Contents menu. */
	@container research (min-width: 980px) {
		.answer-layout.with-contents {
			display: grid;
			grid-template-columns: minmax(0, 1fr) 220px;
			gap: 32px;
		}
		.contents {
			display: block;
			position: sticky;
			top: 56px;
			align-self: start;
			max-height: calc(100vh - 160px);
			overflow: auto;
			font-size: var(--app-text-md);
		}
		.contents-button {
			display: none;
		}
	}
	.contents p {
		margin-bottom: 6px;
		color: var(--app-muted);
		font-size: var(--app-text-xs);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
	.contents ul {
		list-style: none;
		margin: 0;
		padding: 0;
		border-left: 1px solid var(--app-border);
	}
	.contents a {
		display: block;
		padding: 4px 10px;
		margin-left: -1px;
		border-left: 2px solid transparent;
		color: var(--app-muted);
		text-decoration: none;
		line-height: 1.4;
	}
	.contents .depth-2 a {
		padding-left: 18px;
	}
	.contents .depth-3 a {
		padding-left: 28px;
	}
	.contents a:hover {
		color: var(--app-text);
	}
	.contents a[aria-current='true'] {
		color: var(--app-accent);
		border-left-color: var(--app-accent);
	}
</style>
