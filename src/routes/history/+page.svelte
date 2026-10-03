<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import CostSummary from '$lib/components/run/CostSummary.svelte';
	import CostWarnings from '$lib/components/run/CostWarnings.svelte';
	import { storedCosts } from '$lib/utils/costs';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import { debugUi } from '$lib/utils/diagnostics';
	import {
		listRuns,
		deleteRun,
		getRunSchema,
		getRunRows,
		getImageResults,
		getLinkResults,
		getResearchResult,
		getRunIssues,
	} from '$lib/api/tauri';
	import type {
		RunInfo,
		SchemaColumn,
		ImageResult,
		LinkResult,
		ResearchStep,
		LlmIssueEvent,
	} from '$lib/types';
	import type { RunRow } from '$lib/stores/run';
	import ResultsTable from '$lib/components/run/ResultsTable.svelte';
	import RowDetailPanel from '$lib/components/run/RowDetailPanel.svelte';
	import ExportDialog from '$lib/components/run/ExportDialog.svelte';
	import ImageGallery from '$lib/components/run/ImageGallery.svelte';
	import LinkList from '$lib/components/run/LinkList.svelte';
	import ResearchView from '$lib/components/run/ResearchView.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import LlmIssues from '$lib/components/run/LlmIssues.svelte';
	import { errorText, presentError } from '$lib/utils/errors';
	import { statusLabel, statusTone } from '$lib/utils/status';
	import { RowSelection } from '$lib/utils/rowSelection.svelte';
	import Badge from '$lib/components/common/Badge.svelte';
	import EmptyState from '$lib/components/common/EmptyState.svelte';
	import {
		TrashIcon,
		ExternalLinkIcon,
		ArrowLeftIcon,
		DownloadIcon,
		TableIcon,
		ImageIcon,
		LinkIcon,
		BrainIcon,
	} from '@lucide/svelte';

	let runs = $state<RunInfo[]>([]);
	let loading = $state(true);
	let error = $state('');

	// View state
	let viewingRun = $state<RunInfo | null>(null);
	let costs = $derived(storedCosts(viewingRun?.stats));
	let viewSchema = $state<SchemaColumn[]>([]);
	let viewRows = $state<RunRow[]>([]);
	let viewImages = $state<ImageResult[]>([]);
	let viewLinks = $state<LinkResult[]>([]);
	let viewResearchSteps = $state<ResearchStep[]>([]);
	let viewResearchAnswer = $state<string | null>(null);
	let viewLoading = $state(false);
	let viewIssues = $state<LlmIssueEvent[]>([]);
	let issueError = $state('');
	let issuesLoading = $state(false);
	let viewRequest = 0;
	const selection = new RowSelection();
	let selectedRow = $derived(viewRows.find((row) => row.id === selection.id) ?? null);
	let showExport = $state(false);
	let deleteTarget = $state<RunInfo | null>(null);
	let deleting = $state(false);
	let deleteError = $state('');
	onDestroy(() => {
		viewRequest++;
	});

	onMount(async () => {
		await loadRuns();
	});

	async function loadRuns() {
		loading = true;
		error = '';
		try {
			runs = await listRuns(100);
		} catch (e) {
			error = errorText(e);
		} finally {
			loading = false;
		}
	}

	async function handleView(run: RunInfo) {
		const request = ++viewRequest;
		viewLoading = true;
		viewingRun = run;
		viewSchema = [];
		viewRows = [];
		viewImages = [];
		viewLinks = [];
		viewResearchSteps = [];
		viewResearchAnswer = null;
		error = '';
		selection.clear();
		viewIssues = [];
		issueError = '';
		issuesLoading = true;
		void getRunIssues(run.id)
			.then((issues) => {
				if (request === viewRequest)
					viewIssues = issues.filter((issue) => issue.run_id === run.id).slice(-100);
			})
			.catch((e) => {
				if (request === viewRequest) issueError = errorText(e);
			})
			.finally(() => {
				if (request === viewRequest) issuesLoading = false;
			});
		try {
			if (run.run_type === 'images') {
				const images = await getImageResults(run.id);
				if (request !== viewRequest) return;
				viewImages = images;
				viewSchema = [];
				viewRows = [];
				viewLinks = [];
				viewResearchSteps = [];
				viewResearchAnswer = null;
			} else if (run.run_type === 'links') {
				const links = await getLinkResults(run.id);
				if (request !== viewRequest) return;
				viewLinks = links;
				viewSchema = [];
				viewRows = [];
				viewImages = [];
				viewResearchSteps = [];
				viewResearchAnswer = null;
			} else if (run.run_type === 'research') {
				const res = await getResearchResult(run.id);
				if (request !== viewRequest) return;
				viewResearchSteps = res.steps;
				viewResearchAnswer = res.answer_markdown;
				viewSchema = [];
				viewRows = [];
				viewImages = [];
				viewLinks = [];
			} else {
				const [schemaInfo, rows] = await Promise.all([getRunSchema(run.id), getRunRows(run.id)]);
				if (request !== viewRequest) return;
				viewSchema = schemaInfo?.columns ?? [];
				viewRows = rows.map((r) => ({
					id: r.id,
					data: r.data as Record<string, unknown>,
					confidence: r.confidence,
					sources: r.source_count,
				}));
				viewImages = [];
				viewLinks = [];
				viewResearchSteps = [];
				viewResearchAnswer = null;
			}
		} catch (e) {
			if (request === viewRequest) error = errorText(e);
		} finally {
			if (request === viewRequest) viewLoading = false;
		}
	}

	function handleBack() {
		viewRequest++;
		viewingRun = null;
		viewLoading = false;
		viewIssues = [];
		issueError = '';
		issuesLoading = false;
		error = '';
		viewSchema = [];
		viewRows = [];
		viewImages = [];
		viewLinks = [];
		viewResearchSteps = [];
		viewResearchAnswer = null;
		selection.clear();
		showExport = false;
	}

	async function handleDelete() {
		if (!deleteTarget || deleting) return;
		deleting = true;
		deleteError = '';
		debugUi('history_delete_started');
		try {
			await deleteRun(deleteTarget.id);
			runs = runs.filter((run) => run.id !== deleteTarget!.id);
			deleteTarget = null;
			debugUi('history_delete_finished');
		} catch (reason) {
			deleteError = errorText(reason);
			debugUi('history_delete_failed');
		} finally {
			deleting = false;
		}
	}

	function resultCount(count: number, unit: string): string {
		return `${count} ${unit}${count === 1 ? '' : 's'}`;
	}

	function formatDate(ts: number): string {
		return new Date(ts * 1000).toLocaleString();
	}

