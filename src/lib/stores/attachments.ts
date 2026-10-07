import { get, writable } from 'svelte/store';
import { addAttachmentData, addAttachments, getAttachments, removeAttachment } from '$lib/api/tauri';
import type { AttachmentInfo, AttachResult } from '$lib/types';
import { attachErrorText } from '$lib/utils/attachments';
import { readStorage, writeStorage } from '$lib/utils/storage';

/** Most files one query can carry. */
export const MAX_ATTACHMENTS = 20;
const DRAFT_KEY = 'q2t-draft-attachments';

/** A file in the query form: being read, attached, or refused. */
export interface DraftAttachment {
	key: string;
	name: string;
	state: 'reading' | 'ready' | 'error';
	info?: AttachmentInfo;
	error?: string;
}

/** Files attached to the query being written; ready ones are remembered across restarts. */
export const draftAttachments = writable<DraftAttachment[]>([]);

let nextKey = 0;
const newKey = () => `draft-${++nextKey}`;

function persist() {
	const ids = get(draftAttachments).flatMap((d) => (d.state === 'ready' && d.info ? [d.info.id] : []));
	writeStorage(DRAFT_KEY, JSON.stringify(ids));
}

function settle(key: string, result: AttachResult) {
	draftAttachments.update((list) => {
		// The same file attached earlier: drop the new chip.
		if (result.attachment && list.some((d) => d.key !== key && d.info?.id === result.attachment!.id))
			return list.filter((d) => d.key !== key);
		return list.map((d) =>
			d.key !== key
				? d
				: result.attachment
					? { ...d, state: 'ready', info: result.attachment, name: result.attachment.file_name, error: undefined }
					: { ...d, state: 'error', error: attachErrorText(result.error ?? { code: 'storage', message: '' }) }
		);
	});
}

function fail(key: string, error: unknown) {
	settle(key, { attachment: null, error: { file_name: '', code: 'storage', message: String(error) } });
}

/** Adds chips for files being read; returns their keys and how many did not fit. */
function reserve(names: string[]): { keys: string[]; refused: number } {
	const free = Math.max(0, MAX_ATTACHMENTS - get(draftAttachments).length);
	const taken = names.slice(0, free);
	const drafts = taken.map((name) => ({ key: newKey(), name, state: 'reading' as const }));
	draftAttachments.update((list) => [...list, ...drafts]);
	return { keys: drafts.map((d) => d.key), refused: names.length - taken.length };
}

const baseName = (path: string) => path.split(/[\\/]/).pop() || path;

/** Attaches files by path (file dialog, drag and drop). Returns how many did not fit. */
export async function attachPaths(paths: string[]): Promise<number> {
	const { keys, refused } = reserve(paths.map(baseName));
	if (!keys.length) return refused;
	try {
		const results = await addAttachments(paths.slice(0, keys.length));
		keys.forEach((key, i) => settle(key, results[i] ?? { attachment: null, error: null }));
	} catch (error) {
		keys.forEach((key) => fail(key, error));
	}
	persist();
	return refused;
}

/** Attaches pasted files. Returns how many did not fit. */
export async function attachFiles(files: File[]): Promise<number> {
	const { keys, refused } = reserve(files.map((f) => f.name || 'pasted'));
	await Promise.all(
		keys.map(async (key, i) => {
			try {
				settle(key, await addAttachmentData(files[i]));
			} catch (error) {
				fail(key, error);
			}
		})
	);
	persist();
	return refused;
}

export async function removeDraftAttachment(key: string) {
	const draft = get(draftAttachments).find((d) => d.key === key);
	draftAttachments.update((list) => list.filter((d) => d.key !== key));
	persist();
	if (draft?.info) await removeAttachment(draft.info.id).catch(() => {});
}

/** Brings back the files of a query that was being written before a restart. */
export async function restoreDraftAttachments() {
	let ids: string[] = [];
	try {
		ids = JSON.parse(readStorage(DRAFT_KEY) ?? '[]');
	} catch {
		return;
	}
	if (!Array.isArray(ids) || !ids.length || get(draftAttachments).length) return;
	try {
		const infos = await getAttachments(ids);
		draftAttachments.set(infos.map((info) => ({ key: newKey(), name: info.file_name, state: 'ready', info })));
	} catch {
		// Without the files the query still works; the chips simply do not come back.
	}
}

/** Ids of the attached files for a new run; the form starts empty again. */
export function takeDraftAttachments(): string[] {
	const ids = get(draftAttachments).flatMap((d) => (d.state === 'ready' && d.info ? [d.info.id] : []));
	draftAttachments.set([]);
	persist();
	return ids;
}
