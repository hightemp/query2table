import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import RunControls from '$lib/components/run/RunControls.svelte';

afterEach(cleanup);

describe('run control buttons', () => {
	it.each(['pending', 'running', 'schema_review'])('supports Pause and Cancel during %s', async (status) => {
		const onpause = vi.fn();
		const oncancel = vi.fn();
		render(RunControls, { status, onpause, oncancel, onresume: vi.fn(), onreset: vi.fn() });
		await fireEvent.click(screen.getByRole('button', { name: 'Pause' }));
		await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
		expect(onpause).toHaveBeenCalledOnce();
		expect(oncancel).toHaveBeenCalledOnce();
	});

	it('shows pending feedback while preserving Cancel until cancellation itself is requested', async () => {
		const page = render(RunControls, {
			status: 'running', pending: 'pause',
			onpause: vi.fn(), onresume: vi.fn(), oncancel: vi.fn(), onreset: vi.fn(),
		});
		expect(screen.getByRole('button', { name: 'Pause' })).toBeDisabled();
		expect(screen.getByText('Pausing…')).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Cancel' })).toBeEnabled();
		await page.rerender({ pending: 'cancel' });
		expect(screen.getByRole('button', { name: 'Cancel' })).toBeDisabled();
		expect(screen.getByText('Cancelling…')).toBeInTheDocument();
		expect(screen.getByText('running')).toBeInTheDocument();
	});
});
