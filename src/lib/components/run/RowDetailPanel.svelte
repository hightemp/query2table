<script lang="ts">
	import { tooltip } from '$lib/actions/tooltip';
	import { t } from '$lib/i18n';
	import type { RunRow } from '$lib/stores/run';
	import type { RowSource } from '$lib/types';
	import { getRowSources } from '$lib/api/tauri';
	import { formatValue, webUrl, columnLabel } from '$lib/utils/values';
	import { errorText, type ErrorContext } from '$lib/utils/errors';
	import { ChevronUpIcon, ChevronDownIcon } from '@lucide/svelte';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import CopyButton from '$lib/components/common/CopyButton.svelte';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	let {
		row,
		columns,
		onclose,
		position = null,
		onnavigate,
		context = 'run',
	}: {
		row: RunRow;
		columns: string[];
		onclose: () => void;
		/** Place of this row among the displayed rows, for previous/next navigation. */
		position?: { index: number; total: number } | null;
		onnavigate?: (delta: number) => void;
		context?: ErrorContext;
	} = $props();
	let canPrevious = $derived(!!position && position.index > 0);
	let canNext = $derived(!!position && position.index < position.total - 1);
	function navigate(delta: number) {
		if ((delta < 0 && canPrevious) || (delta > 0 && canNext)) onnavigate?.(delta);
	}
	function handleKey(event: KeyboardEvent) {
		if ((event.target as Element).closest('input, textarea, select')) return;
		const previous = (event.altKey && event.key === 'ArrowUp') || (!event.altKey && event.key === 'k');
		const next = (event.altKey && event.key === 'ArrowDown') || (!event.altKey && event.key === 'j');
		if ((!previous && !next) || event.ctrlKey || event.metaKey) return;
		event.preventDefault();
		navigate(previous ? -1 : 1);
	}
	let sources = $state<RowSource[]>([]);
	let loading = $state(true);
	let error = $state('');
	let retry = $state(0);
	$effect(() => {
		const id = row.id;
		void retry;
		let current = true;
		loading = true;
		sources = [];
		error = '';
		getRowSources(id)
			.then((result) => {
				if (current) sources = result;
			})
			.catch((reason) => {
				if (current) error = errorText(reason);
			})
			.finally(() => {
				if (current) loading = false;
			});
		return () => {
			current = false;
		};
	});
</script>

{#snippet navigation()}
	<span class="position">{t('row.position', { n: (position?.index ?? 0) + 1, total: position?.total ?? 0 })}</span>
	<button
		class="button sm"
		disabled={!canPrevious}
		onclick={() => navigate(-1)}
		use:tooltip={t('row.previousHint')}><ChevronUpIcon size={16} />{t('row.previous')}</button
	>
	<button
		class="button sm"
		disabled={!canNext}
		onclick={() => navigate(1)}
		use:tooltip={t('row.nextHint')}><ChevronDownIcon size={16} />{t('row.next')}</button
	>
{/snippet}

<svelte:window onkeydown={handleKey} />
<Dialog
	title={t('row.title')}
	variant="drawer"
	{onclose}
	footer={position && onnavigate ? navigation : undefined}
>
	<div class="confidence" class:low={row.confidence < 0.6}>
		<span>{t('row.confidence')}</span><strong>{Math.round(row.confidence * 100)}%</strong>
		{#if row.sources !== undefined}<span class="source-count"
				>{t('units.source', { count: row.sources })}</span
			>{/if}
	</div>
	<dl>
		{#each columns as column}<div class="field">
				<dt>
					<span use:tooltip={column}>{columnLabel(column)}</span><CopyButton
						text={formatValue(row.data[column], true)}
						label={t('row.copyField', { name: columnLabel(column) })}
					/>
				</dt>
				<dd>
					{#if webUrl(row.data[column])}<ExternalLink
							href={String(row.data[column])}
						/>{:else}<pre>{formatValue(row.data[column], true)}</pre>{/if}
				</dd>
			</div>{/each}
	</dl>
	<section aria-label={t('row.sources')}>
		<h3>{t('row.sourcesTitle')}</h3>
		{#if loading}<p role="status">{t('row.loadingSources')}</p>
		{:else if error}<ErrorNotice {error} {context} /><button
				class="button"
				onclick={() => {
					retry++;
				}}>{t('row.retrySources')}</button
			>
		{:else if !sources.length}<p class="muted">{t('row.noSources')}</p>
		{:else}{#each sources as source}<article>
					<div class="source-heading">
						<ExternalLink href={source.url} label={source.title || source.url} /><CopyButton
							text={source.url}
							label={t('row.copySourceUrl')}
						/>
					</div>
					{#if source.title}<p class="source-url">
							{source.url}
						</p>{/if}{#if source.snippet}<blockquote>{source.snippet}</blockquote>{/if}
				</article>{/each}{/if}
	</section>
</Dialog>

<style>
	.position {
		margin-right: auto;
		color: var(--app-muted);
		font-size: var(--app-text-md);
	}
	.source-count {
		margin-left: auto;
		color: var(--app-muted);
	}
	.confidence.low strong {
		color: var(--app-warning);
	}
	.confidence {
		display: flex;
		gap: 12px;
		padding: 12px;
		border-radius: var(--app-radius);
		background: var(--app-subtle);
		font-size: var(--app-text-md);
	}
	dl {
		margin: 16px 0 24px;
	}
	.field {
		padding: 12px 0;
		border-bottom: 1px solid var(--app-border);
	}
	dt {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		align-items: center;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		font-weight: 600;
	}
	dd {
		margin: 4px 0 0;
	}
	pre {
		font: inherit;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		margin: 0;
	}
	h3 {
		font-size: var(--app-text-lg);
		font-weight: 650;
		margin-bottom: 8px;
	}
	article {
		margin: 12px 0;
		padding: 12px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
	}
	.source-heading {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 8px;
	}
	.source-url {
		font-size: var(--app-text-sm);
		color: var(--app-muted);
		overflow-wrap: anywhere;
	}
	blockquote {
		margin: 10px 0 0;
		padding-left: 10px;
		border-left: 2px solid var(--app-border);
		font-size: var(--app-text-md);
		white-space: pre-wrap;
	}
</style>
