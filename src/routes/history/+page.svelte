<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
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
	let selectedRow = $state<RunRow | null>(null);
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
		selectedRow = null;
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
		selectedRow = null;
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

	function statusClass(status: string): string {
		if (status === 'completed') return 'badge-success';
		if (status === 'failed') return 'badge-error';
		if (status === 'running') return 'badge-running';
		if (status === 'cancelled') return 'badge-cancelled';
		return '';
	}
</script>

<div class="history-page">
	{#if viewingRun}
		<div class="view-header">
			<button class="btn-back" onclick={handleBack}>
				<ArrowLeftIcon size={16} />
				Back to History
			</button>
			<div class="view-title">
				<h1>{viewingRun.query}</h1>
				<span class="badge {statusClass(viewingRun.status)}"
					>{viewingRun.status.replaceAll('_', ' ')}</span
				>
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
						class="btn-export"
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
				<div class="empty-state">Loading run results…</div>
			{:else if viewingRun.run_type === 'images'}
				{#if viewImages.length > 0}
					<ImageGallery images={viewImages} />
				{:else}
					<div class="empty-state">No images found for this run.</div>
				{/if}
			{:else if viewingRun.run_type === 'links'}
				{#if viewLinks.length > 0}
					<LinkList links={viewLinks} />
				{:else}
					<div class="empty-state">No links found for this run.</div>
				{/if}
			{:else if viewingRun.run_type === 'research'}
				{#if viewResearchAnswer || viewResearchSteps.length > 0}
					<ResearchView steps={viewResearchSteps} answer={viewResearchAnswer} />
				{:else}
					<div class="empty-state">No research output for this run.</div>
				{/if}
			{:else if viewSchema.length > 0 && viewRows.length > 0}
				<ResultsTable
					schema={viewSchema}
					rows={viewRows}
					onrowclick={(row) => {
						selectedRow = row;
					}}
				/>
			{:else}
				<div class="empty-state">No results found for this run.</div>
			{/if}
		</div>
		{#if selectedRow}
			<RowDetailPanel
				row={selectedRow}
				columns={viewSchema.map((c) => c.name)}
				onclose={() => {
					selectedRow = null;
				}}
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
			<div class="empty-state">Loading...</div>
		{:else if runs.length === 0}
			<div class="empty-state">
				<p>No runs yet. Start a new query to see results here.</p>
			</div>
		{:else}
			<div class="runs-list">
				{#each runs as run}
					<div class="run-card">
						<div class="run-card-header">
							<span class="run-query">{run.query}</span>
							<div class="header-badges">
								{#if run.run_type === 'images'}
									<span class="badge badge-type"><ImageIcon size={12} /> Images</span>
								{:else if run.run_type === 'links'}
									<span class="badge badge-type"><LinkIcon size={12} /> Links</span>
								{:else if run.run_type === 'research'}
									<span class="badge badge-type"><BrainIcon size={12} /> Research</span>
								{:else}
									<span class="badge badge-type"><TableIcon size={12} /> Table</span>
								{/if}
								<span class="badge {statusClass(run.status)}"
									>{run.status.replaceAll('_', ' ')}</span
								>
							</div>
						</div>
						<div class="run-card-meta">
							<span class="run-date">{formatDate(run.created_at)}</span>
							{#if run.error}
								<span class="run-error">{presentError(run.error).title}</span>
							{/if}
						</div>
						<div class="run-card-actions">
							<button class="action-link" onclick={() => handleView(run)} disabled={viewLoading}>
								<ExternalLinkIcon size={14} />
								View
							</button>
							<button
								class="action-btn danger"
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
		border-radius: 12px;
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
		font-size: 15px;
		overflow-wrap: anywhere;
		flex: 1 1 260px;
	}
	.header-badges {
		display: flex;
		align-items: flex-start;
		gap: 6px;
		flex-wrap: wrap;
	}
	.badge {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 8px;
		border-radius: 20px;
		background: var(--app-subtle);
		color: var(--app-muted);
		font-size: 11px;
		text-transform: capitalize;
		white-space: nowrap;
	}
	.badge-success {
		color: var(--color-success-500);
		background: color-mix(in srgb, var(--color-success-500) 10%, transparent);
	}
	.badge-error {
		color: var(--color-error-500);
		background: color-mix(in srgb, var(--color-error-500) 10%, transparent);
	}
	.badge-running {
		color: var(--app-accent);
		background: color-mix(in srgb, var(--app-accent) 10%, transparent);
	}
	.run-card-meta {
		display: flex;
		flex-wrap: wrap;
		gap: 8px 16px;
		font-size: 12px;
		color: var(--app-muted);
		margin: 8px 0 12px;
	}
	.run-error {
		color: var(--color-error-500);
	}
	.run-card-actions {
		display: flex;
		gap: 8px;
	}
	.action-link,
	.action-btn,
	.btn-back,
	.btn-export {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 7px 12px;
		border-radius: 8px;
		border: 1px solid var(--app-border);
		background: var(--app-panel);
		color: var(--app-text);
		font-size: 13px;
	}
	.action-link {
		color: var(--app-accent);
	}
	.danger {
		color: var(--color-error-500);
	}
	button:hover:not(:disabled) {
		background: var(--app-subtle);
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
		font-size: 20px;
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
		font-size: 12px;
		color: var(--app-muted);
	}
	.btn-export {
		margin-left: auto;
	}
</style>
