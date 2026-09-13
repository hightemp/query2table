import { openUrl } from '@tauri-apps/plugin-opener';
import { webUrl } from './values';
import { debugUi } from './diagnostics';

export async function openExternal(href: string): Promise<void> {
	const url = webUrl(href);
	if (!url) throw new Error('Only HTTP and HTTPS links can be opened.');
	try {
		await openUrl(url);
	} catch (error) {
		debugUi('external_link_failed');
		throw error;
	}
}
