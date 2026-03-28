<script lang="ts">
	import { onMount } from 'svelte';
	import { listRuns, deleteRun, getRunSchema, getRunRows, type RunSchemaInfo, type EntityRowInfo } from '$lib/api/tauri';
	import type { RunInfo, SchemaColumn } from '$lib/types';
	import type { RunRow } from '$lib/stores/run';
	import ResultsTable from '$lib/components/run/ResultsTable.svelte';
	import RowDetailPanel from '$lib/components/run/RowDetailPanel.svelte';
	import ExportDialog from '$lib/components/run/ExportDialog.svelte';
	import { TrashIcon, ExternalLinkIcon, ArrowLeftIcon, DownloadIcon } from '@lucide/svelte';

	let runs = $state<RunInfo[]>([]);
	let loading = $state(true);
	let error = $state('');

	// View state
	let viewingRun = $state<RunInfo | null>(null);
	let viewSchema = $state<SchemaColumn[]>([]);
	let viewRows = $state<RunRow[]>([]);
	let viewLoading = $state(false);
	let selectedRow = $state<RunRow | null>(null);
	let showExport = $state(false);

	onMount(async () => {
		await loadRuns();
	});

	async function loadRuns() {
		loading = true;
		error = '';
		try {
			runs = await listRuns(100);
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function handleView(run: RunInfo) {
		viewLoading = true;
		error = '';
		selectedRow = null;
		try {
			const [schemaInfo, rows] = await Promise.all([
				getRunSchema(run.id),
				getRunRows(run.id),
			]);
			viewSchema = schemaInfo?.columns ?? [];
			viewRows = rows.map((r) => ({
				id: r.id,
				data: r.data as Record<string, unknown>,
				confidence: r.confidence,
			}));
			viewingRun = run;
		} catch (e) {
			error = String(e);
		} finally {
			viewLoading = false;
		}
	}

	function handleBack() {
		viewingRun = null;
		viewSchema = [];
		viewRows = [];
		selectedRow = null;
		showExport = false;
	}

	async function handleDelete(runId: string) {
		try {
			await deleteRun(runId);
			runs = runs.filter((r) => r.id !== runId);
		} catch (e) {
			error = String(e);
		}
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
				<span class="badge {statusClass(viewingRun.status)}">{viewingRun.status}</span>
			</div>
			<div class="view-meta">
				<span>{formatDate(viewingRun.created_at)}</span>
				<span>{viewRows.length} rows</span>
				{#if viewRows.length > 0}
					<button class="btn-export" onclick={() => { showExport = true; }}>
						<DownloadIcon size={14} />
						Export
					</button>
				{/if}
			</div>
		</div>

		{#if showExport}
			<ExportDialog
				runId={viewingRun.id}
				onclose={() => { showExport = false; }}
			/>
		{/if}

		{#if viewSchema.length > 0 && viewRows.length > 0}
			<ResultsTable
				schema={viewSchema}
				rows={viewRows}
				onrowclick={(row) => { selectedRow = row; }}
			/>
		{:else}
			<div class="empty-state">No results found for this run.</div>
		{/if}

		{#if selectedRow}
			<RowDetailPanel
				row={selectedRow}
				columns={viewSchema.map((c) => c.name)}
				onclose={() => { selectedRow = null; }}
			/>
		{/if}
	{:else}
		<h1>Run History</h1>
		<p class="subtitle">View past research queries and their results.</p>

		{#if error}
			<p class="error-msg">{error}</p>
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
							<span class="badge {statusClass(run.status)}">{run.status}</span>
						</div>
						<div class="run-card-meta">
							<span class="run-date">{formatDate(run.created_at)}</span>
							{#if run.error}
								<span class="run-error">{run.error}</span>
							{/if}
						</div>
						<div class="run-card-actions">
							<button class="action-link" onclick={() => handleView(run)} disabled={viewLoading}>
								<ExternalLinkIcon size={14} />
								View
							</button>
							<button class="action-btn danger" onclick={() => handleDelete(run.id)}>
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

<style>
	.history-page {
		width: 100%;
		display: flex;
		flex-direction: column;
		min-height: 0;
		flex: 1;
		overflow: hidden;
	}

	h1 {
		font-size: 1.8rem;
		font-weight: 700;
		margin-bottom: 8px;
	}

	.subtitle {
		color: var(--color-surface-600-400);
		margin-bottom: 24px;
	}

	.empty-state {
		text-align: center;
		padding: 48px 24px;
		border: 2px dashed var(--color-surface-300-700);
		border-radius: 12px;
		color: var(--color-surface-400-600);
	}

	.error-msg {
		color: var(--color-error-500);
		margin-bottom: 12px;
	}

	.runs-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
		overflow-y: auto;
		flex: 1;
		min-height: 0;
	}

	.run-card {
		border: 1px solid var(--color-surface-300-700);
		border-radius: 10px;
		padding: 16px;
		background: var(--color-surface-100-900);
	}

	.run-card-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 12px;
		margin-bottom: 8px;
	}

	.run-query {
		font-weight: 600;
		font-size: 1rem;
		word-break: break-word;
	}

	.badge {
		padding: 3px 10px;
		border-radius: 10px;
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: capitalize;
		white-space: nowrap;
		background: var(--color-surface-200-800);
		color: var(--color-surface-600-400);
	}

	.badge-success {
		background: rgba(34, 197, 94, 0.15);
		color: var(--color-success-500, #22c55e);
	}

	.badge-error {
		background: rgba(239, 68, 68, 0.15);
		color: var(--color-error-500);
	}

	.badge-running {
		background: var(--color-primary-100, rgba(59, 130, 246, 0.15));
		color: var(--color-primary-500);
	}

	.badge-cancelled {
		background: var(--color-surface-200-800);
		color: var(--color-surface-600-400);
	}

	.run-card-meta {
		font-size: 0.85rem;
		color: var(--color-surface-600-400);
		margin-bottom: 10px;
		display: flex;
		gap: 16px;
	}

	.run-error {
		color: var(--color-error-500);
		font-size: 0.8rem;
	}

	.run-card-actions {
		display: flex;
		gap: 12px;
	}

	.action-link {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 0.85rem;
		color: var(--color-primary-500);
		text-decoration: none;
	}

	.action-link:hover {
		text-decoration: underline;
	}

	.action-btn {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 0.85rem;
		background: none;
		border: none;
		cursor: pointer;
		color: inherit;
	}

	.action-btn.danger {
		color: var(--color-error-500);
	}

	.action-btn.danger:hover {
		text-decoration: underline;
	}

	.view-header {
		margin-bottom: 20px;
	}

	.btn-back {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 0.9rem;
		background: none;
		border: none;
		color: var(--color-primary-500);
		cursor: pointer;
		padding: 4px 0;
		margin-bottom: 12px;
	}

	.btn-back:hover {
		text-decoration: underline;
	}

	.view-title {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-bottom: 6px;
	}

	.view-title h1 {
		margin-bottom: 0;
	}

	.view-meta {
		display: flex;
		align-items: center;
		gap: 16px;
		font-size: 0.85rem;
		color: var(--color-surface-600-400);
		margin-bottom: 16px;
	}

	.btn-export {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 14px;
		font-size: 0.85rem;
		font-weight: 600;
		color: var(--color-primary-500);
		background: none;
		border: 1px solid var(--color-primary-500);
		border-radius: 6px;
		cursor: pointer;
		margin-left: auto;
	}

	.btn-export:hover {
		background: rgba(59, 130, 246, 0.1);
	}

	button.action-link {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 0.85rem;
		color: var(--color-primary-500);
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
	}

	button.action-link:hover {
		text-decoration: underline;
	}

	button.action-link:disabled {
		opacity: 0.5;
		cursor: wait;
	}
</style>
