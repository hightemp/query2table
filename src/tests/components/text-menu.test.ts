import { afterEach, describe, expect, it, vi } from 'vitest';
import { textMenuFor } from '$lib/utils/textMenu';
import { copyText, pasteText } from '$lib/api/tauri';

vi.mock('$lib/api/tauri', () => ({ copyText: vi.fn(async () => {}), pasteText: vi.fn(async () => 'pasted') }));

function field(value: string, start: number, end: number, attrs: Record<string, string> = {}) {
	const input = document.createElement('input');
	for (const [k, v] of Object.entries(attrs)) input.setAttribute(k, v);
	input.value = value;
	document.body.append(input);
	input.focus();
	input.setSelectionRange(start, end);
	return input;
}
const labels = (items: { label: string; disabled?: boolean }[]) =>
	items.map((item) => `${item.label}${item.disabled ? ' (off)' : ''}`);

afterEach(() => {
	document.body.innerHTML = '';
	vi.clearAllMocks();
});

describe('text field menu', () => {
	it('offers cut, copy, paste and select all in fields', () => {
		const input = field('hello world', 0, 5);
		expect(labels(textMenuFor(input)!)).toEqual(['Cut', 'Copy', 'Paste', 'Select all']);
		input.setSelectionRange(3, 3);
		expect(labels(textMenuFor(input)!)).toEqual(['Cut (off)', 'Copy (off)', 'Paste', 'Select all']);
		const readonly = field('fixed', 0, 5, { readonly: '' });
		expect(labels(textMenuFor(readonly)!)).toEqual(['Cut (off)', 'Copy', 'Paste (off)', 'Select all']);
	});

	it('cuts, copies and pastes into the field like typing does', async () => {
		const input = field('hello world', 0, 6);
		const oninput = vi.fn();
		input.addEventListener('input', oninput);
		const [cut, copy, paste, all] = textMenuFor(input)!;
		await copy.action();
		expect(copyText).toHaveBeenLastCalledWith('hello ');
		await cut.action();
		expect(copyText).toHaveBeenLastCalledWith('hello ');
		expect(input.value).toBe('world');
		await paste.action();
		expect(pasteText).toHaveBeenCalled();
		expect(input.value).toBe('pastedworld');
		expect(oninput).toHaveBeenCalledTimes(2);
		await all.action();
		expect([input.selectionStart, input.selectionEnd]).toEqual([0, 11]);
	});

	it('offers Copy for selected page text and nothing elsewhere', async () => {
		const p = document.createElement('p');
		p.textContent = 'Some answer';
		document.body.append(p);
		expect(textMenuFor(p)).toBeNull();
		window.getSelection()!.selectAllChildren(p);
		const items = textMenuFor(p)!;
		expect(labels(items)).toEqual(['Copy']);
		await items[0].action();
		expect(copyText).toHaveBeenCalledWith('Some answer');
	});
});
