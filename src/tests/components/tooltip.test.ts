import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { tooltip } from '$lib/actions/tooltip';

let node: HTMLElement;
const tip = () => document.querySelector<HTMLElement>('[role="tooltip"]');
const visible = () => !!tip() && !tip()!.hidden;

beforeEach(() => {
	vi.useFakeTimers();
	node = document.createElement('span');
	node.textContent = 'Cited';
	document.body.append(node);
});
afterEach(async () => {
	node.dispatchEvent(new PointerEvent('pointerleave'));
	await vi.advanceTimersByTimeAsync(1000);
	node.remove();
	vi.useRealTimers();
});

describe('tooltip', () => {
	it('appears after a short hover and describes the element', async () => {
		const action = tooltip(node, 'Used in the answer');
		node.dispatchEvent(new PointerEvent('pointerenter'));
		expect(visible()).toBe(false);
		await vi.advanceTimersByTimeAsync(500);
		expect(visible()).toBe(true);
		expect(tip()).toHaveTextContent('Used in the answer');
		expect(node.getAttribute('aria-describedby')).toBe(tip()!.id);
		node.dispatchEvent(new PointerEvent('pointerleave'));
		await vi.advanceTimersByTimeAsync(10);
		expect(visible()).toBe(false);
		expect(node.hasAttribute('aria-describedby')).toBe(false);
		action.destroy();
	});

	it('hides with Escape and follows text updates', async () => {
		const action = tooltip(node, 'First');
		action.update('Second');
		node.dispatchEvent(new PointerEvent('pointerenter'));
		await vi.advanceTimersByTimeAsync(500);
		expect(tip()).toHaveTextContent('Second');
		document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
		expect(visible()).toBe(false);
		action.destroy();
	});

	it('shows text of truncated elements only when it is cut off', async () => {
		const action = tooltip(node, { text: 'Cited', whenTruncated: true });
		Object.defineProperty(node, 'scrollWidth', { value: 50, configurable: true });
		Object.defineProperty(node, 'clientWidth', { value: 100, configurable: true });
		node.dispatchEvent(new PointerEvent('pointerenter'));
		await vi.advanceTimersByTimeAsync(600);
		expect(visible()).toBe(false);
		node.dispatchEvent(new PointerEvent('pointerleave'));
		Object.defineProperty(node, 'scrollWidth', { value: 150, configurable: true });
		node.dispatchEvent(new PointerEvent('pointerenter'));
		await vi.advanceTimersByTimeAsync(600);
		expect(visible()).toBe(true);
		// The text is the element's own, so it is not announced twice.
		expect(node.hasAttribute('aria-describedby')).toBe(false);
		action.destroy();
	});

	it('does nothing without text', async () => {
		const action = tooltip(node, '');
		node.dispatchEvent(new PointerEvent('pointerenter'));
		await vi.advanceTimersByTimeAsync(600);
		expect(visible()).toBe(false);
		action.destroy();
	});
});
