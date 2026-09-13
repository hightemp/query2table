<script lang="ts">
	import {
		runState,
		startNewRun,
		cancelCurrentRun,
		pauseCurrentRun,
		resumeCurrentRun,
		confirmCurrentSchema,
		resetRun,
	} from '$lib/stores/run';
	import type { SchemaColumn } from '$lib/types';
	import type { RunRow } from '$lib/stores/run';
	import type { StopConditions } from '$lib/api/tauri';
	import SchemaEditor from '$lib/components/run/SchemaEditor.svelte';
	import ResultsTable from '$lib/components/run/ResultsTable.svelte';
	import RowDetailPanel from '$lib/components/run/RowDetailPanel.svelte';
	import ProgressBar from '$lib/components/run/ProgressBar.svelte';
	import RunControls from '$lib/components/run/RunControls.svelte';
	import ExportDialog from '$lib/components/run/ExportDialog.svelte';
	import RunStatusPanel from '$lib/components/run/RunStatusPanel.svelte';
	import ImageGallery from '$lib/components/run/ImageGallery.svelte';
	import LinkList from '$lib/components/run/LinkList.svelte';
	import ResearchView from '$lib/components/run/ResearchView.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import LlmIssues from '$lib/components/run/LlmIssues.svelte';
	import {
		ChevronDownIcon,
		ChevronUpIcon,
		TableIcon,
		ImageIcon,
		LinkIcon,
		BrainIcon,
	} from '@lucide/svelte';
	import { settings } from '$lib/stores/settings';

	let query = $state('');
	let runType = $state<'table' | 'images' | 'links' | 'research'>('table');
	let selectedRow = $state<RunRow | null>(null);
	let submitError = $state('');
	let showExport = $state(false);
	let showStopConditions = $state(false);

	// Stop conditions with defaults
	let targetRows = $state('50');
	let maxBudget = $state('1.00');
	let maxDuration = $state('600');

	let isIdle = $derived($runState.status === 'idle');
	let isSchemaReview = $derived($runState.status === 'schema_review');
	let isSchemaPaused = $derived(
		$runState.status === 'paused' && $runState.pausedFrom === 'schema_review'
	);
	let isActive = $derived(
		$runState.status === 'running' ||
			$runState.status === 'paused' ||
			$runState.status === 'pending'
	);
	let isFinished = $derived(
		$runState.status === 'completed' ||
			$runState.status === 'failed' ||
			$runState.status === 'cancelled'
	);
	let showResults = $derived(isActive || isFinished || isSchemaReview);
	let isImageRun = $derived($runState.runType === 'images');
	let isLinkRun = $derived($runState.runType === 'links');
	let isResearchRun = $derived($runState.runType === 'research');
	let columnNames = $derived($runState.schema.map((c) => c.name));

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!query.trim()) return;
		submitError = '';
		try {
			const sc: StopConditions = {};
			const rows = parseInt(targetRows);
			if (!isNaN(rows) && rows > 0) sc.target_row_count = rows;
			const budget = parseFloat(maxBudget);
			if (!isNaN(budget) && budget > 0) sc.max_budget_usd = budget;
			const dur = parseInt(maxDuration);
			if (!isNaN(dur) && dur > 0) sc.max_duration_seconds = dur;
			await startNewRun(query, runType, sc);
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

	let queryExpanded = $state(false);
	const modes = [
		{
			value: 'table' as const,
			label: 'Table',
			icon: TableIcon,
			description: 'Find entities and compare their details in a table with sources.',
		},
		{
			value: 'images' as const,
			label: 'Images',
			icon: ImageIcon,
			description: 'Find and browse images with links to their original sources.',
		},
		{
			value: 'links' as const,
			label: 'Links',
			icon: LinkIcon,
			description: 'Find relevant pages and resources with short descriptions.',
		},
		{
			value: 'research' as const,
			label: 'Research',
			icon: BrainIcon,
			description: 'Explore a question and get a written answer with sources.',
		},
	];
	let provider = $derived($settings.get('llm_provider') ?? 'openrouter');
	const providerNames: Record<string, string> = {
		openrouter: 'OpenRouter',
		ollama: 'Ollama',
		ollama_cloud: 'Ollama Cloud',
		openai_compatible: 'OpenAI-compatible',
	};
	let model = $derived(
		$settings.get(
			(
				{
					openrouter: 'openrouter_model',
					ollama: 'ollama_model',
					ollama_cloud: 'ollama_cloud_model',
					openai_compatible: 'openai_model',
				} as Record<string, string>
			)[provider]
		) ?? 'No model selected'
	);
</script>

<div class="query-page">
	{#if isIdle}
		<header class="page-header">
			<div>
				<h1>New Research Query</h1>
				<p>Turn a question into useful, sourced results.</p>
			</div>
		</header>
		<div class="query-scroll">
			<form class="query-form" onsubmit={handleSubmit}>
				<div class="mode-toggle" role="group" aria-label="Result format">
					{#each modes as mode}<button
							type="button"
							class="mode-btn"
							class:active={runType === mode.value}
							aria-pressed={runType === mode.value}
							onclick={() => {
								runType = mode.value;
							}}><mode.icon size={18} />{mode.label}</button
						>{/each}
				</div>
				<p class="mode-description">{modes.find((mode) => mode.value === runType)?.description}</p>
				<label class="query-label" for="research-query">What would you like to find?</label>
				<textarea
					id="research-query"
					class="query-input"
					bind:value={query}
					placeholder="e.g. Find YouTube channels about building robots, with their language, focus and website…"
					rows={5}></textarea>
				<div class="connection-summary">
					<span>{providerNames[provider] ?? provider}</span><span class="model-name" title={model}
						>{model}</span
					><a href="/settings">Configure</a>
				</div>
				<button
					type="button"
					class="stop-toggle"
					aria-expanded={showStopConditions}
					onclick={() => {
						showStopConditions = !showStopConditions;
					}}
				>
					{#if showStopConditions}<ChevronUpIcon size={16} />{:else}<ChevronDownIcon
							size={16}
						/>{/if}Stop Conditions
					<span
						>{targetRows}
						{runType === 'table' ? 'rows' : runType === 'research' ? 'steps' : runType} · ${maxBudget}
						· {Math.round(Number(maxDuration) / 60)} min</span
					>
				</button>
				{#if showStopConditions}
					<div class="stop-conditions">
						<label for="targetRows"
							>{runType === 'images'
								? 'Max Images'
								: runType === 'links'
									? 'Max Links'
									: runType === 'research'
										? 'Max Steps'
										: 'Target Rows'}<input
								id="targetRows"
								type="number"
								min="1"
								bind:value={targetRows}
							/></label
						>
						<label for="maxBudget"
							>Max Cost ($)<input
								id="maxBudget"
								type="number"
								min="0.01"
								step="0.01"
								bind:value={maxBudget}
							/></label
						>
						<label for="maxDuration"
							>Max Duration (s)<input
								id="maxDuration"
								type="number"
								min="10"
								bind:value={maxDuration}
							/></label
						>
					</div>
				{/if}
				{#if submitError}<ErrorNotice error={submitError} />{/if}
				<div class="query-actions">
					<button type="submit" class="button primary" disabled={!query.trim()}
						>{runType === 'images'
							? 'Search Images'
							: runType === 'links'
								? 'Find Links'
								: 'Start Research'}</button
					>
				</div>
			</form>
		</div>
	{/if}
	{#if showResults}
		<header class="run-header">
			<div class="run-query-display">
				<span class="eyebrow"
					>{modes.find((mode) => mode.value === $runState.runType)?.label ?? 'Results'}</span
				>
				<h2 class:expanded={queryExpanded}>{$runState.query}</h2>
				{#if $runState.query.length > 90}<button
						class="query-expand"
						onclick={() => {
							queryExpanded = !queryExpanded;
						}}
						aria-expanded={queryExpanded}>{queryExpanded ? 'Show less' : 'Show full query'}</button
					>{/if}
			</div>
			<RunControls
				status={$runState.status}
				pending={$runState.controlPending}
				onpause={pauseCurrentRun}
				onresume={resumeCurrentRun}
				oncancel={cancelCurrentRun}
				onreset={handleReset}
				onexport={() => {
					showExport = true;
				}}
				showExport={isFinished &&
					($runState.rows.length > 0 ||
						$runState.imageResults.length > 0 ||
						$runState.linkResults.length > 0 ||
						!!$runState.researchAnswer)}
			/>
		</header>
		<div class="run-summary">
			{#if isActive || isSchemaReview}<RunStatusPanel
					status={$runState.status}
					runType={$runState.runType}
					activity={$runState.activity}
				/>{/if}
			<ProgressBar
				stats={$runState.progress}
				status={$runState.status}
				runType={$runState.runType}
			/>
		</div>
		{#if $runState.error || $runState.controlError || $runState.llmIssues.length}
			<div class="run-notices">
				{#if $runState.error}<ErrorNotice error={$runState.error} />{/if}
				{#if $runState.controlError}<ErrorNotice
						error={$runState.controlError}
						context="control"
					/>{/if}
				<LlmIssues issues={$runState.llmIssues} runStatus={$runState.status} />
			</div>
		{/if}
		{#if isSchemaReview || isSchemaPaused}
			<div class="schema-workspace" hidden={!isSchemaReview}>
				<SchemaEditor
					columns={$runState.schema}
					pending={$runState.controlPending === 'confirm_schema'}
					onconfirm={handleSchemaConfirm}
					oncancel={handleSchemaCancel}
				/>
			</div>
			{#if isSchemaPaused}<div class="empty-state">
					Schema review is paused. Resume to continue editing.
				</div>{/if}
		{:else}
			<div class="result-workspace">
				{#if isImageRun}<ImageGallery images={$runState.imageResults} />
				{:else if isLinkRun}<LinkList links={$runState.linkResults} />
				{:else if isResearchRun}<ResearchView
						steps={$runState.researchSteps}
						answer={$runState.researchAnswer}
						running={$runState.status === 'running' || $runState.status === 'pending'}
					/>
				{:else}<ResultsTable
						schema={$runState.schema}
						rows={$runState.rows}
						onrowclick={(row) => {
							selectedRow = row;
						}}
					/>{/if}
			</div>
		{/if}
	{/if}
	{#if selectedRow}<RowDetailPanel
			row={selectedRow}
			columns={columnNames}
			onclose={() => {
				selectedRow = null;
			}}
		/>{/if}
	{#if showExport && $runState.runId}<ExportDialog
			runId={$runState.runId}
			runType={$runState.runType}
			onclose={() => {
				showExport = false;
			}}
		/>{/if}
</div>

<style>
	.query-page {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
	}
	.query-scroll {
		min-height: 0;
		overflow: auto;
		scrollbar-gutter: stable;
		padding-right: 12px;
	}
	.query-form {
		max-width: 900px;
		background: var(--app-panel);
		border: 1px solid var(--app-border);
		padding: 24px;
		border-radius: 12px;
	}
	.mode-toggle {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}
	.mode-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		flex: 1;
		min-width: 100px;
		padding: 10px 12px;
		border: 1px solid var(--app-border);
		border-radius: 8px;
		background: var(--app-bg);
		color: var(--app-muted);
	}
	.mode-btn.active {
		color: var(--app-accent);
		border-color: var(--app-accent);
		background: color-mix(in srgb, var(--app-accent) 9%, var(--app-panel));
		font-weight: 650;
	}
	.mode-description {
		color: var(--app-muted);
		margin: 12px 0 24px;
		font-size: 13px;
	}
	.query-label {
		font-weight: 600;
		display: block;
		margin-bottom: 8px;
	}
	.query-input {
		display: block;
		width: 100%;
		resize: vertical;
		min-height: 120px;
		padding: 14px 16px;
		border: 1px solid var(--app-border);
		border-radius: 8px;
		background: var(--app-bg);
		color: var(--app-text);
		line-height: 1.6;
	}
	.connection-summary {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 6px 12px;
		color: var(--app-muted);
		font-size: 12px;
		padding: 12px 0 20px;
	}
	.model-name {
		max-width: 380px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.connection-summary a {
		color: var(--app-accent);
		margin-left: auto;
	}
	.stop-toggle {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 8px;
		background: transparent;
		border: 0;
		padding: 8px 0;
		color: var(--app-text);
		text-align: left;
	}
	.stop-toggle span {
		color: var(--app-muted);
		font-size: 12px;
	}
	.stop-conditions {
		display: flex;
		flex-wrap: wrap;
		gap: 12px;
		margin: 12px 0;
	}
	.stop-conditions label {
		flex: 1 1 130px;
		font-size: 12px;
		color: var(--app-muted);
	}
	.stop-conditions input {
		display: block;
		width: 100%;
		margin-top: 4px;
		padding: 7px 10px;
		color: var(--app-text);
		background: var(--app-bg);
		border: 1px solid var(--app-border);
		border-radius: 8px;
	}
	.query-actions {
		display: flex;
		justify-content: flex-end;
		margin-top: 16px;
	}
	.run-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		flex-wrap: wrap;
		gap: 12px 20px;
		flex-shrink: 0;
		padding-bottom: 12px;
	}
	.run-query-display {
		flex: 1 1 240px;
		min-width: 0;
	}
	.eyebrow {
		color: var(--app-muted);
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}
	h2 {
		font-size: 19px;
		font-weight: 650;
		line-height: 1.35;
		overflow-wrap: anywhere;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
		margin: 4px 0 0;
	}
	h2.expanded {
		display: block;
		max-height: 120px;
		overflow: auto;
	}
	.query-expand {
		font-size: 12px;
		color: var(--app-accent);
		background: transparent;
		padding: 4px 0;
		border: 0;
	}
	.run-summary {
		display: flex;
		flex-direction: column;
		gap: 8px;
		flex-shrink: 0;
		padding: 0 0 12px;
	}
	.run-notices {
		max-height: 28vh;
		overflow: auto;
		flex-shrink: 0;
		padding-right: 12px;
		scrollbar-gutter: stable;
		margin-bottom: 8px;
	}
	.result-workspace,
	.schema-workspace {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
	}
	.schema-workspace[hidden] {
		display: none;
	}
</style>
