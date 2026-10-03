<script lang="ts">
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
	} = $props();

	type Tab = 'answer' | 'activity' | 'sources';
	const id = $props.id();
	let chosen = $state<Tab | null>(null);
	// Until the reader picks a tab, follow the run: activity while working, then the answer.
	let tab = $derived<Tab>(chosen ?? (answer ? 'answer' : 'activity'));
	$effect(() => {
		if (!answer && chosen === 'answer') chosen = null;
	});
	let sources = $derived(collectSources(steps, answer));
	let headings = $derived(answer ? answerHeadings(answer) : []);
	let showContents = $derived(headings.length >= 3);
	let activeHeading = $state<string | null>(null);
	let layout = $state<HTMLDivElement>();

	const tabs: { value: Tab; label: string }[] = [
		{ value: 'answer', label: 'Answer' },
		{ value: 'activity', label: 'Activity' },
		{ value: 'sources', label: 'Sources' },
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
	function goTo(slug: string) {
		activeHeading = slug;
		document.getElementById(`${id}-h-${slug}`)?.scrollIntoView({ block: 'start' });
	}
	// Highlights the section being read.
	$effect(() => {
		const element = scroller();
		if (!element || !showContents || tab !== 'answer') return;
		const list = headings;
		const update = () => {
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
		update();
		element.addEventListener('scroll', update, { passive: true });
		return () => element.removeEventListener('scroll', update);
	});

	function copyMenu(event: MouseEvent) {
		if (!answer) return;
		const text = answer;
		showContextMenu(menuPointFor(event.currentTarget as Element), 'Copy answer', [
			{ label: 'Copy Markdown', icon: CopyIcon, action: () => copyWithToast(text, 'Answer copied as Markdown.') },
			{ label: 'Copy as text', icon: FileTextIcon, action: () => copyWithToast(markdownToText(text), 'Answer copied as text.') },
			{
				label: 'Copy with sources',
				icon: LinkIcon,
				action: () => copyWithToast(answerWithSources(text, sources), 'Answer copied with sources.'),
			},
		]);
	}
	function contentsMenu(event: MouseEvent) {
		showContextMenu(
			menuPointFor(event.currentTarget as Element),
			'Contents',
			headings.map((heading) => ({
				label: `${'  '.repeat(heading.depth - 1)}${heading.text}`,
				action: () => goTo(heading.slug),
			}))
		);
	}
</script>

<section class="turn" aria-label={question ?? 'Research'}>
	{#if showQuestion && question}<h2 class="question">{question}</h2>{/if}
	{#if !answer && !running && (status === 'cancelled' || status === 'failed')}
		<p class="turn-note" class:failed={status === 'failed'}>
			{status === 'cancelled'
				? 'This question was cancelled before an answer. The steps collected so far are kept.'
				: 'This question could not be answered. See Activity for what went wrong.'}
		</p>
	{/if}
	<div class="tab-bar" bind:this={tabBar}>
		<div class="tabs" role="tablist" aria-label="Research results" tabindex="-1" onkeydown={handleTabKey}>
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
					>{item.label}{#if n !== null}<span class="count">{n}</span>{/if}</button
				>
			{/each}
		</div>
		{#if tab === 'answer' && answer}
			<div class="tab-actions">
				{#if showContents}<button
						class="button ghost sm contents-button"
						aria-haspopup="menu"
						onclick={contentsMenu}><ListIcon size={15} />Contents<ChevronDownIcon size={14} /></button
					>{/if}
				<button class="button ghost sm" aria-haspopup="menu" onclick={copyMenu}
					><CopyIcon size={15} />Copy answer<ChevronDownIcon size={14} /></button
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
				<div class="answer" aria-label="Research answer" role="region">
					<Markdown content={answer} idPrefix={`${id}-h`} />
				</div>
				{#if showContents}
					<nav class="contents" aria-label="Contents">
						<p>Contents</p>
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
	{#if cost !== null}<p class="turn-cost">Cost of this answer: {formatUsd(cost)}</p>{/if}
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
