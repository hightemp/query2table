import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import QueryPage from '../routes/+page.svelte';
import { resetRun, runState, startNewRun } from '$lib/stores/run';
import { llmIssue } from './fixtures/llm-issue';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

const callbacks = new Map<string, (event: { payload: unknown }) => void>();
function status(value: string) {
	callbacks.get('run:status_changed')?.({ payload: { run_id: 'run-1', status: value } });
}

describe('query page run feedback', () => {
	beforeEach(() => {
		resetRun();
		callbacks.clear();
		vi.mocked(invoke)
			.mockReset()
			.mockImplementation(
				async (command) => (command === 'start_run' ? { run_id: 'run-1' } : undefined) as never
			);
		vi.mocked(listen)
			.mockReset()
			.mockImplementation(async (event, callback) => {
				callbacks.set(event as string, callback as (event: { payload: unknown }) => void);
				return () => {};
			});
	});

	afterEach(() => {
		cleanup();
		resetRun();
	});

	it('renders a Cancel command failure while retaining the backend status', async () => {
		await startNewRun('Find robot channels');
		status('running');
		render(QueryPage);
		vi.mocked(invoke).mockRejectedValueOnce('Run is not active');
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
		expect(await screen.findByRole('alert')).toHaveTextContent(
			'This run can no longer receive commands'
		);
		expect(screen.getByText('Could not cancel: Run is not active')).not.toBeVisible();
		expect(get(runState).status).toBe('running');
		expect(screen.getByRole('button', { name: 'Cancel' })).toBeEnabled();
	});

	it('shows a nonfatal model limit and preserves it in a completed run until reset', async () => {
		await startNewRun('Find robot channels');
		status('running');
		render(QueryPage);
		callbacks.get('run:llm_issue')?.({ payload: llmIssue });
		expect(await screen.findByRole('region', { name: 'Model request issues' })).toHaveTextContent(
			'The model reached its output limit'
		);
		expect(get(runState).status).toBe('running');
		status('completed');
		await waitFor(() =>
			expect(screen.getByRole('region', { name: 'Model request issues' })).toHaveTextContent(
				'The run completed'
			)
		);
		await fireEvent.click(screen.getByRole('button', { name: 'New query' }));
		expect(screen.queryByRole('region', { name: 'Model request issues' })).not.toBeInTheDocument();
	});

	it('shows query expansion as the active phase when its log arrives', async () => {
		await startNewRun('Find robot channels');
		status('running');
		render(QueryPage);
		callbacks.get('run:log_entry')?.({
			payload: {
				run_id: 'run-1',
				level: 'INFO',
				role: 'query_expander',
				message: 'Expanding search queries',
			},
		});
		await waitFor(() => expect(screen.getByText('Expanding searches')).toBeVisible());
	});

	it('shows a failed startup after the query form has been replaced', async () => {
		render(QueryPage);
		vi.mocked(invoke).mockRejectedValueOnce('Provider connection failed');
		await fireEvent.input(screen.getByLabelText('What would you like to find?'), {
			target: { value: 'Find robot channels' },
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Start Research' }));
		expect(await screen.findByRole('alert')).toHaveTextContent('Provider connection failed');
		expect(screen.getByRole('button', { name: 'New query' })).toBeInTheDocument();
	});

	it('preserves schema edits across backend pause and resume acknowledgments', async () => {
		await startNewRun('Find robot channels');
		callbacks.get('run:schema_proposed')?.({
			payload: {
				run_id: 'run-1',
				columns: [{ name: 'Channel', type: 'text', description: '', required: false }],
			},
		});
		render(QueryPage);
		const columnName = screen.getByPlaceholderText('Column name');
		await fireEvent.input(columnName, { target: { value: 'Robot channel' } });
		await fireEvent.click(screen.getByRole('button', { name: 'Pause' }));
		status('paused');
		await waitFor(() => expect(screen.getByRole('button', { name: 'Resume' })).toBeInTheDocument());
		expect(columnName).not.toBeVisible();
		await fireEvent.click(screen.getByRole('button', { name: 'Resume' }));
		status('schema_review');
		await waitFor(() => expect(columnName).toBeVisible());
		expect(screen.getByPlaceholderText('Column name')).toBe(columnName);
		expect(columnName).toHaveValue('Robot channel');
	});
});
