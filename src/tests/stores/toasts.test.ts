import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { toasts, toast, dismissToast } from '$lib/stores/toasts';

describe('toasts', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		for (const item of get(toasts)) dismissToast(item.id);
	});
	afterEach(() => vi.useRealTimers());

	it('dismisses messages automatically, errors later than others', () => {
		toast('Saved', 'success');
		toast('Failed', 'error');
		vi.advanceTimersByTime(4000);
		expect(get(toasts).map((item) => item.message)).toEqual(['Failed']);
		vi.advanceTimersByTime(4000);
		expect(get(toasts)).toEqual([]);
	});

	it('shows a repeated message once and restarts its timer', () => {
		toast('Copied');
		vi.advanceTimersByTime(3000);
		toast('Copied');
		vi.advanceTimersByTime(3000);
		expect(get(toasts)).toHaveLength(1);
		vi.advanceTimersByTime(1000);
		expect(get(toasts)).toHaveLength(0);
	});

	it('keeps at most four messages', () => {
		for (let i = 0; i < 6; i++) toast(`Message ${i}`);
		expect(get(toasts).map((item) => item.message)).toEqual([
			'Message 2',
			'Message 3',
			'Message 4',
			'Message 5',
		]);
	});
});
