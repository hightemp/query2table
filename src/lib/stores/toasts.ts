import { writable } from 'svelte/store';

export type ToastTone = 'info' | 'success' | 'error';

export interface Toast {
	id: number;
	message: string;
	tone: ToastTone;
}

const DURATION: Record<ToastTone, number> = { info: 3500, success: 3500, error: 7000 };
const MAX_VISIBLE = 4;

export const toasts = writable<Toast[]>([]);
let nextId = 1;
const timers = new Map<number, ReturnType<typeof setTimeout>>();

export function dismissToast(id: number) {
	clearTimeout(timers.get(id));
	timers.delete(id);
	toasts.update((items) => items.filter((item) => item.id !== id));
}

/** Shows a short, self-dismissing message. Repeating the visible message restarts its timer. */
export function toast(message: string, tone: ToastTone = 'info'): number {
	let id = 0;
	toasts.update((items) => {
		const existing = items.find((item) => item.message === message && item.tone === tone);
		id = existing?.id ?? nextId++;
		const next = existing ? items : [...items, { id, message, tone }];
		for (const dropped of next.slice(0, Math.max(0, next.length - MAX_VISIBLE))) {
			clearTimeout(timers.get(dropped.id));
			timers.delete(dropped.id);
		}
		return next.slice(-MAX_VISIBLE);
	});
	clearTimeout(timers.get(id));
	timers.set(
		id,
		setTimeout(() => dismissToast(id), DURATION[tone])
	);
	return id;
}
