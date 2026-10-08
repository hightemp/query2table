<script lang="ts">
	import { formatDateTime, t, type MessageKey } from '$lib/i18n';
	import { onDestroy, tick } from 'svelte';
	import { get } from 'svelte/store';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { ArrowLeftIcon, DownloadIcon, EllipsisIcon } from '@lucide/svelte';
	import {
		getRun,
		getRunSchema,
		getRunRows,
		getImageResults,
		getLinkResults,
		getResearchResult,
		getRunIssues,
		dismissRunNotices,
		pinRun,
		renameRun,
		getRunAttachments,
	} from '$lib/api/tauri';
	import type {
		RunInfo,
		SchemaColumn,
		ImageResult,
		LinkResult,
		ResearchStep,
		LlmIssueEvent,
		RunAttachment,
	} from '$lib/types';
	import type { FollowUpFiles } from '$lib/components/run/FollowUpBox.svelte';
	import type { RunRow, ResearchTurnState } from '$lib/stores/run';
	import type { StopConditions } from '$lib/api/tauri';
	import {
		askFollowUp,
		openConversation,
		parseDismissed,
		researchTurnsFrom,
		runInProgress,
		runState,
	} from '$lib/stores/run';
	import { deleteWithUndo, history } from '$lib/stores/history';
	import { showContextMenu } from '$lib/stores/contextMenu';
	import { toast } from '$lib/stores/toasts';
	import CostSummary from '$lib/components/run/CostSummary.svelte';
	import RunHeader from '$lib/components/run/RunHeader.svelte';
	import ResultsTable from '$lib/components/run/ResultsTable.svelte';
	import RowDetailPanel from '$lib/components/run/RowDetailPanel.svelte';
	import ExportDialog from '$lib/components/run/ExportDialog.svelte';
	import ImageGallery from '$lib/components/run/ImageGallery.svelte';
	import LinkList from '$lib/components/run/LinkList.svelte';
	import ResearchView from '$lib/components/run/ResearchView.svelte';
	import RunNotices from '$lib/components/run/RunNotices.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import EmptyState from '$lib/components/common/EmptyState.svelte';
	import Badge from '$lib/components/common/Badge.svelte';
	import { storedCosts } from '$lib/utils/costs';
	import { errorText } from '$lib/utils/errors';
	import { statusLabel, statusTone } from '$lib/utils/status';
	import { RowSelection } from '$lib/utils/rowSelection.svelte';
	import { runMenuItems } from '$lib/utils/runActions';


	let run = $state<RunInfo | null>(null);
	let missing = $state(false);
	let costs = $derived(storedCosts(run?.stats));
	let schema = $state<SchemaColumn[]>([]);
	let rows = $state<RunRow[]>([]);
	let images = $state<ImageResult[]>([]);
	let links = $state<LinkResult[]>([]);
	let researchSteps = $state<ResearchStep[]>([]);
	let researchAnswer = $state<string | null>(null);
	let researchTurns = $state<ResearchTurnState[]>([]);
	let researchLimits = $state<StopConditions | null>(null);
	let researchFiles = $state<RunAttachment[]>([]);
	let loading = $state(true);
	let error = $state('');
	let issues = $state<LlmIssueEvent[]>([]);
	let issueError = $state('');
	let issuesLoading = $state(true);
	let dismissed = $state<Record<string, number> | null>(null);
	let showExport = $state(false);
	let renaming = $state(false);
	let renameInput = $state<HTMLInputElement>();
	let renameDraft = $state('');
	const selection = new RowSelection();
	let selectedRow = $derived(rows.find((row) => row.id === selection.id) ?? null);
	let request = 0;
	onDestroy(() => request++);

	let hasResults = $derived(rows.length > 0 || images.length > 0 || links.length > 0 || !!researchAnswer);
	let resultLabel = $derived.by(() => {
		if (!run) return '';
		if (run.run_type === 'images') return t('units.image', { count: images.length });
		if (run.run_type === 'links') return t('units.link', { count: links.length });
		if (run.run_type === 'research')
			return researchTurns.length > 1
				? t('history.question', { count: researchTurns.length })
				: t('units.step', { count: researchSteps.length });
		return t('units.row', { count: rows.length });
	});

	$effect(() => {
		void load(page.params.id ?? '');
	});

	async function load(id: string) {
		const current = ++request;
		loading = true;
		missing = false;
		error = '';
		run = null;
		schema = [];
		rows = [];
		images = [];
		links = [];
		researchSteps = [];
		researchAnswer = null;
		researchTurns = [];
		researchLimits = null;
		researchFiles = [];
		issues = [];
		issueError = '';
		issuesLoading = true;
		selection.clear();
		showExport = false;
		renaming = false;
		// The run being worked on lives on the query page.
		if (get(runState).runId === id && runInProgress()) {
			await goto('/', { replaceState: true });
			return;
		}
		try {
			const found = await getRun(id);
			if (current !== request) return;
			if (!found) {
				missing = true;
				return;
			}
			run = found;
			dismissed = parseDismissed(found.dismissed_notices);
			void getRunIssues(id)
				.then((list) => {
					if (current === request) issues = list.filter((issue) => issue.run_id === id).slice(-100);
				})
				.catch((e) => {
					if (current === request) issueError = errorText(e);
				})
				.finally(() => {
					if (current === request) issuesLoading = false;
				});
			if (found.run_type === 'images') {
				const result = await getImageResults(id);
				if (current === request) images = result;
			} else if (found.run_type === 'links') {
				const result = await getLinkResults(id);
				if (current === request) links = result;
			} else if (found.run_type === 'research') {
				const result = await getResearchResult(id);
				if (current !== request) return;
				researchSteps = result.steps;
				researchAnswer = result.answer_markdown;
				researchTurns = researchTurnsFrom(result, found.query, found.status);
				researchLimits = result.turns?.[0]?.limits ?? null;
				researchFiles = await getRunAttachments(found.id).catch(() => []);
			} else {
				const [schemaInfo, saved] = await Promise.all([getRunSchema(id), getRunRows(id)]);
				if (current !== request) return;
				schema = schemaInfo?.columns ?? [];
				rows = saved.map((r) => ({
					id: r.id,
					data: r.data as Record<string, unknown>,
					confidence: r.confidence,
					sources: r.source_count,
				}));
			}
		} catch (e) {
			if (current === request) error = errorText(e);
		} finally {
			if (current === request) loading = false;
		}
	}

	/** Back to the list where it was left, or to the list when opened directly. */
	function back() {
		if (get(history).loaded && window.history.length > 1) window.history.back();
		else void goto('/history');
	}

	/** Continues a saved conversation on the query page, as a live run. */
	async function continueConversation(question: string, limits: Required<StopConditions>, files: FollowUpFiles) {
		if (!run) return;
		await openConversation(run.id);
		await askFollowUp(question, limits, files.attachments, files.sourceMode);
		await goto('/');
	}

	async function dismissNotices(value: Record<string, number>) {
		if (!run) return;
		dismissed = value;
		const id = run.id;
		history.update((s) => ({
			...s,
			runs: s.runs.map((item) => (item.id === id ? { ...item, dismissed_notices: JSON.stringify(value) } : item)),
		}));
		try {
			await dismissRunNotices(id, value);
		} catch (e) {
			toast(errorText(e), 'error');
		}
	}

	function updateListed(patch: Record<string, unknown>) {
		if (!run) return;
		const id = run.id;
		history.update((s) => ({ ...s, runs: s.runs.map((item) => (item.id === id ? { ...item, ...patch } : item)) }));
	}

	async function startRename() {
		if (!run) return;
		renameDraft = run.title || run.query;
		renaming = true;
		await tick();
		renameInput?.select();
	}
	async function finishRename(value: string | null) {
		if (!renaming || !run) return;
		renaming = false;
		if (value === null) return;
		const title = value.trim() || null;
		try {
			await renameRun(run.id, title);
			run = { ...run, title };
			updateListed({ title });
		} catch (e) {
			toast(errorText(e), 'error');
		}
	}

	function openMenu(event: MouseEvent) {
		if (!run) return;
		const current = run;
		const rect = (event.currentTarget as Element).getBoundingClientRect();
		showContextMenu({ x: rect.right - 200, y: rect.bottom + 4 }, t('runActions.menu'), runMenuItems(current, {
			rename: () => void startRename(),
			togglePin: async () => {
				try {
					const pinned = !current.pinned_at;
					await pinRun(current.id, pinned);
					run = { ...current, pinned_at: pinned ? Math.floor(Date.now() / 1000) : null };
					// Pinning reorders the list; it is loaded again on return.
					history.update((s) => ({ ...s, loaded: false }));
				} catch (e) {
					toast(errorText(e), 'error');
				}
			},
			remove: async () => {
				try {
					await deleteWithUndo([current.id]);
					back();
				} catch (e) {
					toast(errorText(e), 'error');
				}
			},
		}));
	}
