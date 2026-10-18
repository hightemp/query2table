import { beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';

const api = vi.hoisted(() => ({
	checkUpdate: vi.fn(),
	installUpdate: vi.fn(),
	restartApp: vi.fn(),
	onUpdateProgress: vi.fn(),
}));
vi.mock('$lib/api/tauri', () => api);

import {
	CHECK_INTERVAL_MS,
	checkAtStartup,
	checkForUpdates,
	installFoundUpdate,
	noticeVersion,
	skipVersion,
	updates,
} from '$lib/stores/updates';

const info = { version: '1.1.0', current_version: '1.0.0', notes: '- New', date: null, installer: 'appimage' };

beforeEach(() => {
	localStorage.clear();
	updates.set({ status: 'idle', info: null, progress: null, error: null, open: false });
	vi.clearAllMocks();
	api.onUpdateProgress.mockResolvedValue(() => {});
});

describe('checking for updates', () => {
	it('checks at startup at most once a day, and not when turned off', async () => {
		api.checkUpdate.mockResolvedValue(null);
		const now = Date.now();
		expect(await checkAtStartup(undefined, now)).toBe(true);
		expect(get(updates).status).toBe('none');
		expect(await checkAtStartup('true', now + 60_000)).toBe(false);
		expect(await checkAtStartup('true', now + CHECK_INTERVAL_MS + 1)).toBe(true);
		expect(await checkAtStartup('false', now + 3 * CHECK_INTERVAL_MS)).toBe(false);
		expect(api.checkUpdate).toHaveBeenCalledTimes(2);
	});

	it('opens the dialog only for a manual check', async () => {
		api.checkUpdate.mockResolvedValue(info);
		await checkForUpdates(false);
		expect(get(updates)).toMatchObject({ status: 'available', info, open: false });
		await checkForUpdates(true);
		expect(get(updates).open).toBe(true);
	});

	it('keeps the error kind from the backend', async () => {
		api.checkUpdate.mockRejectedValue({ kind: 'check', message: 'offline' });
		await checkForUpdates(true);
		expect(get(updates)).toMatchObject({ status: 'error', error: { kind: 'check', message: 'offline' } });
		api.checkUpdate.mockRejectedValue('plain failure');
		await checkForUpdates(true);
		expect(get(updates).error).toEqual({ kind: 'check', message: 'plain failure' });
	});
});

describe('the sidebar notice', () => {
	it('shows a found version until it is skipped', async () => {
		api.checkUpdate.mockResolvedValue(info);
		await checkForUpdates(false);
		expect(noticeVersion(get(updates))).toBe('1.1.0');
		skipVersion('1.1.0');
		expect(noticeVersion(get(updates))).toBeNull();
		// A newer one is shown again.
		expect(noticeVersion({ ...get(updates), info: { ...info, version: '1.2.0' } })).toBe('1.2.0');
		expect(noticeVersion({ ...get(updates), status: 'none' }, null)).toBeNull();
	});
});

describe('installing', () => {
	it('reports progress and ends waiting for a restart', async () => {
		api.checkUpdate.mockResolvedValue(info);
		await checkForUpdates(true);
		let report: (p: { downloaded: number; total: number | null }) => void = () => {};
		api.onUpdateProgress.mockImplementation(async (callback) => {
			report = callback;
			return () => {};
		});
		let finish: (value: string) => void = () => {};
		api.installUpdate.mockReturnValue(new Promise((resolve) => (finish = resolve)));
		const done = installFoundUpdate();
		await vi.waitFor(() => expect(api.installUpdate).toHaveBeenCalled());
		report({ downloaded: 5, total: 10 });
		expect(get(updates)).toMatchObject({ status: 'downloading', progress: { downloaded: 5, total: 10 } });
		finish('restart');
		await done;
		expect(get(updates)).toMatchObject({ status: 'restart', progress: null });
	});

	it('shows a failed signature as such', async () => {
		api.installUpdate.mockRejectedValue({ kind: 'signature', message: 'The signature does not match' });
		await installFoundUpdate();
		expect(get(updates)).toMatchObject({ status: 'error', error: { kind: 'signature' } });
	});
});
