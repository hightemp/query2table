import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import Select from '$lib/components/common/Select.svelte';

const formats = [
	{ value: 'csv', label: 'CSV' },
	{ value: 'json', label: 'JSON' },
	{ value: 'xlsx', label: 'Excel', disabled: true },
	{ value: 'md', label: 'Markdown' },
];
const many = Array.from({ length: 10 }, (_, i) => ({ value: `c${i}`, label: `Column ${i}` }));

afterEach(cleanup);

describe('Select', () => {
	it('shows the chosen option and opens a listbox', async () => {
		render(Select, { value: 'json', options: formats, label: 'Format', onchange: vi.fn() });
		const trigger = screen.getByRole('combobox', { name: 'Format' });
		expect(trigger).toHaveTextContent('JSON');
		expect(trigger).toHaveAttribute('aria-expanded', 'false');
		await fireEvent.click(trigger);
		expect(trigger).toHaveAttribute('aria-expanded', 'true');
		expect(screen.getByRole('option', { name: 'JSON' })).toHaveAttribute('aria-selected', 'true');
		expect(screen.getByRole('option', { name: 'Excel' })).toHaveAttribute('aria-disabled', 'true');
		// Short lists have no search.
		expect(screen.queryByRole('searchbox')).toBeNull();
	});

	it('chooses with a click and closes', async () => {
		const onchange = vi.fn();
		render(Select, { value: 'csv', options: formats, label: 'Format', onchange });
		await fireEvent.click(screen.getByRole('combobox'));
		await fireEvent.click(screen.getByRole('option', { name: 'Markdown' }));
		expect(onchange).toHaveBeenCalledExactlyOnceWith('md');
		expect(screen.queryByRole('listbox')).toBeNull();
		// Disabled options cannot be chosen.
		await fireEvent.click(screen.getByRole('combobox'));
		await fireEvent.click(screen.getByRole('option', { name: 'Excel' }));
		expect(onchange).toHaveBeenCalledTimes(1);
	});

	it('works from the keyboard and skips disabled options', async () => {
		const onchange = vi.fn();
		render(Select, { value: 'json', options: formats, label: 'Format', onchange });
		const trigger = screen.getByRole('combobox');
		await fireEvent.keyDown(trigger, { key: 'ArrowDown' });
		expect(trigger).toHaveAttribute('aria-expanded', 'true');
		const active = () => document.getElementById(trigger.getAttribute('aria-activedescendant')!);
		expect(active()).toHaveTextContent('JSON');
		await fireEvent.keyDown(trigger, { key: 'ArrowDown' });
		expect(active()).toHaveTextContent('Markdown');
		await fireEvent.keyDown(trigger, { key: 'Home' });
		expect(active()).toHaveTextContent('CSV');
		await fireEvent.keyDown(trigger, { key: 'm' });
		expect(active()).toHaveTextContent('Markdown');
		await fireEvent.keyDown(trigger, { key: 'Enter' });
		expect(onchange).toHaveBeenCalledExactlyOnceWith('md');
		expect(trigger).toHaveAttribute('aria-expanded', 'false');

		await fireEvent.keyDown(trigger, { key: 'Enter' });
		await fireEvent.keyDown(trigger, { key: 'Escape' });
		expect(trigger).toHaveAttribute('aria-expanded', 'false');
		expect(onchange).toHaveBeenCalledTimes(1);
	});

	it('searches long lists', async () => {
		const onchange = vi.fn();
		render(Select, { value: 'c0', options: many, label: 'Column', onchange });
		await fireEvent.click(screen.getByRole('combobox', { name: 'Column' }));
		const search = screen.getByRole('searchbox');
		expect(search).toHaveFocus();
		await fireEvent.input(search, { target: { value: '7' } });
		expect(screen.getAllByRole('option').map((o) => o.textContent?.trim())).toEqual(['Column 7']);
		await fireEvent.keyDown(search, { key: 'Enter' });
		expect(onchange).toHaveBeenCalledExactlyOnceWith('c7');
	});

	it('says when nothing matches', async () => {
		render(Select, { value: 'c0', options: many, label: 'Column', onchange: vi.fn() });
		await fireEvent.click(screen.getByRole('combobox'));
		await fireEvent.input(screen.getByRole('searchbox'), { target: { value: 'zzz' } });
		expect(screen.getByText('Nothing found')).toBeInTheDocument();
	});
});
