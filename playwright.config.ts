import { defineConfig } from '@playwright/test';
// Keep local readiness checks local even when the desktop uses an environment proxy.
process.env.NO_PROXY = [process.env.NO_PROXY, '127.0.0.1', 'localhost'].filter(Boolean).join(',');
export default defineConfig({
	testDir: './tests/ui',
	timeout: 30_000,
	fullyParallel: true,
	workers: 3,
	use: {
		baseURL: 'http://127.0.0.1:4174',
		trace: 'retain-on-failure',
		screenshot: 'only-on-failure',
	},
	projects: [
		{ name: 'chromium', use: { browserName: 'chromium' } },
		{ name: 'webkit', use: { browserName: 'webkit' } },
	],
	webServer: {
		command: 'npm run build && npm run preview -- --host 127.0.0.1 --port 4174 --strictPort',
		url: 'http://127.0.0.1:4174',
		reuseExistingServer: false,
	},
});
