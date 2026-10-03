import { writable } from 'svelte/store';
import type { MenuItem } from '$lib/components/common/ContextMenu.svelte';

export interface OpenMenu {
	x: number;
	y: number;
	label: string;
	items: MenuItem[];
}

/** The app-wide context menu, rendered once by the layout. */
export const contextMenu = writable<OpenMenu | null>(null);

export function showContextMenu(point: { x: number; y: number }, label: string, items: MenuItem[]) {
	contextMenu.set({ ...point, label, items });
}

export function closeContextMenu() {
	contextMenu.set(null);
}
