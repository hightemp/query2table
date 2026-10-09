import { t, type MessageKey } from '$lib/i18n';
import type { ProgressStats, SearchQueryInfo } from '$lib/types';

export type StageState = 'done' | 'active' | 'waiting' | 'skipped';

export interface StageView {
	id: string;
	label: string;
	state: StageState;
	/** Counts for the stage, e.g. "12 of 45". */
	detail: string | null;
}

const STAGES: Record<string, string[]> = {
	table: ['interpret', 'schema', 'plan', 'search', 'read', 'dedup'],
	links: ['plan', 'search', 'read'],
	images: ['plan', 'search', 'rank'],
};

const FINISHED = ['completed', 'cancelled', 'failed'];

function searchDetail(queries: SearchQueryInfo[], active: boolean): string | null {
	if (!queries.length) return null;
	if (!active) return t('stages.queries', { count: queries.length });
	const done = queries.filter((q) => ['completed', 'failed', 'skipped'].includes(q.status)).length;
	return t('stages.of', { done, total: queries.length });
}

/** The stages of a run in order, with where it is now and counts where they help. */
export type StageCounts = Record<string, { done: number; total: number }>;

export function stageViews(
	runType: string,
	current: string | null,
	status: string,
	progress: ProgressStats | null,
	queries: SearchQueryInfo[],
	counts: StageCounts = {},
	/** Short names for the folded line. */
	short = false
): StageView[] {
	const ids = [...(STAGES[runType] ?? STAGES.table)];
	// Comparing with a reference picture happens only in some image runs.
	if (runType === 'images' && (current === 'compare' || counts.compare)) ids.push('compare');
	const finished = FINISHED.includes(status);
	const at = current ? ids.indexOf(current) : -1;
	// A run without queries past the search stage did not search (files only).
	const searched = queries.length > 0;
	return ids.map((id, index) => {
		let state: StageState = finished || index < at ? 'done' : index === at ? 'active' : 'waiting';
		if ((id === 'plan' || id === 'search') && !searched && (finished || at > index) && runType === 'table') state = 'skipped';
		let detail: string | null = null;
		const count = counts[id];
		if (id === 'search') detail = searchDetail(queries, state === 'active');
		else if (count && count.total > 0 && state !== 'waiting')
			detail = t(id === 'read' ? 'stages.pages' : 'stages.of', { done: count.done, total: count.total });
		else if (id === 'read' && progress && progress.pages_total > 0 && state !== 'waiting')
			detail = t('stages.pages', { done: progress.pages_fetched, total: progress.pages_total });
		return { id, label: t(`stages.${short ? 'short.' : ''}${id}` as MessageKey), state, detail };
	});
}

/** One line about a run's searches: "45 queries · 120 results · 3 failed". */
export function queriesSummary(queries: SearchQueryInfo[]): string {
	if (!queries.length) return '';
	const parts = [
		t('stages.queries', { count: queries.length }),
		t('stages.results', { count: queries.reduce((sum, q) => sum + q.result_count, 0) }),
	];
	const failed = queries.filter((q) => q.status === 'failed').length;
	const skipped = queries.filter((q) => q.status === 'skipped').length;
	if (failed) parts.push(t('stages.failed', { count: failed }));
	if (skipped) parts.push(t('stages.skipped', { count: skipped }));
	return parts.join(' · ');
}