</script>

<div class="run-page">
	<div class="back">
		<button class="button sm" onclick={back}><ArrowLeftIcon size={16} />{t('runPage.back')}</button>
	</div>
	{#if missing}
		<EmptyState>{t('history.notFound')}</EmptyState>
	{:else if !run}
		{#if error}<ErrorNotice {error} context="history" />{:else}<EmptyState role="status">{t('runPage.loading')}</EmptyState>{/if}
	{:else}
		{@const current = run}
		<RunHeader
			eyebrow={['table', 'images', 'links', 'research'].includes(current.run_type) ? t(`mode.${current.run_type}` as MessageKey) : current.run_type}
			title={current.title || current.query}
			subtitle={current.title ? current.query : null}
			level={1}
		>
			{#snippet heading()}
				{#if renaming}<input
						bind:this={renameInput}
						class="input rename"
						aria-label={t('history.runName')}
						bind:value={renameDraft}
						onkeydown={(event) => {
							if (event.key === 'Enter') void finishRename(renameDraft);
							else if (event.key === 'Escape') void finishRename(null);
						}}
						onblur={() => void finishRename(renameDraft)}
					/>{:else}<h1 class="title">{current.title || current.query}</h1>{/if}
			{/snippet}
			{#snippet meta()}
				{#if current.status !== 'completed'}<Badge tone={statusTone(current.status)}
						>{statusLabel(current.status)}</Badge
					>{/if}
				<span>{formatDateTime(current.created_at)}</span>
				{#if !loading}<span>{resultLabel}</span>{/if}
				<CostSummary accounting={costs.accounting} legacy={costs.legacy} inline />
			{/snippet}
			{#snippet actions()}
				{#if hasResults}<button class="button sm accent" onclick={() => (showExport = true)}
						><DownloadIcon size={14} />{t('controls.export')}</button
					>{/if}
				<button class="icon-button" aria-label={t('runPage.moreActions')} aria-haspopup="menu" onclick={openMenu}
					><EllipsisIcon size={16} /></button
				>
			{/snippet}
		</RunHeader>

		{#if error || current.error || issueError}
			<div class="notices">
				{#if error}<ErrorNotice {error} context="history" />{/if}
				{#if current.error}<ErrorNotice error={current.error} />{/if}
				{#if issueError}<ErrorNotice error={issueError} context="history" />{/if}
			</div>
		{/if}
		{#if !issuesLoading}
			<RunNotices
				{issues}
				accounting={costs.accounting}
				runStatus={current.status}
				turnCount={researchTurns.length}
				{dismissed}
				ondismiss={dismissNotices}
			/>
		{/if}

		{#if showExport}
			<ExportDialog
				runId={current.id}
				runType={current.run_type}
				turns={researchTurns}
				onclose={() => (showExport = false)}
			/>
		{/if}

		<div class="result">
			{#if loading}
				<EmptyState role="status">{t('runPage.loadingResults')}</EmptyState>
			{:else if current.run_type === 'images'}
				{#if images.length > 0}<ImageGallery {images} />{:else}<EmptyState
						>{t('runPage.noImages')}</EmptyState
					>{/if}
			{:else if current.run_type === 'links'}
				{#if links.length > 0}<LinkList {links} />{:else}<EmptyState>{t('runPage.noLinks')}</EmptyState
					>{/if}
			{:else if current.run_type === 'research'}
				{#if researchAnswer || researchSteps.length > 0}
					<ResearchView turns={researchTurns} limits={researchLimits} attachments={researchFiles} onask={continueConversation} />
				{:else}<EmptyState>{t('runPage.noResearch')}</EmptyState>{/if}
			{:else if schema.length > 0 && rows.length > 0}
				<ResultsTable {schema} {rows} onrowclick={(row, order) => selection.select(row.id, order)} />
			{:else}
				<EmptyState>{t('runPage.noResults')}</EmptyState>
			{/if}
		</div>
		{#if selectedRow}
			<RowDetailPanel
				row={selectedRow}
				columns={schema.map((c) => c.name)}
				position={selection.position}
				onnavigate={selection.move}
				context="history"
				onclose={selection.clear}
			/>
		{/if}
	{/if}
</div>

<style>
	.run-page {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
	}
	.back {
		flex-shrink: 0;
		margin-bottom: 12px;
	}
	.title {
		font-size: var(--app-text-2xl);
		font-weight: 650;
		line-height: 1.35;
		margin: 4px 0 0;
		overflow-wrap: anywhere;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
	.rename {
		width: 100%;
		max-width: 640px;
		margin-top: 4px;
		font-size: var(--app-text-lg);
	}
	.notices {
		flex-shrink: 0;
		max-height: 28vh;
		overflow: auto;
		scrollbar-gutter: stable;
		padding-right: 12px;
	}
	.result {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
	}
</style>
