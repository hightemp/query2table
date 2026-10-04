<script lang="ts">
	import { tick } from 'svelte';
	import type { HistoryRun } from '$lib/types';
	import {
		TableIcon,
		ImageIcon,
		LinkIcon,
		BrainIcon,
		PinIcon,
		EllipsisIcon,
	} from '@lucide/svelte';
	import Badge from '$lib/components/common/Badge.svelte';
	import { isMenuKey } from '$lib/components/common/ContextMenu.svelte';
	import { relativeTime, runSummary } from '$lib/utils/history';
	import { statusLabel, statusTone } from '$lib/utils/status';
	import { presentError } from '$lib/utils/errors';

	let {
		run,
		now,
		selected = false,
		renaming = false,
		onselect,
		onopen,
		onmenu,
		onrename,
	}: {
		run: HistoryRun;
		now: number;
		selected?: boolean;
		renaming?: boolean;
		onselect: (checked: boolean, range: boolean) => void;
		/** Called before following the link; may prevent it. */
		onopen: (event: MouseEvent) => void;
		onmenu: (point: { x: number; y: number }) => void;
		/** The new name, or null when renaming was cancelled. */
		onrename: (title: string | null) => void;
	} = $props();

	const icons = { table: TableIcon, images: ImageIcon, links: LinkIcon, research: BrainIcon } as const;
	const typeNames: Record<string, string> = { table: 'Table', images: 'Images', links: 'Links', research: 'Research' };
	let Icon = $derived(icons[run.run_type as keyof typeof icons] ?? TableIcon);
	let name = $derived(run.title || run.query);
	let summary = $derived(runSummary(run));
	let unusual = $derived(run.status !== 'completed');
	let meta = $derived(
		[summary.results, summary.duration, summary.cost, summary.model].filter(Boolean) as string[]
	);
	let input = $state<HTMLInputElement>();
	let draft = $state('');
	let done = false;
	$effect(() => {
		if (!renaming) return;
		draft = name;
		done = false;
		void tick().then(() => input?.select());
	});
	function finish(value: string | null) {
		if (done) return;
		done = true;
		onrename(value);
	}
</script>

<li
	class="history-row"
	class:selected
	oncontextmenu={(event) => {
		event.preventDefault();
		onmenu({ x: event.clientX, y: event.clientY });
	}}
>
	<input
		type="checkbox"
		class="select"
		aria-label={`Select ${name}`}
		checked={selected}
		onclick={(event) => onselect(event.currentTarget.checked, event.shiftKey)}
	/>
	<span class="type" title={typeNames[run.run_type] ?? run.run_type}><Icon size={16} /></span>
	<div class="main">
		<div class="title-line">
			{#if renaming}
				<input
					bind:this={input}
					class="input sm rename"
					aria-label="Run name"
					bind:value={draft}
					onkeydown={(event) => {
						if (event.key === 'Enter') finish(draft);
						else if (event.key === 'Escape') finish(null);
					}}
					onblur={() => finish(draft)}
				/>
			{:else}
				<a
					class="row-link"
					href={`/history/${encodeURIComponent(run.id)}`}
					onclick={onopen}
					onkeydown={(event) => {
						if (isMenuKey(event)) {
							event.preventDefault();
							const rect = event.currentTarget.getBoundingClientRect();
							onmenu({ x: rect.left + 8, y: rect.bottom });
						}
					}}>{name}</a
				>
			{/if}
			{#if run.pinned_at}<span class="pin" title="Pinned"><PinIcon size={13} /></span>{/if}
		</div>
		<p class="meta">
			{#if run.title}<span class="query">{run.query}</span>{/if}
			{#each meta as item}<span>{item}</span>{/each}
			{#if run.error && run.status === 'failed'}<span class="error">{presentError(run.error).title}</span
				>{/if}
			<time datetime={new Date(run.created_at * 1000).toISOString()} title={new Date(run.created_at * 1000).toLocaleString()}
				>{relativeTime(run.created_at, now)}</time
			>
		</p>
	</div>
	{#if unusual}<Badge tone={statusTone(run.status)}>{statusLabel(run.status)}</Badge>{/if}
	<button
		class="icon-button menu"
		aria-label={`Actions for ${name}`}
		aria-haspopup="menu"
		onclick={(event) => {
			const rect = event.currentTarget.getBoundingClientRect();
			onmenu({ x: rect.right - 200, y: rect.bottom + 4 });
		}}><EllipsisIcon size={16} /></button
	>
</li>

<style>
	.history-row {
		position: relative;
		display: grid;
		grid-template-columns: auto auto minmax(0, 1fr) auto auto;
		align-items: center;
		gap: 10px;
		padding: 8px 10px;
		border-bottom: 1px solid var(--app-border);
		border-radius: var(--app-radius-sm);
	}
	.history-row:hover,
	.history-row:focus-within {
		background: var(--app-subtle);
	}
	.history-row.selected {
		background: color-mix(in srgb, var(--app-accent) 10%, transparent);
	}
	.select {
		position: relative;
		z-index: 1;
		opacity: 0;
	}
	.history-row:hover .select,
	.history-row.selected .select,
	.select:focus-visible,
	:global(.history-list.selecting) .select {
		opacity: 1;
	}
	.type {
		display: flex;
		color: var(--app-muted);
	}
	.main {
		min-width: 0;
	}
	.title-line {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}
	.row-link {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-weight: 600;
		color: var(--app-text);
		text-decoration: none;
	}
	/* The whole row opens the run. */
	.row-link::after {
		content: '';
		position: absolute;
		inset: 0;
	}
	.row-link:focus-visible {
		outline: none;
	}
	.history-row:has(.row-link:focus-visible) {
		outline: 2px solid var(--app-accent);
		outline-offset: -2px;
	}
	.rename {
		width: 100%;
		max-width: 480px;
	}
	.pin {
		display: flex;
		flex-shrink: 0;
		color: var(--app-accent);
	}
	.meta {
		display: flex;
		flex-wrap: wrap;
		gap: 0 6px;
		margin-top: 2px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.meta > :not(:last-child)::after {
		content: '·';
		margin-left: 6px;
	}
	.query {
		max-width: 40ch;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.error {
		color: var(--app-danger);
	}
	.menu {
		position: relative;
		z-index: 1;
	}
	.history-row :global(.badge) {
		position: relative;
		z-index: 1;
	}
</style>
