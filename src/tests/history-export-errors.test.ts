import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import HistoryPage from '../routes/history/+page.svelte';
import ExportDialog from '$lib/components/run/ExportDialog.svelte';
import { llmIssue } from './fixtures/llm-issue';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ save: vi.fn() }));

const runs = [
	{ id: 'run-1', query: 'First query', status: 'completed', run_type: 'table', stats: null, error: null, created_at: 1 },
	{ id: 'run-2', query: 'Second query', status: 'completed', run_type: 'table', stats: null, error: null, created_at: 2 },
];
const apiInvoke = vi.mocked(invoke);
async function respond(command: string, args?: Record<string, unknown>) {
	if (command === 'list_runs') return runs;
	if (command === 'get_run_schema') return { columns: [], confirmed: true };
	if (command === 'get_run_rows') return [];
	if (command === 'get_run_issues') return args?.runId === 'run-1' ? [llmIssue] : [];
	return undefined;
}

beforeEach(() => {
	apiInvoke.mockReset().mockImplementation(respond as typeof invoke);
	vi.mocked(save).mockReset().mockResolvedValue('/tmp/export.csv');
});
afterEach(cleanup);

describe('history and export errors', () => {
	it('loads persisted model issues for a completed run and clears them when another run is selected', async () => {
		render(HistoryPage);
		await fireEvent.click((await screen.findAllByRole('button', { name: 'View' }))[0]);
		expect(await screen.findByRole('region', { name: 'Model request issues' })).toHaveTextContent('Requested output cap4,096 tokens per request');
		expect(apiInvoke).toHaveBeenCalledWith('get_run_issues', { runId: 'run-1' });
		await fireEvent.click(screen.getByRole('button', { name: 'Back to History' }));
		await fireEvent.click(screen.getAllByRole('button', { name: 'View' })[1]);
		await screen.findByRole('heading', { name: 'Second query' });
		await waitFor(() => expect(screen.queryByText('Loading model request issues…')).not.toBeInTheDocument());
		expect(screen.queryByRole('region', { name: 'Model request issues' })).not.toBeInTheDocument();
	});

	it('ignores a delayed issue response after the user changes the selected run', async () => {
		let finishIssues!: (value: unknown) => void;
		apiInvoke.mockImplementation((command, args) => {
			if (command === 'get_run_issues' && (args as { runId: string }).runId === 'run-1') {
				return new Promise((resolve) => { finishIssues = resolve; });
			}
			return respond(command, args as Record<string, unknown>) as never;
		});
		render(HistoryPage);
		await fireEvent.click((await screen.findAllByRole('button', { name: 'View' }))[0]);
		await fireEvent.click(screen.getByRole('button', { name: 'Back to History' }));
		await fireEvent.click(screen.getAllByRole('button', { name: 'View' })[1]);
		finishIssues([llmIssue]);
		await screen.findByRole('heading', { name: 'Second query' });
		await waitFor(() => expect(screen.queryByText('Loading model request issues…')).not.toBeInTheDocument());
		expect(screen.queryByRole('region', { name: 'Model request issues' })).not.toBeInTheDocument();
	});

	it('keeps a result-load failure visible inside the selected history entry', async () => {
		apiInvoke.mockImplementation(async (command, args) => {
			if (command === 'get_run_rows') throw new Error('sqlite: database is locked');
			return respond(command, args as Record<string, unknown>) as never;
		});
		render(HistoryPage);
		await fireEvent.click((await screen.findAllByRole('button', { name: 'View' }))[0]);
		expect(await screen.findByRole('alert')).toHaveTextContent('Local data could not be accessed');
		expect(screen.getByRole('heading', { name: 'First query' })).toBeInTheDocument();
	});

	it('catches save-dialog failures and offers an actionable export retry', async () => {
		vi.mocked(save).mockRejectedValueOnce(new Error('Permission denied'));
		render(ExportDialog, { runId: 'run-1', onclose: vi.fn() });
		await fireEvent.click(screen.getByRole('button', { name: 'Export' }));
		expect(await screen.findByRole('alert')).toHaveTextContent('Choose a writable export folder and retry');
		expect(screen.getByRole('button', { name: 'Export' })).toBeEnabled();
		expect(apiInvoke).not.toHaveBeenCalled();
	});
});
