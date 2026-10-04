import { writable } from 'svelte/store';
import {
	getSettings,
	updateSetting as apiUpdateSetting,
	updateSettings as apiUpdateSettings,
	type Setting,
} from '$lib/api/tauri';

function createSettingsStore() {
	const { subscribe, set, update } = writable<Map<string, string>>(new Map());

	return {
		subscribe,
		async load() {
			const settings: Setting[] = await getSettings();
			const map = new Map<string, string>();
			for (const s of settings) {
				map.set(s.key, s.value);
			}
			set(map);
		},
		async save(key: string, value: string) {
			await apiUpdateSetting(key, value);
			update((map) => {
				map.set(key, value);
				return new Map(map);
			});
		},
		/** Saves several settings in one transaction. */
		async saveMany(values: Record<string, string>) {
			await apiUpdateSettings(values);
			update((map) => {
				for (const [key, value] of Object.entries(values)) map.set(key, value);
				return new Map(map);
			});
		},
		get(map: Map<string, string>, key: string, fallback = ''): string {
			return map.get(key) ?? fallback;
		}
	};
}

export const settings = createSettingsStore();
