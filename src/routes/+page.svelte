<script lang="ts">
	import { tooltip } from '$lib/actions/tooltip';
	import { t } from '$lib/i18n';
	import NumberInput from '$lib/components/common/NumberInput.svelte';
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
		queryDraft,
		runInProgress,
	} from '$lib/stores/run';
	import type { RunInfo, SchemaColumn } from '$lib/types';
	import { listRuns } from '$lib/api/tauri';
	import SchemaEditor from '$lib/components/run/SchemaEditor.svelte';
	import AttachmentBar from '$lib/components/query/AttachmentBar.svelte';
	import FileChips from '$lib/components/common/FileChips.svelte';
	import { attachFiles, draftAttachments, MAX_ATTACHMENTS, takeDraftAttachments } from '$lib/stores/attachments';
	import Checkbox from '$lib/components/common/Checkbox.svelte';
	import { filesOnlyModes } from '$lib/utils/attachments';
	import ResultsTable from '$lib/components/run/ResultsTable.svelte';
	import RowDetailPanel from '$lib/components/run/RowDetailPanel.svelte';
	import CostSummary from '$lib/components/run/CostSummary.svelte';
	import ProgressBar from '$lib/components/run/ProgressBar.svelte';
	import RunProgress from '$lib/components/run/RunProgress.svelte';
	import RunControls from '$lib/components/run/RunControls.svelte';
	import RunHeader from '$lib/components/run/RunHeader.svelte';
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
	let readingFiles = $derived($draftAttachments.some((d) => d.state === 'reading'));
	let canSubmit = $derived(!!query.trim() && !configProblems.length && !readingFiles);
	let dragging = $state(false);
	/** With files attached, a run can skip the web and answer from the files only. */
	let searchWeb = $state(true);
	let hasFiles = $derived($draftAttachments.some((d) => d.state === 'ready'));

	/** Files pasted into the query (screenshots, copied files) are attached instead of typed. */
	async function handlePaste(event: ClipboardEvent) {
		const files = [...(event.clipboardData?.files ?? [])];
		if (!files.length) return;
		event.preventDefault();
		if (await attachFiles(files)) toast(t('attachments.limit', { count: MAX_ATTACHMENTS }), 'error');
	}

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
			toast(t('query.rememberFailed'), 'error');
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
			const filesOnly = !searchWeb && filesOnlyModes.includes(runType);
			await startNewRun(query, runType, conditions, null, takeDraftAttachments(), filesOnly ? 'files' : 'web');
			searchWeb = true;
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

	const modeIcons = { table: TableIcon, images: ImageIcon, links: LinkIcon, research: BrainIcon } as const;
	const modeUnits = { table: 'units.row', images: 'units.image', links: 'units.link', research: 'units.step' } as const;
	// Rebuilt when the language changes.
	let modes = $derived(
		(['table', 'images', 'links', 'research'] as const).map((value) => ({
			value,
			label: t(`mode.${value}`),
			icon: modeIcons[value],
			description: t(`mode.${value}.description`),
			action: t(`mode.${value}.action`),
			unit: modeUnits[value],
			targetLabel: t(`mode.${value}.target`),
			examples: [t(`mode.${value}.example1`), t(`mode.${value}.example2`), t(`mode.${value}.example3`)],
		}))
	);
	let mode = $derived(modes.find((item) => item.value === runType) ?? modes[0]);
	let stopSummary = $derived(
		stopParsed.conditions
			? `${t(mode.unit, { count: stopParsed.conditions.target_row_count })} · ${formatUsd(stopParsed.conditions.max_budget_usd)} · ${formatMinutes(stopParsed.conditions.max_duration_seconds)}`
			: t('query.checkValues')
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
		applyDraft();
	});
	/** Edit and run from History: fill the form without starting. */
	function applyDraft() {
		const draft = $queryDraft;
		if (!draft) return;
		queryDraft.set(null);
		if (runInProgress()) return;
		if (!isIdle) resetRun();
		query = draft.query;
		if (modes.some((item) => item.value === draft.runType)) {
			// Set the mode first so its remembered limit does not replace the run's own.
			stopMode = draft.runType as Mode;
			runType = draft.runType as Mode;
		}
		if (draft.limits) {
			stopInput = {
				target: String(draft.limits.target_row_count),
				budget: draft.limits.max_budget_usd.toFixed(2),
				duration: String(Math.max(1, Math.round(draft.limits.max_duration_seconds / 60))),
			};
			stopEdited = true;
		}
	}
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
		) || t('query.noModel')
	);
</script>

