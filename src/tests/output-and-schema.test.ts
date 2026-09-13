import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import SchemaEditor from '$lib/components/run/SchemaEditor.svelte';
import ProgressBar from '$lib/components/run/ProgressBar.svelte';
import { formatValue } from '$lib/utils/values';

afterEach(cleanup);
const column = { name: 'Name', type: 'text', description: '', required: true };
describe('schema draft and structured values', () => {
	it('does not restore a deliberately removed final column', async () => {
		render(SchemaEditor, { columns: [column], onconfirm: vi.fn(), oncancel: vi.fn() });
		await fireEvent.click(screen.getByRole('button', { name: 'Remove column 1' }));
		expect(screen.queryByPlaceholderText('Column name')).not.toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Confirm Schema' })).toBeDisabled();
		await fireEvent.click(screen.getByRole('button', { name: 'Add Column' }));
		expect(screen.getByPlaceholderText('Column name')).toHaveValue('');
	});
	it('rejects blank or duplicate names and disables repeat confirmation while pending', async () => {
		const confirm = vi.fn();
		const page = render(SchemaEditor, {
			columns: [column, { ...column, name: ' name ' }],
			onconfirm: confirm,
			oncancel: vi.fn(),
		});
		expect(screen.getByText('Column names must be unique.')).toBeInTheDocument();
		await fireEvent.input(screen.getByLabelText('Column 2 name'), { target: { value: 'URL' } });
		await fireEvent.click(screen.getByRole('button', { name: 'Confirm Schema' }));
		expect(confirm).toHaveBeenCalledOnce();
		await page.rerender({ pending: true });
		expect(screen.getByRole('button', { name: 'Confirming…' })).toBeDisabled();
	});
	it('keeps missing, false, zero, arrays and objects distinct', () => {
		expect(
			[null, undefined, '', false, 0, ['A', 'B'], { enabled: false }].map((value) =>
				formatValue(value)
			)
		).toEqual(['—', '—', '—', 'false', '0', 'A · B', '{"enabled":false}']);
		expect(formatValue({ enabled: false }, true)).toBe('{\n  "enabled": false\n}');
	});
	it('labels mode-specific counts without claiming overall completion', () => {
		render(ProgressBar, {
			runType: 'links',
			status: 'running',
			stats: {
				rows_found: 12,
				pages_fetched: 4,
				pages_total: 4,
				queries_executed: 2,
				queries_total: 2,
				elapsed_secs: 65,
				spent_usd: 0.04,
			},
		});
		expect(screen.getByLabelText('Run statistics')).toHaveTextContent('Links 12');
		expect(screen.queryByText('100%')).not.toBeInTheDocument();
	});
});
