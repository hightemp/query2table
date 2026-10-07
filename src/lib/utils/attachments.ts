import { formatNumber, t, type MessageKey } from '$lib/i18n';
import type { AttachError, AttachmentInfo } from '$lib/types';

/** Extensions offered in the file dialog; keep in sync with `parse::EXTENSIONS` in Rust. */
export const ATTACHMENT_EXTENSIONS = {
	documents: ['pdf', 'docx', 'odt', 'txt', 'md', 'markdown', 'json', 'html', 'htm'],
	spreadsheets: ['xlsx', 'xlsm', 'xls', 'ods', 'csv', 'tsv'],
	images: ['png', 'jpg', 'jpeg', 'webp', 'gif'],
};

export function formatSize(bytes: number): string {
	if (bytes < 1024 * 1024) return t('attachments.kb', { size: formatNumber(Math.max(1, Math.round(bytes / 1024))) });
	return t('attachments.mb', { size: formatNumber(bytes / 1024 / 1024, { maximumFractionDigits: 1 }) });
}

/** One line under a file's name: pages or sheets, amount of text, scans; size for images. */
export function attachmentMeta(info: AttachmentInfo): string {
	if (info.kind === 'image') return formatSize(info.size);
	const parts: string[] = [];
	if (info.page_count != null) parts.push(t('attachments.pages', { count: info.page_count }));
	if (info.sheet_count != null) parts.push(t('attachments.sheets', { count: info.sheet_count }));
	if (info.char_count > 0)
		parts.push(t('attachments.chars', { count: info.char_count, chars: formatNumber(info.char_count, { notation: 'compact', maximumFractionDigits: 1 }) }));
	if (info.scanned_pages > 0) parts.push(t('attachments.scan'));
	return parts.join(' · ') || formatSize(info.size);
}

const KNOWN_ERRORS = ['unsupported', 'empty', 'damaged', 'protected', 'tooLarge', 'unreadable', 'storage', 'tooMany'];

export function attachErrorText(error: Pick<AttachError, 'code' | 'message'>): string {
	return KNOWN_ERRORS.includes(error.code) ? t(`attachments.error.${error.code}` as MessageKey) : error.message;
}

const CLOUD_NAMES: Record<string, string> = { openrouter: 'OpenRouter', ollama_cloud: 'Ollama Cloud' };

function isLocalHost(url: string | undefined): boolean {
	try {
		const host = new URL(url ?? '').hostname;
		return host === 'localhost' || host === '::1' || host === '[::1]' || host.startsWith('127.') || host.endsWith('.local');
	} catch {
		return true;
	}
}

/** Where attached files go with the current model provider. */
export function privacyNote(provider: string, settings: Map<string, string>): string {
	if (CLOUD_NAMES[provider]) return t('attachments.sentTo', { service: CLOUD_NAMES[provider] });
	const url = provider === 'ollama' ? settings.get('ollama_url') : provider === 'openai_compatible' ? settings.get('openai_base_url') : undefined;
	if (provider === 'ollama' && !url) return t('attachments.staysLocal');
	if (isLocalHost(url)) return t('attachments.staysLocal');
	return t('attachments.sentTo', { service: new URL(url!).hostname });
}
