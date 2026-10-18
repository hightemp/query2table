import { describe, expect, it } from 'vitest';
import { isAndroid, readablePath } from '$lib/utils/platform';

describe('platform', () => {
	it('recognises Android by its user agent', () => {
		expect(isAndroid('Mozilla/5.0 (Linux; Android 15; Pixel 8) AppleWebKit/537.36 Chrome/130 Mobile')).toBe(true);
		expect(isAndroid('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15')).toBe(false);
	});

	it('shows Android content addresses as folders and names when they tell them', () => {
		expect(readablePath('/home/me/export.csv')).toBe('/home/me/export.csv');
		expect(
			readablePath('content://com.android.externalstorage.documents/document/primary%3ADownload%2Fexport.csv')
		).toBe('Download/export.csv');
		expect(
			readablePath('content://com.android.providers.downloads.documents/document/raw%3A%2Fstorage%2Femulated%2F0%2FDownload%2Fa.json')
		).toBe('/storage/emulated/0/Download/a.json');
		// Download ids say nothing about the file.
		expect(readablePath('content://com.android.providers.downloads.documents/document/msf%3A1000000033')).toBeNull();
	});
});
