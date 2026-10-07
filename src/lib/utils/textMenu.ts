import { ClipboardPasteIcon, CopyIcon, ScissorsIcon, TextSelectIcon } from '@lucide/svelte';
import type { MenuItem } from '$lib/components/common/ContextMenu.svelte';
import { copyText, pasteText } from '$lib/api/tauri';
import { t } from '$lib/i18n';
import { toast } from '$lib/stores/toasts';
import { errorText } from '$lib/utils/errors';

type TextField = HTMLInputElement | HTMLTextAreaElement;

const TEXT_TYPES = new Set(['text', 'search', 'url', 'email', 'tel', 'password', 'number', '']);

function textFieldOf(target: Element | null): TextField | null {
	const field = target?.closest('input, textarea');
	if (field instanceof HTMLTextAreaElement) return field;
	if (field instanceof HTMLInputElement && TEXT_TYPES.has(field.type)) return field;
	return null;
}

/** Replaces the selection like typing would: undoable, and reported with an input event. */
function insert(field: TextField, text: string) {
	field.focus();
	if (!document.execCommand?.('insertText', false, text)) {
		const start = field.selectionStart ?? field.value.length;
		const end = field.selectionEnd ?? start;
		field.setRangeText(text, start, end, 'end');
		field.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: text }));
	}
}

async function guarded(action: () => Promise<void>) {
	try {
		await action();
	} catch (error) {
		toast(errorText(error), 'error');
	}
}

/** Items of the app's menu for text fields and selected text, or null where it has none. */
export function textMenuFor(target: Element | null): MenuItem[] | null {
	const field = textFieldOf(target);
	if (field) {
		// Number fields report no selection; treat their whole value as selected text.
		const hasRange = field.selectionStart !== null;
		const start = hasRange ? field.selectionStart! : 0;
		const end = hasRange ? field.selectionEnd! : field.value.length;
		const selected = field.type === 'password' ? '' : field.value.slice(start, end);
		const editable = !field.readOnly && !field.disabled;
		return [
			{
				label: t('textMenu.cut'),
				icon: ScissorsIcon,
				disabled: !selected || !editable,
				action: () =>
					guarded(async () => {
						await copyText(selected);
						field.setSelectionRange?.(start, end);
						insert(field, '');
					}),
			},
			{
				label: t('textMenu.copy'),
				icon: CopyIcon,
				disabled: !selected,
				action: () => guarded(() => copyText(selected)),
			},
			{
				label: t('textMenu.paste'),
				icon: ClipboardPasteIcon,
				disabled: !editable,
				action: () =>
					guarded(async () => {
						// The field keeps its selection while the menu is open; paste over it.
						const text = await pasteText();
						if (text) insert(field, text);
					}),
			},
			{
				label: t('textMenu.selectAll'),
				icon: TextSelectIcon,
				separator: true,
				action: () => {
					field.focus();
					field.select();
				},
			},
		];
	}
	const selection = window.getSelection()?.toString() ?? '';
	if (!selection.trim()) return null;
	return [{ label: t('textMenu.copy'), icon: CopyIcon, action: () => guarded(() => copyText(selection)) }];
}
