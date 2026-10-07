import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import type { AttachmentInfo } from '$lib/types';
import { attachmentMeta, privacyNote } from '$lib/utils/attachments';
import {
	attachFiles,
	attachPaths,
	draftAttachments,
	MAX_ATTACHMENTS,
	removeDraftAttachment,
	restoreDraftAttachments,
	takeDraftAttachments,
} from '$lib/stores/attachments';
import { addAttachmentData, addAttachments, getAttachments, removeAttachment } from '$lib/api/tauri';

vi.mock('$lib/api/tauri', () => ({
	addAttachments: vi.fn(),
	addAttachmentData: vi.fn(),
	getAttachments: vi.fn(),
	removeAttachment: vi.fn(async () => {}),
}));

function info(id: string, extra: Partial<AttachmentInfo> = {}): AttachmentInfo {
	return {
		id,
		file_name: `${id}.pdf`,
		kind: 'document',
		mime: 'application/pdf',
		size: 1024,
		status: 'ready',
		page_count: 3,
		sheet_count: null,
		char_count: 4200,
		scanned_pages: 0,
		thumbnail: null,
		created_at: 0,
		...extra,
	};
}

beforeEach(() => {
	localStorage.clear();
	draftAttachments.set([]);
	vi.mocked(addAttachments).mockReset();
	vi.mocked(addAttachmentData).mockReset();
	vi.mocked(getAttachments).mockReset();
});
afterEach(() => vi.clearAllMocks());

describe('attachment descriptions', () => {
	it('summarise pages, sheets, text and scans', () => {
		expect(attachmentMeta(info('a'))).toBe('3 pages · 4.2K characters');
		expect(attachmentMeta(info('b', { page_count: 1, scanned_pages: 1, char_count: 0 }))).toBe('1 page · scan');
		expect(attachmentMeta(info('c', { kind: 'spreadsheet', page_count: null, sheet_count: 2, char_count: 900 }))).toBe('2 sheets · 900 characters');
		expect(attachmentMeta(info('d', { kind: 'image', page_count: null, char_count: 0, size: 2_500_000 }))).toBe('2.4 MB');
	});

	it('say where file contents go', () => {
		expect(privacyNote('openrouter', new Map())).toBe('File contents will be sent to OpenRouter.');
		expect(privacyNote('ollama', new Map())).toBe('Files stay on this computer.');
		expect(privacyNote('openai_compatible', new Map([['openai_base_url', 'http://127.0.0.1:8080/v1']]))).toBe('Files stay on this computer.');
		expect(privacyNote('openai_compatible', new Map([['openai_base_url', 'https://api.example.com/v1']]))).toBe('File contents will be sent to api.example.com.');
	});
});

describe('draft attachments', () => {
	it('show files while they are read, then keep the ready ones across restarts', async () => {
		let finish!: (value: unknown) => void;
		vi.mocked(addAttachments).mockReturnValue(new Promise((resolve) => (finish = resolve)) as never);
		const pending = attachPaths(['/docs/a.pdf', '/docs/broken.docx']);
		expect(get(draftAttachments).map((d) => [d.name, d.state])).toEqual([
			['a.pdf', 'reading'],
			['broken.docx', 'reading'],
		]);
		finish([
			{ attachment: info('a'), error: null },
			{ attachment: null, error: { file_name: 'broken.docx', code: 'damaged', message: 'zip' } },
		]);
		await pending;
		expect(get(draftAttachments).map((d) => [d.name, d.state, d.error])).toEqual([
			['a.pdf', 'ready', undefined],
			['broken.docx', 'error', 'The file is damaged and cannot be read.'],
		]);
		expect(JSON.parse(localStorage.getItem('q2t-draft-attachments')!)).toEqual(['a']);

		draftAttachments.set([]);
		vi.mocked(getAttachments).mockResolvedValue([info('a')]);
		await restoreDraftAttachments();
		expect(get(draftAttachments).map((d) => d.info?.id)).toEqual(['a']);
	});

	it('does not add the same file twice and stops at the limit', async () => {
		vi.mocked(addAttachments).mockImplementation(async (paths: string[]) =>
			paths.map((p) => ({ attachment: info(p.split('/').pop()!.split('.')[0]), error: null }))
		);
		await attachPaths(['/a.pdf']);
		await attachPaths(['/a.pdf']);
		expect(get(draftAttachments)).toHaveLength(1);
		const many = Array.from({ length: MAX_ATTACHMENTS + 3 }, (_, i) => `/f${i}.pdf`);
		const refused = await attachPaths(many);
		expect(get(draftAttachments)).toHaveLength(MAX_ATTACHMENTS);
		expect(refused).toBe(4);
	});

	it('reads pasted files and removes drafts', async () => {
		vi.mocked(addAttachmentData).mockResolvedValue({ attachment: info('shot', { kind: 'image' }), error: null });
		await attachFiles([new File([new Uint8Array([1, 2, 3])], 'image.png', { type: 'image/png' })]);
		const [draft] = get(draftAttachments);
		expect(draft.state).toBe('ready');
		await removeDraftAttachment(draft.key);
		expect(removeAttachment).toHaveBeenCalledWith('shot');
		expect(get(draftAttachments)).toEqual([]);
	});

	it('hands ready files to a new run and clears the draft without deleting them', async () => {
		vi.mocked(addAttachments).mockResolvedValue([{ attachment: info('a'), error: null }]);
		await attachPaths(['/a.pdf']);
		expect(takeDraftAttachments()).toEqual(['a']);
		expect(get(draftAttachments)).toEqual([]);
		expect(removeAttachment).not.toHaveBeenCalled();
		expect(localStorage.getItem('q2t-draft-attachments')).toBe('[]');
	});
});
