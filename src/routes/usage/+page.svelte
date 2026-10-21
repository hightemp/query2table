<script lang="ts">
	import { onMount } from 'svelte';
	import { save } from '@tauri-apps/plugin-dialog';
	import { ChevronLeftIcon, ChevronRightIcon, DownloadIcon, Trash2Icon, ChartColumnIcon, FolderOpenIcon } from '@lucide/svelte';
	import { formatNumber, t } from '$lib/i18n';
	import { clearUsage, getMonthlyBudget, getStorageUsage, getUsage, getUsageFirstAt, openAppFolder, repriceUsage, saveFileData } from '$lib/api/tauri';
	import type { MonthlyBudget, StorageUsage, UsageReport } from '$lib/types';
	import { canGoForward, formatBytes, shiftAnchor, USAGE_METRICS, usageChart, usageCsv, usagePeriod, type UsageMetric, type UsagePeriodKind } from '$lib/utils/usage';
	import { providerName } from '$lib/settings/llmProviders';
	import { toast } from '$lib/stores/toasts';
	import { showContextMenu } from '$lib/stores/contextMenu';
	import { menuPointFor } from '$lib/components/common/ContextMenu.svelte';
	import { errorText } from '$lib/utils/errors';
	import { readablePath } from '$lib/utils/platform';
	import Select from '$lib/components/common/Select.svelte';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import EmptyState from '$lib/components/common/EmptyState.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import UsageChart from '$lib/components/usage/UsageChart.svelte';

	const KINDS: UsagePeriodKind[] = ['month', 'year', 'all'];

	let kind = $state<UsagePeriodKind>('month');
	let anchor = $state(new Date());
	let firstAt = $state<number | null>(null);
	let metric = $state<UsageMetric>('cost');
	let report = $state<UsageReport | null>(null);
	let storage = $state<StorageUsage | null>(null);
	let budget = $state<MonthlyBudget | null>(null);
	let error = $state('');
	let busy = $state(false);
	let confirmClear = $state(false);

	// The first call matters only to "All time"; reading it only there keeps other periods from loading twice.
	const period = $derived(usagePeriod(kind, anchor, kind === 'all' ? firstAt : null));
	const step = $derived(kind === 'month' ? 'day' : period.keys[0]?.length === 4 ? 'year' : 'month');
	const chart = $derived(report ? usageChart(report, period.labels, metric, step) : null);
	const empty = $derived(!!report && report.totals.llm_calls + report.totals.searches + report.totals.downloads === 0);
	const searchesBy = $derived(new Map((report?.by_search ?? []).map((s) => [s.provider, s.calls])));

	let requested = 0;
	async function load() {
		const current = ++requested;
		const request = period.request;
		try {
			const result = await getUsage(request);
			if (current === requested) {
				report = result;
				error = '';
			}
		} catch (e) {
			if (current === requested) error = errorText(e);
		}
	}

	onMount(() => {
		void (async () => {
			[firstAt, storage, budget] = await Promise.all([
				getUsageFirstAt().catch(() => null),
				getStorageUsage().catch(() => null),
				getMonthlyBudget().catch(() => null),
			]);
		})();
	});
	$effect(() => {
		void period;
		void load();
	});

	function usd(amount: number): string {
		return amount > 0 && amount < 0.01 ? `$${formatNumber(amount, { maximumFractionDigits: 4 })}` : `$${formatNumber(amount, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
	}
	function change(current: number, previous: number): string | null {
		if (kind === 'all' || previous <= 0) return null;
		const percent = Math.round(((current - previous) / previous) * 100);
		return t('usage.change', { change: `${percent > 0 ? '+' : ''}${percent}%` });
	}
	const searchName = (provider: string) => (provider === 'brave' ? 'Brave' : provider === 'serper' ? 'Serper' : provider);
	const modeName = (runType: string) => t(`mode.${runType}` as 'mode.table');

	async function reprice() {
		busy = true;
		try {
			const count = await repriceUsage();
			toast(count ? t('usage.repriced', { count }) : t('usage.repriceNone'), count ? 'success' : 'info');
			await load();
		} catch (e) {
			toast(errorText(e), 'error');
		} finally {
			busy = false;
		}
	}

	function exportMenu(button: HTMLElement) {
		showContextMenu(menuPointFor(button), t('usage.export'), [
			{ label: t('usage.exportDays'), action: () => exportCsv('days') },
			{ label: t('usage.exportModels'), action: () => exportCsv('models') },
		]);
	}
	async function exportCsv(view: 'days' | 'models') {
		if (!report) return;
		const name = kind === 'all' ? 'all' : kind === 'year' ? period.keys[0].slice(0, 4) : period.keys[0].slice(0, 7);
		try {
			const path = await save({
				defaultPath: `statistics-${name}${view === 'models' ? '-models' : ''}.csv`,
				filters: [{ name: 'CSV', extensions: ['csv'] }],
			});
			if (!path) return;
			const bytes = new TextEncoder().encode(usageCsv(report, period.keys, view));
			let binary = '';
			for (const byte of bytes) binary += String.fromCharCode(byte);
			const written = await saveFileData(path, btoa(binary));
			toast(t('usage.saved', { path: readablePath(written) ?? t('files.chosenFile') }), 'success');
		} catch (e) {
			toast(t('usage.saveFailed', { error: errorText(e) }), 'error');
		}
	}

	async function clear() {
		busy = true;
		try {
			await clearUsage();
			confirmClear = false;
			toast(t('usage.cleared'), 'success');
			firstAt = null;
			await load();
		} catch (e) {
			toast(errorText(e), 'error');
		} finally {
			busy = false;
		}
	}
</script>

<div class="usage-page">
	<header class="page-header">
		<div>
			<h1>{t('usage.title')}</h1>
			<p>{t('usage.subtitle')}</p>
		</div>
		<div class="actions">
			<button type="button" class="button" onclick={(event) => exportMenu(event.currentTarget)} disabled={!report}
				><DownloadIcon size={15} />{t('usage.export')}</button
			>
			<button type="button" class="button" onclick={() => (confirmClear = true)}><Trash2Icon size={15} />{t('usage.clear')}</button>
		</div>
	</header>

	<div class="period">
		<div class="kinds" role="group" aria-label={t('usage.period')}>
			{#each KINDS as option (option)}
				<button type="button" class="chip" aria-pressed={kind === option} onclick={() => ((kind = option), (anchor = new Date()))}
					>{t(`usage.period.${option}`)}</button
				>
			{/each}
		</div>
		<div class="step">
			{#if kind !== 'all'}
				<button type="button" class="button icon" aria-label={t('usage.previous')} onclick={() => (anchor = shiftAnchor(kind, anchor, -1))}
					><ChevronLeftIcon size={16} /></button
				>
			{/if}
			<h2>{period.title}</h2>
			{#if kind !== 'all'}
				<button
					type="button"
					class="button icon"
					aria-label={t('usage.next')}
					disabled={!canGoForward(kind, anchor, new Date())}
					onclick={() => (anchor = shiftAnchor(kind, anchor, 1))}><ChevronRightIcon size={16} /></button
				>
			{/if}
		</div>
	</div>

	<div class="content">
		{#if error}<ErrorNotice {error} context="history" />{/if}

		<section class="budget" aria-labelledby="usage-budget">
			<h2 id="usage-budget">{t('usage.budget')}</h2>
			{#if budget && budget.limit_usd > 0}
				{@const share = Math.min(100, Math.round((budget.spent_usd / budget.limit_usd) * 100))}
				<p>{t('usage.budgetOf', { spent: usd(budget.spent_usd), limit: usd(budget.limit_usd) })}</p>
				<div
					class="meter"
					class:warn={share >= 80}
					class:over={budget.spent_usd >= budget.limit_usd}
					role="progressbar"
					aria-label={t('usage.budget')}
					aria-valuemin={0}
					aria-valuemax={100}
					aria-valuenow={share}
				>
					<span style:width={`${share}%`}></span>
				</div>
			{:else}
				<p class="muted">{t('usage.budgetOff')} <a href="/settings#monthly_budget_usd">{t('usage.budgetSet')}</a></p>
			{/if}
		</section>

		{#if report && empty}
			<EmptyState icon={ChartColumnIcon} title={t('usage.empty')} />
		{:else if report}
			{@const totals = report.totals}
			{@const previous = report.previous}
			<div class="tiles">
				<div class="tile" role="group" aria-label={t('usage.metric.cost')}>
					<span class="label">{t('usage.metric.cost')}</span>
					<strong>{usd(totals.cost_usd)}</strong>
					{#if change(totals.cost_usd, previous.cost_usd)}<span class="change">{change(totals.cost_usd, previous.cost_usd)}</span>{/if}
					{#if totals.unpriced_calls}
						<span class="note"
							>{t('usage.unpriced', { count: totals.unpriced_calls })} · <a href="/settings#settings-llm">{t('usage.setRates')}</a></span
						>
						<button type="button" class="button sm" onclick={reprice} disabled={busy}>{t('usage.reprice')}</button>
					{/if}
				</div>
				<div class="tile" role="group" aria-label={t('usage.metric.tokens')}>
					<span class="label">{t('usage.metric.tokens')}</span>
					<strong>{formatNumber(totals.prompt_tokens + totals.completion_tokens)}</strong>
					<span class="note"
						>{t('usage.tokensSplit', {
							input: formatNumber(totals.prompt_tokens),
							output: formatNumber(totals.completion_tokens),
							cached: formatNumber(totals.cached_tokens),
						})}</span
					>
				</div>
				<div class="tile" role="group" aria-label={t('usage.modelCalls')}>
					<span class="label">{t('usage.modelCalls')}</span>
					<strong>{formatNumber(totals.llm_calls)}</strong>
					<span class="note">{t('usage.runs', { count: totals.runs })}</span>
				</div>
				<div class="tile" role="group" aria-label={t('usage.metric.searches')}>
					<span class="label">{t('usage.metric.searches')}</span>
					<strong>{formatNumber(totals.searches)}</strong>
					<span class="note">Brave {formatNumber(searchesBy.get('brave') ?? 0)} · Serper {formatNumber(searchesBy.get('serper') ?? 0)}</span>
				</div>
				<div class="tile" role="group" aria-label={t('usage.data.web')}>
					<span class="label">{t('usage.data.web')}</span>
					<strong>{formatBytes(totals.web_bytes)}</strong>
					<span class="note">{t('usage.downloads', { count: totals.downloads })}</span>
				</div>
				<div class="tile" role="group" aria-label={t('usage.data.models')}>
					<span class="label">{t('usage.data.models')}</span>
					<strong>{formatBytes(totals.model_bytes_sent + totals.model_bytes_received)}</strong>
					<span class="note"
						>{t('usage.sentReceived', { sent: formatBytes(totals.model_bytes_sent), received: formatBytes(totals.model_bytes_received) })}</span
					>
				</div>
			</div>

			<section class="block" aria-labelledby="usage-chart">
				<div class="block-head">
					<h2 id="usage-chart">{t('usage.chart')}</h2>
					<Select
						size="sm"
						label={t('usage.chart')}
						value={metric}
						options={USAGE_METRICS.map((value) => ({ value, label: t(`usage.metric.${value}`) }))}
						onchange={(value) => (metric = value)}
					/>
				</div>
				{#if chart}<UsageChart {chart} />{/if}
			</section>

			<div class="tables">
				<section class="block">
					<table aria-label={t('usage.byModel')}>
						<caption>{t('usage.byModel')}</caption>
						<thead
							><tr
								><th scope="col">{t('usage.col.model')}</th><th scope="col">{t('usage.col.calls')}</th><th scope="col"
									>{t('usage.col.tokens')}</th
								><th scope="col">{t('usage.col.cost')}</th></tr
							></thead
						>
						<tbody>
							{#each report.by_model as line (`${line.provider}/${line.model}`)}
								<tr>
									<th scope="row"><span class="model">{line.model || '—'}</span><span class="muted">{providerName(line.provider)}</span></th>
									<td>{formatNumber(line.calls)}</td>
									<td>{formatNumber(line.prompt_tokens + line.completion_tokens)}</td>
									<td
										>{usd(line.cost_usd)}{#if line.unpriced_calls}<span class="unpriced muted">{t('usage.unpriced', { count: line.unpriced_calls })}</span
											>{/if}</td
									>
								</tr>
							{/each}
						</tbody>
					</table>
				</section>
				<section class="block">
					<table aria-label={t('usage.byMode')}>
						<caption>{t('usage.byMode')}</caption>
						<thead
							><tr
								><th scope="col">{t('usage.col.mode')}</th><th scope="col">{t('usage.col.runs')}</th><th scope="col">{t('usage.col.calls')}</th><th
									scope="col">{t('usage.col.cost')}</th
								></tr
							></thead
						>
						<tbody>
							{#each report.by_mode as line (line.run_type)}
								<tr
									><th scope="row">{modeName(line.run_type)}</th><td>{formatNumber(line.runs)}</td><td>{formatNumber(line.llm_calls)}</td><td
										>{usd(line.cost_usd)}</td
									></tr
								>
							{/each}
							{#each report.by_search as line (line.provider)}
								<tr class="search-row"
									><th scope="row">{searchName(line.provider)}</th><td></td><td>{t('usage.searchCount', { count: line.calls })}</td><td
										>{usd(line.cost_usd)}</td
									></tr
								>
							{/each}
						</tbody>
					</table>
				</section>
				<section class="block wide">
					<table aria-label={t('usage.topRuns')}>
						<caption>{t('usage.topRuns')}</caption>
						<thead
							><tr><th scope="col">{t('usage.col.run')}</th><th scope="col">{t('usage.col.mode')}</th><th scope="col">{t('usage.col.cost')}</th></tr></thead
						>
						<tbody>
							{#each report.top_runs as run (run.run_id)}
								<tr>
									<th scope="row" class="run"
										>{#if run.exists}<a href={`/history/${run.run_id}`}>{run.title}</a>{:else}{run.title}
											<span class="muted">({t('usage.deletedRun')})</span>{/if}</th
									>
									<td>{modeName(run.run_type)}</td>
									<td>{usd(run.cost_usd)}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</section>
			</div>
		{/if}

		<section class="block disk" aria-labelledby="usage-disk">
			<div class="block-head">
				<h2 id="usage-disk">{t('usage.disk')}</h2>
				<button type="button" class="button sm" onclick={() => void openAppFolder('data')}><FolderOpenIcon size={14} />{t('usage.openFolder')}</button>
			</div>
			{#if storage}
				<dl>
					<dt>{t('usage.disk.database')}</dt>
					<dd>{formatBytes(storage.database_bytes)}</dd>
					<dt>{t('usage.disk.attachments')}</dt>
					<dd>{formatBytes(storage.attachments_bytes)}</dd>
					<dt>{t('usage.disk.images')}</dt>
					<dd>{formatBytes(storage.answer_images_bytes)}</dd>
				</dl>
				<p class="muted">{t('usage.disk.hint')} <a href="/history">{t('nav.history')}</a></p>
			{/if}
		</section>
	</div>
</div>

{#if confirmClear}
	<Dialog title={t('usage.clearConfirm')} onclose={() => (confirmClear = false)} {busy}>
		<p>{t('usage.clearBody')}</p>
		{#snippet footer()}
			<button type="button" class="button" onclick={() => (confirmClear = false)} disabled={busy}>{t('controls.cancel')}</button>
			<button type="button" class="button danger" onclick={clear} disabled={busy}>{t('usage.clearAction')}</button>
		{/snippet}
	</Dialog>
{/if}

<style>
	.usage-page {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
	}
	.actions {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}
	.period {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 8px 16px;
		padding-bottom: 12px;
		flex-shrink: 0;
	}
	.kinds {
		display: flex;
		gap: 6px;
	}
	.chip {
		padding: 4px 12px;
		border: 1px solid var(--app-border);
		border-radius: 999px;
		background: transparent;
		color: var(--app-text);
		font-size: var(--app-text-sm);
		cursor: pointer;
	}
	.chip[aria-pressed='true'] {
		background: var(--app-accent-solid);
		border-color: var(--app-accent-solid);
		color: var(--app-on-accent);
	}
	.step {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.step h2 {
		margin: 0;
		min-width: 9ch;
		text-align: center;
		font-size: var(--app-text-lg);
		font-weight: 600;
	}
	.button.icon {
		padding: 4px;
	}
	.content {
		display: flex;
		flex-direction: column;
		gap: 16px;
		overflow-y: auto;
		min-height: 0;
		padding-bottom: 24px;
	}
	h2 {
		font-size: var(--app-text-md);
		font-weight: 600;
		margin: 0;
	}
	.budget {
		display: grid;
		gap: 6px;
	}
	.budget p {
		margin: 0;
	}
	.meter {
		height: 8px;
		border-radius: 999px;
		background: var(--app-subtle);
		overflow: hidden;
	}
	.meter span {
		display: block;
		height: 100%;
		background: var(--app-accent-solid);
	}
	.meter.warn span {
		background: var(--app-warning);
	}
	.meter.over span {
		background: var(--app-danger);
	}
	.tiles {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 240px), 1fr));
		gap: 12px;
	}
	.tile {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 4px;
		padding: 12px 14px;
		background: var(--app-panel);
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		min-width: 0;
	}
	.tile .label {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.tile strong {
		font-size: var(--app-text-2xl);
		font-weight: 650;
		font-variant-numeric: tabular-nums;
	}
	.tile .change,
	.tile .note {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		overflow-wrap: anywhere;
	}
	.block {
		display: grid;
		gap: 8px;
		min-width: 0;
	}
	.block-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}
	.block-head :global(.select) {
		width: auto;
	}
	.tables {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 360px), 1fr));
		gap: 16px;
	}
	.wide {
		grid-column: 1 / -1;
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: var(--app-text-sm);
		font-variant-numeric: tabular-nums;
	}
	caption {
		text-align: left;
		font-weight: 600;
		font-size: var(--app-text-md);
		padding-bottom: 6px;
	}
	th,
	td {
		padding: 6px 8px;
		border-bottom: 1px solid var(--app-border);
		text-align: right;
	}
	th[scope='row'],
	thead th:first-child {
		text-align: left;
		font-weight: 500;
		overflow-wrap: anywhere;
	}
	thead th {
		color: var(--app-muted);
		font-weight: 500;
	}
	.model,
	.unpriced {
		display: block;
	}
	@media (max-width: 640px) {
		.tiles {
			grid-template-columns: repeat(2, minmax(0, 1fr));
			gap: 8px;
		}
		.tile {
			padding: 10px;
		}
		.tile strong {
			font-size: var(--app-text-xl);
		}
	}
	.search-row th {
		color: var(--app-muted);
	}
	.run a {
		color: var(--app-accent);
	}
	.disk dl {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 4px 16px;
		margin: 0;
	}
	.disk dd {
		margin: 0;
		font-variant-numeric: tabular-nums;
	}
	.disk p {
		margin: 0;
		font-size: var(--app-text-sm);
	}
	a {
		color: var(--app-accent);
	}
</style>
