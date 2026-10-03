const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);

/** Label of the platform's primary modifier key, for hints such as `${modKey}+Enter`. */
export const modKey = isMac ? '⌘' : 'Ctrl';

/** True when the event uses the platform's primary modifier and no other modifiers. */
export function hasMod(event: KeyboardEvent): boolean {
	return (isMac ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey) && !event.altKey;
}
