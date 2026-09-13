<script lang="ts">
	import type { RunRow } from '$lib/stores/run';
	import type { RowSource } from '$lib/types';
	import { getRowSources } from '$lib/api/tauri';
	import { formatValue, webUrl } from '$lib/utils/values';
	import { errorText } from '$lib/utils/errors';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import CopyButton from '$lib/components/common/CopyButton.svelte';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	let { row, columns, onclose }: { row: RunRow; columns: string[]; onclose: () => void } = $props();
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

<Dialog title="Row Details" variant="drawer" {onclose}>
	<div class="confidence">
		<span>Extraction confidence</span><strong>{Math.round(row.confidence * 100)}%</strong>
	</div>
	<dl>
		{#each columns as column}<div class="field">
				<dt>
					<span>{column}</span><CopyButton
						text={formatValue(row.data[column], true)}
						label={`Copy ${column}`}
					/>
				</dt>
				<dd>
					{#if webUrl(row.data[column])}<ExternalLink
							href={String(row.data[column])}
						/>{:else}<pre>{formatValue(row.data[column], true)}</pre>{/if}
				</dd>
			</div>{/each}
	</dl>
	<section aria-label="Row sources">
		<h3>Sources</h3>
		{#if loading}<p role="status">Loading sources…</p>
		{:else if error}<ErrorNotice {error} context="history" /><button
				class="button"
				onclick={() => {
					retry++;
				}}>Retry loading sources</button
			>
		{:else if !sources.length}<p class="muted">No sources were saved for this row.</p>
		{:else}{#each sources as source}<article>
					<div class="source-heading">
						<ExternalLink href={source.url} label={source.title || source.url} /><CopyButton
							text={source.url}
							label="Copy source URL"
						/>
					</div>
					{#if source.title}<p class="source-url">
							{source.url}
						</p>{/if}{#if source.snippet}<blockquote>{source.snippet}</blockquote>{/if}
				</article>{/each}{/if}
	</section>
</Dialog>

<style>
	.confidence {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		padding: 12px;
		border-radius: 8px;
		background: var(--app-subtle);
		font-size: 13px;
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
		font-size: 12px;
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
		font-size: 16px;
		font-weight: 650;
		margin-bottom: 8px;
	}
	article {
		margin: 12px 0;
		padding: 12px;
		border: 1px solid var(--app-border);
		border-radius: 8px;
	}
	.source-heading {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 8px;
	}
	.source-url {
		font-size: 12px;
		color: var(--app-muted);
		overflow-wrap: anywhere;
	}
	blockquote {
		margin: 10px 0 0;
		padding-left: 10px;
		border-left: 2px solid var(--app-border);
		font-size: 13px;
		white-space: pre-wrap;
	}
</style>