<div class="query-page">
	{#if isIdle}
		<header class="page-header">
			<div>
				<h1>{t('query.title')}</h1>
				<p>{t('query.subtitle')}</p>
			</div>
		</header>
		<div class="query-scroll">
			<form class="query-form" class:dragging onsubmit={handleSubmit} novalidate>
				{#if dragging}<div class="drop-overlay" aria-hidden="true">{t('attachments.drop')}</div>{/if}
				<div class="mode-toggle" role="group" aria-label={t('query.resultFormat')}>
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
				<label class="query-label" for="research-query">{t('query.label')}</label>
				<textarea
					id="research-query"
					class="input query-input"
					bind:this={queryInput}
					bind:value={query}
					placeholder={t('query.placeholder')}
					rows={5}
					onpaste={handlePaste}
					onkeydown={(event) => {
						if (event.key === 'Enter' && hasMod(event)) {
							event.preventDefault();
							event.currentTarget.form?.requestSubmit();
						}
					}}></textarea>
				<AttachmentBar bind:dragging />
				{#if hasFiles && filesOnlyModes.includes(runType)}
					<div class="source-mode"><Checkbox bind:checked={searchWeb}>{t('attachments.searchWeb')}</Checkbox></div>
				{/if}
				{#if !query.trim()}
					<div class="examples" aria-label={t('query.examples')} role="group">
						<span>{t('query.try')}</span>
						{#each mode.examples as example}<button
								type="button"
								class="chip"
								onclick={() => useQuery(example, runType)}>{example}</button
							>{/each}
					</div>
				{/if}
				<div class="connection-summary">
					<span>{providerNames[provider] ?? provider}</span><span class="model-name" use:tooltip={{ text: model, whenTruncated: true }}
						>{model}</span
					><a href="/settings#settings-llm">{t('query.configure')}</a>
				</div>
				{#if configProblems.length}
					<div class="config-problems" role="alert">
						<TriangleAlertIcon size={16} />
						<div>
							<p class="config-title">{t('query.finishSetup')}</p>
							<ul>
								{#each configProblems as problem}<li>
										{problem.message}
										<a href={`/settings#settings-${problem.section}`}>{t('query.openSettings')}</a>
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
						/>{/if}{t('query.stopConditions')}
					<span class:invalid={stopInvalid}>{stopSummary}</span>
				</button>
				{#if showStopConditions}
					<div class="stop-conditions" id="stop-conditions">
						<div class="stop-fields">
							<label for="targetRows"
								>{mode.targetLabel}<NumberInput
									id="targetRows"
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
								>{t('query.maxCost')}<NumberInput
									id="maxBudget"
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
								>{t('query.maxDuration')}<NumberInput
									id="maxDuration"
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
							{t('query.budgetHelp')}
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
					<h2 id="recent-title"><HistoryIcon size={15} />{t('query.recent')}</h2>
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
		<RunHeader
			eyebrow={modes.find((item) => item.value === $runState.runType)?.label ?? t('query.results')}
			title={$runState.query}
		>
			{#snippet actions()}
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
			{/snippet}
		</RunHeader>
		{#if !isResearchRun}<FileChips files={$runState.attachments.map((f) => f.attachment)} />{/if}
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
					>{t('query.schemaPaused')}</EmptyState
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
						attachments={$runState.attachments}
						sourceMode={$runState.sourceMode}
						onask={$runState.runId
							? (question, limits, files) => askFollowUp(question, limits, files.attachments, files.sourceMode)
							: undefined}
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
			title={t('query.cancelTitle')}
			onclose={() => {
				confirmCancel = false;
			}}
		>
			<p>
				{t('query.cancelText')}
			</p>
			{#snippet footer()}
				<button
					class="button"
					onclick={() => {
						confirmCancel = false;
					}}>{t('query.keepRunning')}</button
				>
				<button
					class="button danger outline"
					onclick={() => {
						confirmCancel = false;
						void cancelCurrentRun();
					}}>{t('query.cancelRun')}</button
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
		position: relative;
		max-width: 900px;
		background: var(--app-panel);
		border: 1px solid var(--app-border);
		padding: 24px;
		border-radius: var(--app-radius-lg);
	}
	.source-mode {
		margin-top: 8px;
		font-size: var(--app-text-md);
	}
	.query-form.dragging {
		border-color: var(--app-accent);
	}
	.drop-overlay {
		position: absolute;
		inset: 0;
		z-index: 5;
		display: grid;
		place-items: center;
		border: 2px dashed var(--app-accent);
		border-radius: inherit;
		background: color-mix(in srgb, var(--app-accent) 10%, var(--app-panel));
		color: var(--app-accent);
		font-size: var(--app-text-lg);
		font-weight: 600;
		pointer-events: none;
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
	.stop-fields :global(.number-input) {
		margin-top: 4px;
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
