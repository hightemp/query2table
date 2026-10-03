import { CopyIcon, ExternalLinkIcon, LinkIcon } from '@lucide/svelte';
import type { MenuItem } from '$lib/components/common/ContextMenu.svelte';
import { copyText } from '$lib/api/tauri';
import { toast } from '$lib/stores/toasts';
import { openExternal } from './links';

export async function openLink(url: string) {
	try {
		await openExternal(url);
		return true;
	} catch {
		toast('Could not open this link. Copy its address and open it in your browser.', 'error');
		return false;
	}
}

export async function copyWithToast(text: string, message: string) {
	try {
		await copyText(text);
		toast(message, 'success');
	} catch {
		toast('Could not copy. Select the text and copy it manually.', 'error');
	}
}

/** Markdown link with brackets in the title escaped. */
export function markdownLink(title: string, url: string) {
	const label = (title.trim() || url).replace(/([\[\]])/g, '\\$1');
	return `[${label}](${url})`;
}

/** Menu for any external link in the app. */
export function linkMenuItems(url: string, title = ''): MenuItem[] {
	return [
		{ label: 'Open link', icon: ExternalLinkIcon, action: () => void openLink(url) },
		{ label: 'Copy link', icon: CopyIcon, separator: true, action: () => copyWithToast(url, 'Link copied.') },
		{
			label: 'Copy as Markdown',
			icon: LinkIcon,
			action: () => copyWithToast(markdownLink(title, url), 'Markdown link copied.'),
		},
	];
}
