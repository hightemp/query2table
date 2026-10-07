import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import NumberInput from '$lib/components/common/NumberInput.svelte';
import Slider from '$lib/components/common/Slider.svelte';

afterEach(cleanup);

describe('NumberInput', () => {
	it('steps with its own buttons within the limits', async () => {
		const oninput = vi.fn((event: Event) => (event.currentTarget as HTMLInputElement).value);
		render(NumberInput, { value: '9', min: '1', max: '10', step: '1', 'aria-label': 'Rows', oninput });
		const field = screen.getByRole('spinbutton', { name: 'Rows' });
		const up = screen.getByRole('button', { name: 'Increase' });
		const down = screen.getByRole('button', { name: 'Decrease' });
		await fireEvent.pointerDown(up);
		await fireEvent.pointerUp(up);
		expect(field).toHaveValue(10);
		expect(oninput).toHaveLastReturnedWith('10');
		expect(up).toBeDisabled();
		await fireEvent.pointerDown(down);
		await fireEvent.pointerUp(down);
		expect(field).toHaveValue(9);
		// The buttons stay out of the tab order: arrow keys already step the field.
		expect(up).toHaveAttribute('tabindex', '-1');
	});

	it('keeps decimals of the step and starts empty fields from the minimum', async () => {
		const oninput = vi.fn((event: Event) => (event.currentTarget as HTMLInputElement).value);
		render(NumberInput, { value: '', min: '0.01', step: '0.01', 'aria-label': 'Budget', oninput });
		const up = screen.getByRole('button', { name: 'Increase' });
		await fireEvent.pointerDown(up);
		await fireEvent.pointerUp(up);
		expect(oninput).toHaveLastReturnedWith('0.01');
		await fireEvent.pointerDown(up);
		await fireEvent.pointerUp(up);
		expect(oninput).toHaveLastReturnedWith('0.02');
	});

	it('repeats while a button is held', async () => {
		vi.useFakeTimers();
		const oninput = vi.fn();
		render(NumberInput, { value: '0', step: '1', 'aria-label': 'N', oninput });
		const up = screen.getByRole('button', { name: 'Increase' });
		await fireEvent.pointerDown(up);
		await vi.advanceTimersByTimeAsync(400 + 60 * 3);
		await fireEvent.pointerUp(up);
		await vi.advanceTimersByTimeAsync(500);
		expect(oninput.mock.calls.length).toBeGreaterThanOrEqual(4);
		vi.useRealTimers();
	});
});

describe('Slider', () => {
	it('is a range input that shows how full it is', async () => {
		const oninput = vi.fn();
		render(Slider, { value: '0.5', min: 0, max: 2, step: 0.1, 'aria-label': 'Temperature', oninput });
		const slider = screen.getByRole('slider', { name: 'Temperature' });
		expect(slider.style.getPropertyValue('--fill')).toBe('25%');
		await fireEvent.input(slider, { target: { value: '1' } });
		expect(oninput).toHaveBeenCalled();
	});
});
