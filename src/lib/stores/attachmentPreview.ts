import { writable } from 'svelte/store';

/** The `attachment://` place shown in the file preview, or null when it is closed. */
export const attachmentPreview = writable<string | null>(null);

export function showAttachment(url: string) {
	attachmentPreview.set(url);
}
