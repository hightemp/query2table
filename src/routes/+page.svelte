<script lang="ts">
	import { runState, startNewRun, cancelCurrentRun, pauseCurrentRun, resumeCurrentRun, confirmCurrentSchema, resetRun } from '$lib/stores/run';
	import type { SchemaColumn } from '$lib/types';
	import type { RunRow } from '$lib/stores/run';
	import SchemaEditor from '$lib/components/run/SchemaEditor.svelte';
	import ResultsTable from '$lib/components/run/ResultsTable.svelte';
	import RowDetailPanel from '$lib/components/run/RowDetailPanel.svelte';
	import ProgressBar from '$lib/components/run/ProgressBar.svelte';
	import RunControls from '$lib/components/run/RunControls.svelte';
	import ExportDialog from '$lib/components/run/ExportDialog.svelte';

	let query = $state('');
	let selectedRow = $state<RunRow | null>(null);
	let submitError = $state('');
	let showExport = $state(false);

	let isIdle = $derived($runState.status === 'idle');
	let isSchemaReview = $derived($runState.status === 'schema_review');
	let isActive = $derived(
		$runState.status === 'running' || $runState.status === 'paused' || $runState.status === 'pending'
	);
	let isFinished = $derived(
		$runState.status === 'completed' || $runState.status === 'failed' || $runState.status === 'cancelled'
	);
	let showResults = $derived(isActive || isFinished || isSchemaReview);
	let columnNames = $derived($runState.schema.map((c) => c.name));

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!query.trim()) return;
		submitError = '';
		try {
			await startNewRun(query);
		} catch (err) {
			submitError = String(err);
		}
	}

	function handleSchemaConfirm(columns: SchemaColumn[]) {
		confirmCurrentSchema(columns);
	}

	function handleSchemaCancel() {
		cancelCurrentRun();
	}

	function handleReset() {
		resetRun();
		query = '';
		selectedRow = null;
		submitError = '';
		showExport = false;
	}
</script>

<div class="query-page">
	{#if isIdle}
		<h1>New Research Query</h1>
		<p class="subtitle">Describe what you want to research. Query2Table will search, extract, and organize results into a structured table.</p>

		<form class="query-form" onsubmit={handleSubmit}>
			<textarea
				class="query-input"
				bind:value={query}
				placeholder="e.g. Find all YC-backed AI startups from 2024 with their funding amount, CEO name, and website..."
				rows={4}
			></textarea>
			{#if submitError}
				<p class="error-msg">{submitError}</p>
			{/if}
			<div class="query-actions">
				<button type="submit" class="btn-primary" disabled={!query.trim()}>
					Start Research
				</button>
			</div>
		</form>
	{/if}

	{#if showResults}
		<div class="run-header">
			<div class="run-query-display">
				<h2>{$runState.query}</h2>
			</div>
			<RunControls
				status={$runState.status}
				onpause={pauseCurrentRun}
				onresume={resumeCurrentRun}
				oncancel={cancelCurrentRun}
				onreset={handleReset}
				onexport={() => { showExport = true; }}
				showExport={isFinished && $runState.rows.length > 0}
			/>
		</div>

		{#if $runState.error}
			<div class="error-banner">
				<strong>Error:</strong> {$runState.error}
			</div>
		{/if}

		{#if isActive || isFinished}
			<ProgressBar stats={$runState.progress} status={$runState.status} />
		{/if}

		{#if isSchemaReview}
			<SchemaEditor
				columns={$runState.schema}
				onconfirm={handleSchemaConfirm}
				oncancel={handleSchemaCancel}
			/>
		{/if}

		{#if $runState.schema.length > 0 && !isSchemaReview}
			<ResultsTable
				schema={$runState.schema}
				rows={$runState.rows}
				onrowclick={(row) => { selectedRow = row; }}
			/>
		{/if}
	{/if}

	{#if selectedRow}
		<RowDetailPanel
			row={selectedRow}
			columns={columnNames}
			onclose={() => { selectedRow = null; }}
		/>
	{/if}

	{#if showExport && $runState.runId}
		<ExportDialog
			runId={$runState.runId}
			onclose={() => { showExport = false; }}
		/>
	{/if}
</div>

<style>
	.query-page {
		width: 100%;
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

	.query-form {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.query-input {
		width: 100%;
		padding: 12px 16px;
		border: 2px solid var(--color-surface-300-700);
		border-radius: 8px;
		background: var(--color-surface-100-900);
		color: inherit;
		font-size: 1rem;
		font-family: inherit;
		resize: vertical;
		transition: border-color 0.15s;
	}

	.query-input:focus {
		outline: none;
		border-color: var(--color-primary-500);
	}

	.query-actions {
		display: flex;
		justify-content: flex-end;
	}

	.btn-primary {
		padding: 10px 24px;
		background: var(--color-primary-500);
		color: white;
		border: none;
		border-radius: 8px;
		font-size: 1rem;
		font-weight: 600;
		cursor: pointer;
		transition: background 0.15s;
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--color-primary-600);
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.error-msg {
		color: var(--color-error-500);
		font-size: 0.9rem;
	}

	.run-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 16px;
		margin-bottom: 16px;
	}

	.run-query-display h2 {
		font-size: 1.3rem;
		font-weight: 700;
		margin: 0;
	}

	.error-banner {
		padding: 10px 16px;
		border: 1px solid var(--color-error-500);
		border-radius: 8px;
		background: rgba(239, 68, 68, 0.1);
		color: var(--color-error-500);
		margin-bottom: 12px;
		font-size: 0.9rem;
	}
</style>
