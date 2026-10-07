import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import Checkbox from '$lib/components/common/Checkbox.svelte';
import Radio from '$lib/components/common/Radio.svelte';

afterEach(cleanup);

describe('Checkbox', () => {
	it('is a real checkbox with a drawn box', async () => {
		const onchange = vi.fn();
		render(Checkbox, { checked: false, label: 'Required', onchange });
		const box = screen.getByRole('checkbox', { name: 'Required' });
		expect(box).not.toBeChecked();
		expect(box.closest('.checkbox')?.querySelector('.box')).toBeInTheDocument();
		await fireEvent.click(box);
		expect(box).toBeChecked();
		expect(onchange).toHaveBeenCalledExactlyOnceWith(true);
	});

	it('shows a mixed state and can be disabled', () => {
		render(Checkbox, { checked: false, indeterminate: true, disabled: true, label: 'All' });
		const box = screen.getByRole('checkbox', { name: 'All' }) as HTMLInputElement;
		expect(box.indeterminate).toBe(true);
		expect(box).toBeDisabled();
	});
});

describe('Radio', () => {
	it('is a real radio that reports being chosen', async () => {
		const onchange = vi.fn();
		render(Radio, { checked: false, name: 'proxy', label: 'Direct connection', onchange });
		const radio = screen.getByRole('radio', { name: 'Direct connection' });
		await fireEvent.click(radio);
		expect(onchange).toHaveBeenCalledOnce();
		expect(radio.closest('.radio')?.querySelector('.dot')).toBeInTheDocument();
	});
});
