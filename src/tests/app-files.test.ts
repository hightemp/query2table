import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import AppFiles from '$lib/components/settings/AppFiles.svelte';
import { getAppPaths, copyAppPath, openAppFolder } from '$lib/api/tauri';

vi.mock('$lib/api/tauri', () => ({ getAppPaths: vi.fn(), copyAppPath: vi.fn(), openAppFolder: vi.fn() }));
const paths = { data_dir: '/test/App Data', database_file: '/test/App Data/data.db', log_dir: '/test/Logs' };

describe('application file locations', () => {
	beforeEach(() => {
		vi.mocked(getAppPaths).mockReset().mockResolvedValue(paths);
		vi.mocked(copyAppPath).mockReset().mockResolvedValue();
		vi.mocked(openAppFolder).mockReset().mockResolvedValue();
	});
	afterEach(cleanup);

	it('shows backend paths and copies the selected location through the native command', async () => {
		render(AppFiles);
		expect(await screen.findByLabelText('Application data')).toHaveValue(paths.data_dir);
		expect(screen.getByLabelText('Settings database')).toHaveValue(paths.database_file);
		expect(screen.getByLabelText('Application logs')).toHaveValue(paths.log_dir);
		await fireEvent.click(screen.getByRole('button', { name: 'Copy Settings database path' }));
		expect(copyAppPath).toHaveBeenCalledExactlyOnceWith('database');
		expect(await screen.findByRole('status')).toHaveTextContent('Settings database path copied.');
	});

	it('opens data and log folders through their distinct native targets', async () => {
		render(AppFiles);
		await fireEvent.click(await screen.findByRole('button', { name: 'Open Application data folder' }));
		await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('Application data folder opened.'));
		await fireEvent.click(screen.getByRole('button', { name: 'Open Application logs folder' }));
		expect(openAppFolder).toHaveBeenNthCalledWith(1, 'data');
		expect(openAppFolder).toHaveBeenNthCalledWith(2, 'logs');
	});

	it('shows clipboard and folder errors while keeping paths available', async () => {
		vi.mocked(copyAppPath).mockRejectedValueOnce('Could not copy the application path: clipboard unavailable');
		vi.mocked(openAppFolder).mockRejectedValueOnce('Could not open the application folder');
		render(AppFiles);
		await fireEvent.click(await screen.findByRole('button', { name: 'Copy Application data path' }));
		expect(await screen.findByRole('alert')).toHaveTextContent('Select the path field and copy it manually');
		expect(screen.getByLabelText('Application data')).toHaveValue(paths.data_dir);
		await fireEvent.click(screen.getByRole('button', { name: 'Open Application data folder' }));
		await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('Could not open the folder'));
	});

	it('can retry loading the actual locations after a failure', async () => {
		vi.mocked(getAppPaths).mockRejectedValueOnce('Unable to resolve application directory');
		render(AppFiles);
		await fireEvent.click(await screen.findByRole('button', { name: 'Retry loading paths' }));
		expect(await screen.findByLabelText('Application data')).toHaveValue(paths.data_dir);
		expect(screen.queryByRole('alert')).not.toBeInTheDocument();
	});
});
