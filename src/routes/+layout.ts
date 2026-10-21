// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
export const ssr = false;

import { preloadLanguage } from '$lib/i18n';

// The chosen language's catalog loads before the first render.
export async function load() {
	await preloadLanguage();
}
