import { writable, type Writable } from 'svelte/store';

/** Reads a per-device UI preference. Storage can be unavailable, so failures read as missing. */
export function readStorage(key: string): string | null {
	try {
		return localStorage.getItem(key);
	} catch {
		return null;
	}
}

export function writeStorage(key: string, value: string) {
	try {
		localStorage.setItem(key, value);
	} catch {
		// Without storage the preference lasts for this session only.
	}
}

/** A store whose JSON value is kept in localStorage. `parse` rejects unexpected values with null. */
export function persisted<T>(key: string, initial: T, parse: (value: unknown) => T | null): Writable<T> {
	let value = initial;
	const raw = readStorage(key);
	if (raw !== null) {
		try {
			value = parse(JSON.parse(raw)) ?? initial;
		} catch {
			// Ignore a malformed saved value.
		}
	}
	const store = writable<T>(value);
	store.subscribe((next) => writeStorage(key, JSON.stringify(next)));
	return store;
}
