import { get } from 'svelte/store';
import { goto } from '$app/navigation';
import {
	CopyIcon,
	DownloadIcon,
	ExternalLinkIcon,
	PencilIcon,
	PinIcon,
	PinOffIcon,
	RotateCcwIcon,
	SquarePenIcon,
	TrashIcon,
} from '@lucide/svelte';
import type { MenuItem } from '$lib/components/common/ContextMenu.svelte';
import { getRun, getRunSchema } from '$lib/api/tauri';
import { queryDraft, runInProgress, startNewRun } from '$lib/stores/run';
import { settings } from '$lib/stores/settings';
import { toast } from '$lib/stores/toasts';
import { errorText } from '$lib/utils/errors';
import { copyWithToast } from '$lib/utils/linkMenu';
import { runLimits } from '$lib/utils/history';
import { parseStopConditions, stopInputFromSettings } from '$lib/utils/stopConditions';

const BUSY_MESSAGE = 'A run is in progress. Cancel it or wait until it finishes, then try again.';

/** Limits the run was started with, or the remembered ones for runs saved before limits were. */
function limitsFor(config: string | undefined, mode: string) {
	return (
		runLimits(config) ?? parseStopConditions(stopInputFromSettings(get(settings), mode), mode).conditions
	);
}

/** Starts the same query again with its mode and limits; tables offer the earlier schema. */
export async function runAgain(runId: string) {
	try {
		if (runInProgress()) throw new Error(BUSY_MESSAGE);
		const run = await getRun(runId);
		if (!run) throw new Error('This run no longer exists.');
		const schema = run.run_type === 'table' ? ((await getRunSchema(runId))?.columns ?? null) : null;
		const limits = limitsFor(run.config, run.run_type) ?? undefined;
		const started = startNewRun(run.query, run.run_type, limits, schema);
		await goto('/');
		await started;
	} catch (error) {
		toast(errorText(error), 'error');
	}
}

/** Opens the query form filled in with the run's query, mode and limits. */
export async function editAndRun(runId: string) {
	try {
		if (runInProgress()) throw new Error(BUSY_MESSAGE);
		const run = await getRun(runId);
		if (!run) throw new Error('This run no longer exists.');
		queryDraft.set({ query: run.query, runType: run.run_type, limits: runLimits(run.config) });
		await goto('/');
	} catch (error) {
		toast(errorText(error), 'error');
	}
}

export interface RunMenuHandlers {
	open?: () => void;
	exportRun?: () => void;
	rename: () => void;
	togglePin: () => void;
	remove: () => void;
}

/** Actions on a saved run, shared by History rows and the run page. */
export function runMenuItems(
	run: { id: string; query: string; pinned_at?: number | null },
	handlers: RunMenuHandlers
): MenuItem[] {
	const items: MenuItem[] = [];
	if (handlers.open) items.push({ label: 'Open', icon: ExternalLinkIcon, action: handlers.open });
	items.push(
		{ label: 'Run again', icon: RotateCcwIcon, separator: !!handlers.open, action: () => runAgain(run.id) },
		{ label: 'Edit and run', icon: SquarePenIcon, action: () => editAndRun(run.id) },
		{ label: 'Copy query', icon: CopyIcon, action: () => copyWithToast(run.query, 'Query copied.') }
	);
	if (handlers.exportRun) items.push({ label: 'Export…', icon: DownloadIcon, action: handlers.exportRun });
	items.push(
		{ label: 'Rename', icon: PencilIcon, separator: true, action: handlers.rename },
		{
			label: run.pinned_at ? 'Unpin' : 'Pin',
			icon: run.pinned_at ? PinOffIcon : PinIcon,
			action: handlers.togglePin,
		},
		{ label: 'Delete', icon: TrashIcon, separator: true, action: handlers.remove }
	);
	return items;
}
