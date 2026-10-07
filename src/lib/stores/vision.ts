import { writable } from 'svelte/store';
import { getVisionStatus, type VisionStatus } from '$lib/api/tauri';

/** Who reads attached pictures and scans with the saved settings; null until known. */
export const visionStatus = writable<VisionStatus | null>(null);

let latest = 0;

export async function refreshVisionStatus() {
	const request = ++latest;
	try {
		const status = await getVisionStatus();
		if (request === latest) visionStatus.set(status);
	} catch {
		// Unknown: the form simply does not warn.
	}
}
