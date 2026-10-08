import { formatNumber, t, type MessageKey } from '$lib/i18n';
import type { AttachError, AttachmentInfo } from '$lib/types';

/** Extensions offered in the file dialog; keep in sync with `parse::EXTENSIONS` in Rust. */
export const ATTACHMENT_EXTENSIONS = {
	documents: ['pdf', 'docx', 'odt', 'txt', 'md', 'markdown', 'json', 'html', 'htm'],
	spreadsheets: ['xlsx', 'xlsm', 'xls', 'ods', 'csv', 'tsv'],
	images: ['png', 'jpg', 'jpeg', 'webp', 'gif'],
};

/** Modes that can answer from the attached files alone, without searching the web. */
export const filesOnlyModes: string[] = ['research'];

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

export type FilePlace = { page?: number; sheet?: string; rows?: [number, number]; section?: string };

/** The file id and place of an `attachment://` address, or null for other addresses. */
export function attachmentPlace(url: string | null | undefined): { id: string; place: FilePlace } | null {
	if (!url?.startsWith('attachment://')) return null;
	const [head, query = ''] = url.slice('attachment://'.length).split('?');
	const params = new URLSearchParams(query);
	const place: FilePlace = {};
	const page = Number(params.get('page'));
	if (params.has('page') && Number.isFinite(page)) place.page = page;
	if (params.get('sheet')) place.sheet = params.get('sheet')!;
	const rows = params.get('rows')?.split('-').map(Number);
	if (rows?.length === 2 && rows.every(Number.isFinite)) place.rows = [rows[0], rows[1]];
	if (params.get('section')) place.section = params.get('section')!;
	return { id: decodeURIComponent(head.replace(/\/$/, '')), place };
}

/** "report.pdf, p. 3", "prices.xlsx, sheet Prices, rows 40–60" — a place in a file for people. */
export function placeLabel(fileName: string, place: { page?: number; sheet?: string; rows?: [number, number]; section?: string }): string {
	const parts = [fileName];
	if (place.page != null) parts.push(t('attachments.place.page', { page: place.page }));
	if (place.sheet) parts.push(t('attachments.place.sheet', { sheet: place.sheet }));
	if (place.rows)
		parts.push(
			place.rows[0] === place.rows[1]
				? t('attachments.place.row', { row: place.rows[0] })
				: t('attachments.place.rows', { from: place.rows[0], to: place.rows[1] })
		);
	if (place.section) parts.push(t('attachments.place.section', { section: place.section }));
	return parts.join(', ');
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
