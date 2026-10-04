<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import {
		runState,
		startNewRun,
		cancelCurrentRun,
		pauseCurrentRun,
		resumeCurrentRun,
		confirmCurrentSchema,
		resetRun,
		askFollowUp,
		dismissNotices,
	} from '$lib/stores/run';
	import type { RunInfo, SchemaColumn } from '$lib/types';
	import { listRuns } from '$lib/api/tauri';
	import SchemaEditor from '$lib/components/run/SchemaEditor.svelte';
	import ResultsTable from '$lib/components/run/ResultsTable.svelte';
	import RowDetailPanel from '$lib/components/run/RowDetailPanel.svelte';
	import CostSummary from '$lib/components/run/CostSummary.svelte';
	import ProgressBar from '$lib/components/run/ProgressBar.svelte';
	import RunProgress from '$lib/components/run/RunProgress.svelte';
	import RunControls from '$lib/components/run/RunControls.svelte';
	import ExportDialog from '$lib/components/run/ExportDialog.svelte';
	import RunStatusPanel from '$lib/components/run/RunStatusPanel.svelte';
	import ImageGallery from '$lib/components/run/ImageGallery.svelte';
	import LinkList from '$lib/components/run/LinkList.svelte';
	import ResearchView from '$lib/components/run/ResearchView.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import RunNotices from '$lib/components/run/RunNotices.svelte';
	import EmptyState from '$lib/components/common/EmptyState.svelte';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import {
		ChevronDownIcon,
		ChevronUpIcon,
		TableIcon,
		ImageIcon,
		LinkIcon,
		BrainIcon,
		TriangleAlertIcon,
		HistoryIcon,
	} from '@lucide/svelte';
	import { settings } from '$lib/stores/settings';
	import { toast } from '$lib/stores/toasts';
	import { hasMod, modKey } from '$lib/utils/shortcuts';
	import { errorText } from '$lib/utils/errors';
	import { configurationProblems } from '$lib/utils/runConfig';
	import { RowSelection } from '$lib/utils/rowSelection.svelte';
	import {
		DEFAULT_STOP_INPUT,
		stopSettingKeys,
		formatMinutes,
		formatUsd,
		parseStopConditions,
		stopInputFromSettings,
		type StopConditionInput,
	} from '$lib/utils/stopConditions';

	type Mode = 'table' | 'images' | 'links' | 'research';
	let query = $state('');
	let runType = $state<Mode>('table');
	const selection = new RowSelection();
	let selectedRow = $derived($runState.rows.find((row) => row.id === selection.id) ?? null);
	let submitError = $state('');
	let showExport = $state(false);
	let showStopConditions = $state(false);
	let confirmCancel = $state(false);
	let queryInput = $state<HTMLTextAreaElement>();

	// Stop conditions start from the values used last time (stored in settings).
	let stopInput = $state<StopConditionInput>(
		untrack(() =>
			$settings.size ? stopInputFromSettings($settings, runType) : { ...DEFAULT_STOP_INPUT }
		)
	);
	let stopEdited = $state(false);
	$effect(() => {
		if ($settings.size && !stopEdited) stopInput = stopInputFromSettings($settings, runType);
	});
	// Research counts steps, the other modes count results: switching modes loads that limit.
	let stopMode = untrack(() => runType);
	$effect(() => {
		if (runType === stopMode) return;
		stopMode = runType;
		stopInput = { ...stopInput, target: stopInputFromSettings($settings, runType).target };
	});
	let stopParsed = $derived(parseStopConditions(stopInput, runType));
	let stopErrors = $derived(stopParsed.errors);
	let stopInvalid = $derived(!stopParsed.conditions);
	let configProblems = $derived(configurationProblems($settings));
	let canSubmit = $derived(!!query.trim() && !configProblems.length);

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

	async function rememberStopConditions(input: StopConditionInput, mode: string) {
		const conditions = parseStopConditions(input, mode).conditions;
		if (!conditions) return;
		const keys = stopSettingKeys(mode);
		const values: [string, string][] = [
			[keys.target, String(conditions.target_row_count)],
			[keys.budget, String(conditions.max_budget_usd)],
			[keys.duration, String(conditions.max_duration_seconds)],
		];
		try {
			for (const [key, value] of values)
				if ($settings.get(key) !== value) await settings.save(key, value);
		} catch {
			toast('Could not remember these stop conditions for the next run.', 'error');
		}
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!canSubmit) return;
		const conditions = stopParsed.conditions;
		if (!conditions) {
			showStopConditions = true;
			return;
		}
		submitError = '';
		const input = { ...stopInput };
		try {
			await startNewRun(query, runType, conditions);
			stopEdited = false;
			void rememberStopConditions(input, runType);
		} catch (err) {
			submitError = errorText(err);
		}
	}

	function handleSchemaConfirm(columns: SchemaColumn[]) {
		confirmCurrentSchema(columns);
	}

	function resetPage(nextQuery: string) {
		resetRun();
		query = nextQuery;
		selection.clear();
		submitError = '';
		showExport = false;
		confirmCancel = false;
		void loadRecent();
	}

	function handleReset() {
		resetPage('');
	}

	function handleEdit() {
		const previousType = $runState.runType as Mode;
		resetPage($runState.query);
		if (modes.some((mode) => mode.value === previousType)) runType = previousType;
	}

	$effect(() => {
		// Focus the query when the form appears, e.g. after New query or Edit query.
		if (isIdle && queryInput) queryInput.focus();
	});

	let queryExpanded = $state(false);
	const modes = [
		{
			value: 'table' as const,
			label: 'Table',
			icon: TableIcon,
			description: 'Find entities and compare their details in a table with sources.',
			action: 'Build Table',
			unit: 'rows',
			targetLabel: 'Target rows',
			examples: [
				'Open-source vector databases with license, language and GitHub stars',
				'EU climate-tech startups founded after 2020 with funding stage and website',
				'Robotics YouTube channels with language, focus and subscriber count',
			],
		},
		{
			value: 'images' as const,
			label: 'Images',
			icon: ImageIcon,
			description: 'Find and browse images with links to their original sources.',
			action: 'Search Images',
			unit: 'images',
			targetLabel: 'Max images',
			examples: [
				'Brutalist libraries built after 1960',
				'Hand-drawn maps of fantasy worlds',
				'Diagrams of the Krebs cycle',
			],
		},
		{
			value: 'links' as const,
			label: 'Links',
			icon: LinkIcon,
			description: 'Find relevant pages and resources with short descriptions.',
			action: 'Find Links',
			unit: 'links',
			targetLabel: 'Max links',
			examples: [
				'Beginner tutorials for Rust async programming',
				'Public datasets about urban air quality',
				'Engineering blogs about database migrations at scale',
			],
		},
		{
			value: 'research' as const,
			label: 'Research',
			icon: BrainIcon,
			description: 'Explore a question and get a written answer with sources.',
			action: 'Start Research',
			unit: 'steps',
			targetLabel: 'Max steps',
			examples: [
				'How do heat pumps perform in very cold climates?',
				'What changed in the EU AI Act between the draft and the final text?',
				'Which battery chemistries are used in grid storage, and why?',
			],
		},
	];
	let mode = $derived(modes.find((item) => item.value === runType) ?? modes[0]);
	let stopSummary = $derived(
		stopParsed.conditions
			? `${stopParsed.conditions.target_row_count} ${mode.unit} · ${formatUsd(stopParsed.conditions.max_budget_usd)} · ${formatMinutes(stopParsed.conditions.max_duration_seconds)}`
			: 'Check the values'
	);

	let recentRuns = $state<RunInfo[]>([]);
	async function loadRecent() {
		try {
			const runs = await listRuns(20);
			const seen = new Set<string>();
			recentRuns = runs
				.filter((run) => {
					const key = `${run.run_type}:${run.query.trim()}`;
					if (seen.has(key)) return false;
					seen.add(key);
					return true;
				})
				.slice(0, 5);
		} catch {
			// Recent queries are a convenience; the form works without them.
			recentRuns = [];
		}
	}
	onMount(() => {
		void loadRecent();
	});
	function useQuery(text: string, type: string) {
		query = text;
		if (modes.some((item) => item.value === type)) runType = type as Mode;
		queryInput?.focus();
	}

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
		) || 'No model selected'
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
			<form class="query-form" onsubmit={handleSubmit} novalidate>
				<div class="mode-toggle" role="group" aria-label="Result format">
					{#each modes as item}<button
							type="button"
							class="mode-btn"
							class:active={runType === item.value}
							aria-pressed={runType === item.value}
							onclick={() => {
								runType = item.value;
							}}><item.icon size={18} />{item.label}</button
						>{/each}
				</div>
				<p class="mode-description">{mode.description}</p>
				<label class="query-label" for="research-query">What would you like to find?</label>
				<textarea
					id="research-query"
					class="input query-input"
					bind:this={queryInput}
					bind:value={query}
					placeholder="e.g. Find YouTube channels about building robots, with their language, focus and website…"
					rows={5}
					onkeydown={(event) => {
						if (event.key === 'Enter' && hasMod(event)) {
							event.preventDefault();
							event.currentTarget.form?.requestSubmit();
						}
					}}></textarea>
				{#if !query.trim()}
					<div class="examples" aria-label="Example queries" role="group">
						<span>Try:</span>
						{#each mode.examples as example}<button
								type="button"
								class="chip"
								onclick={() => useQuery(example, runType)}>{example}</button
							>{/each}
					</div>
				{/if}
				<div class="connection-summary">
					<span>{providerNames[provider] ?? provider}</span><span class="model-name" title={model}
						>{model}</span
					><a href="/settings#settings-llm">Configure</a>
				</div>
				{#if configProblems.length}
					<div class="config-problems" role="alert">
						<TriangleAlertIcon size={16} />
						<div>
							<p class="config-title">Finish setup before starting a run</p>
							<ul>
								{#each configProblems as problem}<li>
										{problem.message}
										<a href={`/settings#settings-${problem.section}`}>Open settings</a>
									</li>{/each}
							</ul>
						</div>
					</div>
				{/if}
				<button
					type="button"
					class="stop-toggle"
					aria-expanded={showStopConditions}
					aria-controls="stop-conditions"
					onclick={() => {
						showStopConditions = !showStopConditions;
					}}
				>
					{#if showStopConditions}<ChevronUpIcon size={16} />{:else}<ChevronDownIcon
							size={16}
						/>{/if}Stop conditions
					<span class:invalid={stopInvalid}>{stopSummary}</span>
				</button>
				{#if showStopConditions}
					<div class="stop-conditions" id="stop-conditions">
						<div class="stop-fields">
							<label for="targetRows"
								>{mode.targetLabel}<input
									id="targetRows"
									class="input"
									type="number"
									min="1"
									step="1"
									inputmode="numeric"
									aria-invalid={!!stopErrors.target}
									aria-describedby={stopErrors.target ? 'targetRows-error' : undefined}
									value={stopInput.target}
									oninput={(event) => {
										stopEdited = true;
										stopInput.target = event.currentTarget.value;
									}}
								/>{#if stopErrors.target}<span class="field-error" id="targetRows-error"
										>{stopErrors.target}</span
									>{/if}</label
							>
							<label for="maxBudget"
								>Max cost (USD)<input
									id="maxBudget"
									class="input"
									type="number"
									min="0.01"
									step="0.01"
									inputmode="decimal"
									aria-invalid={!!stopErrors.budget}
									aria-describedby={stopErrors.budget ? 'maxBudget-error' : 'budget-help'}
									value={stopInput.budget}
									oninput={(event) => {
										stopEdited = true;
										stopInput.budget = event.currentTarget.value;
									}}
								/>{#if stopErrors.budget}<span class="field-error" id="maxBudget-error"
										>{stopErrors.budget}</span
									>{/if}</label
							>
							<label for="maxDuration"
								>Max duration (min)<input
									id="maxDuration"
									class="input"
									type="number"
									min="1"
									step="1"
									inputmode="numeric"
									aria-invalid={!!stopErrors.duration}
									aria-describedby={stopErrors.duration ? 'maxDuration-error' : undefined}
									value={stopInput.duration}
									oninput={(event) => {
										stopEdited = true;
										stopInput.duration = event.currentTarget.value;
									}}
								/>{#if stopErrors.duration}<span class="field-error" id="maxDuration-error"
										>{stopErrors.duration}</span
									>{/if}</label
							>
						</div>
						<p class="budget-help" id="budget-help">
							The run stops at whichever limit it reaches first. The cost limit covers reported or
							estimated charges; unpriced requests and requests already in flight can exceed it.
							These values are remembered for the next run.
						</p>
					</div>
				{/if}
				{#if submitError}<ErrorNotice error={submitError} />{/if}
				<div class="query-actions">
					<span class="shortcut-hint" aria-hidden="true"><kbd>{modKey}</kbd>+<kbd>Enter</kbd></span>
					<button
						type="submit"
						aria-keyshortcuts={modKey === '⌘' ? 'Meta+Enter' : 'Control+Enter'}
						class="button primary"
						disabled={!canSubmit}>{mode.action}</button
					>
				</div>
			</form>
			{#if recentRuns.length}
				<section class="recent" aria-labelledby="recent-title">
					<h2 id="recent-title"><HistoryIcon size={15} />Recent queries</h2>
					<ul>
						{#each recentRuns as run (run.id)}
							{@const runMode = modes.find((item) => item.value === run.run_type)}
							<li>
								<button type="button" onclick={() => useQuery(run.query, run.run_type)}>
									{#if runMode}<runMode.icon size={14} />{/if}<span>{run.query}</span>
								</button>
							</li>
						{/each}
					</ul>
				</section>
			{/if}
		</div>
	{/if}
	{#if showResults}
		<header class="run-header">
			<div class="run-query-display">
				<span class="eyebrow"
					>{modes.find((item) => item.value === $runState.runType)?.label ?? 'Results'}</span
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
				oncancel={() => {
					confirmCancel = true;
				}}
				onreset={handleReset}
				onedit={handleEdit}
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
			<div class="summary-line">
				{#if isActive || isSchemaReview}<RunStatusPanel
						status={$runState.status}
						runType={$runState.runType}
						activity={$runState.activity}
					/>{/if}
				<ProgressBar
					stats={$runState.progress}
					status={$runState.status}
					runType={$runState.runType}
					target={$runState.limits?.target_row_count ?? null}
				/>
				<div class="summary-cost"><CostSummary accounting={$runState.accounting} /></div>
			</div>
			{#if isActive}<RunProgress
					stats={$runState.progress}
					limits={$runState.limits}
					accounting={$runState.accounting}
					runType={$runState.runType}
					paused={$runState.status === 'paused'}
				/>{/if}
		</div>
		{#if $runState.error || $runState.controlError}
			<div class="run-errors">
				{#if $runState.error}<ErrorNotice error={$runState.error} />{/if}
				{#if $runState.controlError}<ErrorNotice
						error={$runState.controlError}
						context="control"
					/>{/if}
			</div>
		{/if}
		<RunNotices
			issues={$runState.llmIssues}
			accounting={$runState.accounting}
			runStatus={$runState.status}
			turnCount={$runState.researchTurns.length}
			dismissed={$runState.noticesDismissed}
			ondismiss={(dismissed) =>
				dismissNotices(dismissed).catch((error) => toast(errorText(error), 'error'))}
		/>
		{#if isSchemaReview || isSchemaPaused}
			<div class="schema-workspace" hidden={!isSchemaReview}>
				<SchemaEditor
					columns={$runState.schema}
					pending={$runState.controlPending === 'confirm_schema'}
					onconfirm={handleSchemaConfirm}
				/>
			</div>
			{#if isSchemaPaused}<EmptyState role="status"
					>Schema review is paused. Resume to continue editing.</EmptyState
				>{/if}
		{:else}
			<div class="result-workspace">
				{#if isImageRun}<ImageGallery images={$runState.imageResults} />
				{:else if isLinkRun}<LinkList links={$runState.linkResults} />
				{:else if isResearchRun}<ResearchView
						turns={$runState.researchTurns}
						running={$runState.status === 'running' || $runState.status === 'pending'}
						liveAccounting={$runState.accounting}
						limits={$runState.limits}
						onask={$runState.runId ? (question, limits) => askFollowUp(question, limits) : undefined}
					/>
				{:else}<ResultsTable
						schema={$runState.schema}
						rows={$runState.rows}
						onrowclick={(row, order) => selection.select(row.id, order)}
					/>{/if}
			</div>
		{/if}
	{/if}
	{#if selectedRow}<RowDetailPanel
			row={selectedRow}
			columns={columnNames}
			position={selection.position}
			onnavigate={selection.move}
			onclose={selection.clear}
		/>{/if}
	{#if showExport && $runState.runId}<ExportDialog
			runId={$runState.runId}
			runType={$runState.runType}
			turns={$runState.researchTurns}
			onclose={() => {
				showExport = false;
			}}
		/>{/if}
	{#if confirmCancel && (isActive || isSchemaReview)}
		<Dialog
			title="Cancel this run?"
			onclose={() => {
				confirmCancel = false;
			}}
		>
			<p>
				The run stops making new requests. Results found so far are kept and stay available in
				History.
			</p>
			{#snippet footer()}
				<button
					class="button"
					onclick={() => {
						confirmCancel = false;
					}}>Keep running</button
				>
				<button
					class="button danger outline"
					onclick={() => {
						confirmCancel = false;
						void cancelCurrentRun();
					}}>Cancel run</button
				>
			{/snippet}
		</Dialog>
	{/if}
</div>

<style>
	.budget-help {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		margin-top: 8px;
	}
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
		border-radius: var(--app-radius-lg);
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
		border-radius: var(--app-radius);
		background: var(--app-bg);
		color: var(--app-muted);
	}
	.mode-btn:hover:not(.active) {
		color: var(--app-text);
		background: var(--app-subtle);
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
		font-size: var(--app-text-md);
	}
	.query-label {
		font-weight: 600;
		display: block;
		margin-bottom: 8px;
	}
	.query-input {
		min-height: 120px;
		padding: 14px 16px;
	}
	.connection-summary {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 6px 12px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
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
		font-size: var(--app-text-sm);
	}
	.stop-toggle span.invalid {
		color: var(--app-danger);
	}
	.stop-conditions {
		margin: 8px 0 12px;
	}
	.stop-fields {
		display: flex;
		flex-wrap: wrap;
		gap: 12px;
	}
	.stop-fields label {
		flex: 1 1 130px;
		font-size: var(--app-text-sm);
		color: var(--app-muted);
	}
	.stop-fields input {
		margin-top: 4px;
	}
	.stop-fields input[aria-invalid='true'] {
		border-color: var(--app-danger);
	}
	.field-error {
		display: block;
		margin-top: 4px;
		color: var(--app-danger);
	}
	.examples {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		margin-top: 10px;
		font-size: var(--app-text-sm);
		color: var(--app-muted);
	}
	.chip {
		max-width: 100%;
		padding: 3px 10px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius-pill);
		color: var(--app-text);
		text-align: left;
		overflow-wrap: anywhere;
	}
	.chip:hover {
		border-color: var(--app-accent);
		color: var(--app-accent);
	}
	.config-problems {
		display: flex;
		gap: 10px;
		margin: 0 0 12px;
		padding: 10px 12px;
		border: 1px solid color-mix(in srgb, var(--app-warning) 55%, var(--app-border));
		border-radius: var(--app-radius);
		background: color-mix(in srgb, var(--app-warning) 7%, var(--app-panel));
		color: var(--app-text);
		font-size: var(--app-text-md);
	}
	.config-problems :global(svg) {
		flex-shrink: 0;
		margin-top: 2px;
		color: var(--app-warning);
	}
	.config-title {
		font-weight: 600;
	}
	.config-problems a {
		color: var(--app-accent);
		text-decoration: underline;
		margin-left: 4px;
	}
	.recent {
		max-width: 900px;
		margin-top: 16px;
		padding: 0 4px;
	}
	.recent h2 {
		display: flex;
		align-items: center;
		gap: 6px;
		margin: 0 0 6px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		font-weight: 600;
		-webkit-line-clamp: unset;
		line-clamp: unset;
	}
	.recent button {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 8px;
		border-radius: var(--app-radius-sm);
		color: var(--app-text);
		text-align: left;
		font-size: var(--app-text-md);
	}
	.recent button span {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.recent button :global(svg) {
		color: var(--app-muted);
	}
	.recent button:hover {
		background: var(--app-subtle);
	}
	.query-actions {
		display: flex;
		justify-content: flex-end;
		align-items: center;
		gap: 12px;
		margin-top: 16px;
	}
	.shortcut-hint {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	kbd {
		padding: 1px 5px;
		border: 1px solid var(--app-border);
		border-bottom-width: 2px;
		border-radius: var(--app-radius-sm);
		font-family: inherit;
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
		font-size: var(--app-text-xs);
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}
	h2 {
		font-size: var(--app-text-2xl);
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
		font-size: var(--app-text-sm);
		color: var(--app-accent);
		background: transparent;
		padding: 4px 0;
		border: 0;
	}
	.run-summary {
		display: flex;
		flex-direction: column;
		gap: 6px;
		flex-shrink: 0;
		padding: 0 0 12px;
	}
	.summary-line {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px 20px;
		min-width: 0;
	}
	.summary-cost {
		margin-left: auto;
	}
	.run-errors {
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
