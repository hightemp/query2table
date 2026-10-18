/** The app runs on Android (the WebView reports it in its user agent). */
export function isAndroid(userAgent = typeof navigator === 'undefined' ? '' : navigator.userAgent): boolean {
	return /\bAndroid\b/.test(userAgent);
}

/**
 * A picked file for display. Android save dialogs return content:// addresses; some name the
 * folder and file (`primary:Download/x.csv`, `raw:/storage/...`), others only an id, giving null.
 */
export function readablePath(path: string): string | null {
	if (!path.startsWith('content://')) return path;
	const tail = decodeURIComponent(path.slice(path.lastIndexOf('/') + 1));
	return /^(?:primary|home|raw):(.+)$/.exec(tail)?.[1] ?? null;
}

/** The folder of a saved file, e.g. `Pictures/Query2Table` for `Pictures/Query2Table/a.png`. */
export function folderOf(path: string): string {
	const cut = path.lastIndexOf('/');
	return cut > 0 ? path.slice(0, cut) : path;
}
