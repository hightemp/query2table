import { t } from '$lib/i18n';
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
		toast(t('links.openFailed'), 'error');
		return false;
	}
}

export async function copyWithToast(text: string, message: string) {
	try {
		await copyText(text);
		toast(message, 'success');
	} catch {
		toast(t('common.copyFailed'), 'error');
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
		{ label: t('links.open'), icon: ExternalLinkIcon, action: () => void openLink(url) },
		{ label: t('links.copy'), icon: CopyIcon, separator: true, action: () => copyWithToast(url, t('links.copied')) },
		{
			label: t('links.copyMarkdown'),
			icon: LinkIcon,
			action: () => copyWithToast(markdownLink(title, url), t('links.markdownCopied')),
		},
	];
}
