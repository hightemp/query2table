import { get, writable } from 'svelte/store';
import { checkUpdate, installUpdate as apiInstallUpdate, onUpdateProgress, restartApp } from '$lib/api/tauri';
import type { UpdateError, UpdateInfo, UpdateProgress } from '$lib/types';
import { errorText } from '$lib/utils/errors';

/**
 * idle: not checked yet; checking; none: up to date; available; downloading (with `progress`);
 * restart: installed, waiting for a restart; installer: the Android installer is open;
 * permission: Android must allow installing apps first; error.
 */
export type UpdateStatus = 'idle' | 'checking' | 'none' | 'available' | 'downloading' | 'restart' | 'installer' | 'permission' | 'error';

export interface UpdateState {
	status: UpdateStatus;
	info: UpdateInfo | null;
	progress: UpdateProgress | null;
	error: UpdateError | null;
	/** The dialog is open. */
	open: boolean;
}

const initial: UpdateState = { status: 'idle', info: null, progress: null, error: null, open: false };
export const updates = writable<UpdateState>(initial);

/** Startup checks happen at most this often. */
export const CHECK_INTERVAL_MS = 24 * 60 * 60 * 1000;
const CHECKED_AT = 'q2t.updateCheckedAt';
const SKIPPED = 'q2t.updateSkipped';

function stored(key: string): string | null {
	try {
		return localStorage.getItem(key);
	} catch {
		return null;
	}
}

function store(key: string, value: string) {
	try {
		localStorage.setItem(key, value);
	} catch {
		// Private windows and blocked storage: the check just runs again next time.
	}
}

export function skippedVersion(): string | null {
	return stored(SKIPPED);
}

/** The found version will not be offered again on its own. */
export function skipVersion(version: string) {
	store(SKIPPED, version);
	updates.update((state) => ({ ...state, open: false }));
}

function asUpdateError(error: unknown, kind: UpdateError['kind']): UpdateError {
	if (error && typeof error === 'object' && 'kind' in error && 'message' in error) return error as UpdateError;
	return { kind, message: errorText(error) };
}

/** Looks for a newer release. A manual check opens the dialog when one is found. */
export async function checkForUpdates(manual = false): Promise<void> {
	if (['checking', 'downloading'].includes(get(updates).status)) return;
	updates.update((state) => ({ ...state, status: 'checking', error: null }));
	try {
		const info = await checkUpdate();
		store(CHECKED_AT, String(Date.now()));
		updates.update((state) => ({
			...state,
			status: info ? 'available' : 'none',
			info,
			open: manual && !!info,
		}));
	} catch (error) {
		updates.update((state) => ({ ...state, status: 'error', error: asUpdateError(error, 'check') }));
	}
}

/** The quiet check at startup: unless turned off, and at most once a day. */
export async function checkAtStartup(setting: string | undefined, now = Date.now()): Promise<boolean> {
	if (setting === 'false') return false;
	const last = Number(stored(CHECKED_AT));
	if (last && now - last < CHECK_INTERVAL_MS && now >= last) return false;
	await checkForUpdates(false);
	return true;
}

/** Whether the sidebar should point to an update: found and not skipped, or under way. */
export function noticeVersion(state: UpdateState, skipped: string | null = skippedVersion()): string | null {
	if (!state.info) return null;
	if (state.status === 'available' && state.info.version === skipped) return null;
	return ['available', 'downloading', 'restart', 'installer', 'permission'].includes(state.status) ? state.info.version : null;
}

export function openUpdateDialog() {
	updates.update((state) => ({ ...state, open: true }));
}

export function closeUpdateDialog() {
	updates.update((state) => ({ ...state, open: false }));
}

/** Downloads, verifies and installs the found update, reporting progress. */
export async function installFoundUpdate(): Promise<void> {
	updates.update((state) => ({ ...state, status: 'downloading', progress: { downloaded: 0, total: null }, error: null }));
	const stop = await onUpdateProgress((progress) => updates.update((state) => ({ ...state, progress })));
	try {
		const installed = await apiInstallUpdate();
		updates.update((state) => ({ ...state, status: installed, progress: null }));
	} catch (error) {
		updates.update((state) => ({ ...state, status: 'error', progress: null, error: asUpdateError(error, 'install') }));
	} finally {
		stop();
	}
}

export async function restartNow(): Promise<void> {
	try {
		await restartApp();
	} catch (error) {
		updates.update((state) => ({ ...state, status: 'error', error: asUpdateError(error, 'install') }));
	}
}