</script>

<div class="history-page">
	{#if viewingRun}
		<div class="view-header">
			<button class="button sm" onclick={handleBack}>
				<ArrowLeftIcon size={16} />
				Back to History
			</button>
			<div class="view-title">
				<h1>{viewingRun.query}</h1>
				<Badge tone={statusTone(viewingRun.status)}>{statusLabel(viewingRun.status)}</Badge>
			</div>
			<div class="view-meta">
				<span>{formatDate(viewingRun.created_at)}</span>
				<span
					>{viewingRun.run_type === 'images'
						? resultCount(viewImages.length, 'image')
						: viewingRun.run_type === 'links'
							? resultCount(viewLinks.length, 'link')
							: viewingRun.run_type === 'research'
								? resultCount(viewResearchSteps.length, 'step')
								: resultCount(viewRows.length, 'row')}</span
				>
				{#if viewRows.length > 0 || viewImages.length > 0 || viewLinks.length > 0 || viewResearchAnswer}
					<button
						class="button sm accent btn-export"
						onclick={() => {
							showExport = true;
						}}
					>
						<DownloadIcon size={14} />
						Export
					</button>
				{/if}
			</div>
		</div>

		<CostSummary accounting={costs.accounting} legacy={costs.legacy} inline />
		<CostWarnings accounting={costs.accounting} />
		<div class="history-notices">
			{#if error}<ErrorNotice {error} context="history" />{/if}
			{#if viewingRun.error}<ErrorNotice error={viewingRun.error} />{/if}
			{#if issuesLoading}<p>Loading model request issues…</p>{/if}
			{#if issueError}<ErrorNotice error={issueError} context="history" />{/if}
			<LlmIssues issues={viewIssues} runStatus={viewingRun.status} />
		</div>

		{#if showExport}
			<ExportDialog
				runId={viewingRun.id}
				runType={viewingRun.run_type}
				onclose={() => {
					showExport = false;
				}}
			/>
		{/if}

		<div class="history-result">
			{#if viewLoading}
				<EmptyState role="status">Loading run results…</EmptyState>
			{:else if viewingRun.run_type === 'images'}
				{#if viewImages.length > 0}
					<ImageGallery images={viewImages} />
				{:else}
					<EmptyState>No images found for this run.</EmptyState>
				{/if}
			{:else if viewingRun.run_type === 'links'}
				{#if viewLinks.length > 0}
					<LinkList links={viewLinks} />
				{:else}
					<EmptyState>No links found for this run.</EmptyState>
				{/if}
			{:else if viewingRun.run_type === 'research'}
				{#if viewResearchAnswer || viewResearchSteps.length > 0}
					<ResearchView steps={viewResearchSteps} answer={viewResearchAnswer} />
				{:else}
					<EmptyState>No research output for this run.</EmptyState>
				{/if}
			{:else if viewSchema.length > 0 && viewRows.length > 0}
				<ResultsTable
					schema={viewSchema}
					rows={viewRows}
					onrowclick={(row, order) => selection.select(row.id, order)}
				/>
			{:else}
				<EmptyState>No results found for this run.</EmptyState>
			{/if}
		</div>
		{#if selectedRow}
			<RowDetailPanel
				row={selectedRow}
				columns={viewSchema.map((c) => c.name)}
				position={selection.position}
				onnavigate={selection.move}
				context="history"
				onclose={selection.clear}
			/>
		{/if}
	{:else}
		<header class="page-header">
			<div>
				<h1>Run History</h1>
				<p>Revisit your research and its sources.</p>
			</div>
			<button class="button" disabled={loading} onclick={loadRuns}>Refresh</button>
		</header>

		{#if error}
			<ErrorNotice {error} context="history" />
		{/if}

		{#if loading}
			<EmptyState role="status">Loading…</EmptyState>
		{:else if runs.length === 0}
			<EmptyState>No runs yet. Start a new query to see results here.</EmptyState>
		{:else}
			<div class="runs-list">
				{#each runs as run}
					<div class="run-card">
						<div class="run-card-header">
							<span class="run-query">{run.query}</span>
							<div class="header-badges">
								{#if run.run_type === 'images'}
									<Badge><ImageIcon size={12} /> Images</Badge>
								{:else if run.run_type === 'links'}
									<Badge><LinkIcon size={12} /> Links</Badge>
								{:else if run.run_type === 'research'}
									<Badge><BrainIcon size={12} /> Research</Badge>
								{:else}
									<Badge><TableIcon size={12} /> Table</Badge>
								{/if}
								<Badge tone={statusTone(run.status)}>{statusLabel(run.status)}</Badge>
							</div>
						</div>
						<div class="run-card-meta">
							<span class="run-date">{formatDate(run.created_at)}</span>
							{#if run.error}
								<span class="run-error">{presentError(run.error).title}</span>
							{/if}
						</div>
						<div class="run-card-actions">
							<button class="button sm accent" onclick={() => handleView(run)} disabled={viewLoading}>
								<ExternalLinkIcon size={14} />
								View
							</button>
							<button
								class="button sm danger"
								onclick={() => {
									deleteTarget = run;
									deleteError = '';
								}}
							>
								<TrashIcon size={14} />
								Delete
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	{/if}
</div>

{#if deleteTarget}
	<Dialog
		title="Delete saved run?"
		busy={deleting}
		onclose={() => {
			deleteTarget = null;
		}}
	>
		<p>“{deleteTarget.query}” and its saved results will be removed.</p>
		{#if deleteError}<ErrorNotice error={deleteError} context="history" />{/if}
		{#snippet footer()}<button
				class="button"
				disabled={deleting}
				onclick={() => {
					deleteTarget = null;
				}}>Keep run</button
			><button class="button danger" disabled={deleting} onclick={handleDelete}
				>{deleting ? 'Deleting…' : 'Delete run'}</button
			>{/snippet}
	</Dialog>
{/if}

<style>
	.history-page {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
	}
	.history-result {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
	}
	.history-notices {
		flex-shrink: 0;
		max-height: 28vh;
		overflow: auto;
		scrollbar-gutter: stable;
		padding-right: 12px;
	}
	.runs-list {
		flex: 1;
		min-height: 0;
		overflow: auto;
		scrollbar-gutter: stable;
		padding-right: 12px;
	}
	.run-card {
		padding: 16px 20px;
		margin-bottom: 12px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius-lg);
		background: var(--app-panel);
	}
	.run-card-header {
		display: flex;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
	}
	.run-query {
		font-weight: 600;
		font-size: var(--app-text-lg);
		overflow-wrap: anywhere;
		flex: 1 1 260px;
	}
	.header-badges {
		display: flex;
		align-items: flex-start;
		gap: 6px;
		flex-wrap: wrap;
	}
	.run-card-meta {
		display: flex;
		flex-wrap: wrap;
		gap: 8px 16px;
		font-size: var(--app-text-sm);
		color: var(--app-muted);
		margin: 8px 0 12px;
	}
	.run-error {
		color: var(--app-danger);
	}
	.run-card-actions {
		display: flex;
		gap: 8px;
	}
	.view-header {
		flex-shrink: 0;
		margin-bottom: 16px;
	}
	.view-title {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		margin: 12px 0 8px;
	}
	.view-title h1 {
		font-size: var(--app-text-2xl);
		line-height: 1.35;
		font-weight: 650;
		overflow-wrap: anywhere;
		max-height: 90px;
		overflow: auto;
		min-width: 0;
	}
	.view-meta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 16px;
		font-size: var(--app-text-sm);
		color: var(--app-muted);
	}
	.btn-export {
		margin-left: auto;
	}
</style>
