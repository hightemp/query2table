import { test as base, expect, type Locator, type Page } from '@playwright/test';
export const test = base.extend({
	page: async ({ page }, use) => {
		const pageErrors: string[] = [];
		page.on('pageerror', (error) => pageErrors.push(error.message));
		await page.addInitScript(() => {
			const callbacks = new Map<number, (event: unknown) => void>();
			const listeners = new Map<number, { event: string; handler: number }>();
			let nextId = 0;
			const values: Record<string, string> = {
				theme: 'dark',
				llm_provider: 'ollama_cloud',
				ollama_cloud_url: 'https://ollama.com',
				ollama_cloud_model: 'deepseek-test',
				ollama_cloud_api_key: 'fixture-key',
				brave_api_key: 'fixture-key',
				llm_reasoning_effort: 'auto',
				llm_temperature: '0.2',
				llm_max_tokens: '4096',
				search_provider: 'brave',
				search_fallback_enabled: 'true',
				brave_price_per_1000: '0',
				serper_price_per_1000: '0',
				max_parallel_fetches: '8',
				fetch_timeout_seconds: '30',
				search_results_per_query: '10',
				max_pages_per_query: '10',
				precision_recall: 'balanced',
				evidence_strictness: 'moderate',
				dedup_similarity_threshold: '.85',
				enable_content_truncation: 'true',
				max_extraction_text_chars: '12000',
				max_pdf_text_chars: '50000',
				max_page_size_kb: '1024',
				proxy_list: '[]',
			};
			const schema = ['Name', 'Details', 'Topics', 'Website', 'Count'].map((name) => ({
				name,
				type: name === 'Website' ? 'url' : name === 'Count' ? 'number' : 'text',
				description: '',
				required: false,
			}));
			const rows = Array.from({ length: 1000 }, (_, i) => ({
				id: `row-${i}`,
				data: {
					Name: `Robot channel ${String(i).padStart(4, '0')}`,
					Details: { language: 'English', enabled: false },
					Topics: ['ROS 2', 'Arduino'],
					Website: `https://example.com/${'long-path-'.repeat(12)}${i}`,
					Count: i,
				},
				confidence: 0.9,
				status: 'final',
				source_count: (i % 3) + 1,
			}));
			const runs = ['table', 'images', 'links', 'research'].map((type, i) => ({
				id: type,
				query: `Saved ${type} research`,
				run_type: type,
				status: 'completed',
				stats: null,
				error: null,
				created_at: 1000 + i,
				completed_at: null,
				title: null as string | null,
				pinned_at: null as number | null,
				dismissed_notices: null as string | null,
				turn_count: type === 'research' ? 1 : 0,
				config: '{}',
				deleted: false,
			}));
			const imageData =
				'data:image/svg+xml,' +
				encodeURIComponent(
					'<svg xmlns="http://www.w3.org/2000/svg" width="300" height="180"><rect width="300" height="180" fill="#7895cc"/></svg>'
				);
			const images = [0, 1].map((i) => ({
				id: `image-${i}`,
				image_url: `https://example.com/image-${i}`,
				thumbnail_url: imageData,
				title: `Robot image ${i}`,
				source_url: 'https://example.com/source',
				width: 300,
				height: 180,
				relevance_score: 0.8,
			}));
			const fixture = {
				rows,
				runs,
				schema,
				images,
				values,
				calls: [] as { command: string; args: any }[],
				failSave: false,
				clipboard: '',
				failDelete: false,
				failCopy: false,
				delayImage: false,
				delayExport: false,
				emit(event: string, payload: any) {
					for (const listener of listeners.values())
						if (listener.event === event)
							callbacks.get(listener.handler)?.({ event, payload, id: 0 });
				},
				pendingImages: [] as ((value: string) => void)[],
				finishExport: null as (() => void) | null,
			};
			(window as any).__uiFixture = fixture;
			(window as any).__TAURI_EVENT_PLUGIN_INTERNALS__ = {
				unregisterListener: (_event: string, id: number) => listeners.delete(id),
			};
			(window as any).__TAURI_INTERNALS__ = {
				transformCallback: (callback: any) => {
					callbacks.set(++nextId, callback);
					return nextId;
				},
				unregisterCallback: (id: number) => callbacks.delete(id),
				invoke: async (command: string, args: any = {}) => {
					fixture.calls.push({ command, args });
					switch (command) {
						case 'plugin:event|listen': {
							const id = ++nextId;
							listeners.set(id, args);
							return id;
						}
						case 'plugin:event|unlisten':
							return;
						case 'get_settings':
							return Object.entries(values).map(([key, value]) => ({ key, value }));
						case 'get_setting':
							return values[args.key] ?? null;
						case 'update_setting':
							if (fixture.failSave) throw new Error('sqlite: database is locked');
							values[args.key] = args.value;
							return;
						case 'update_settings':
							if (fixture.failSave) throw new Error('sqlite: database is locked');
							Object.assign(values, args.values);
							return;
						case 'test_llm_connection':
							return { ok: true, message: 'Connected to Ollama Cloud; “deepseek-test” is available.' };
						case 'test_search_connection':
							return { ok: true, message: 'Brave Search answered with 1 result.' };
						case 'test_proxy':
							return { ok: true, message: 'The proxy works (120 ms).' };
						case 'list_ollama_models':
							return ['llama3:latest', 'qwen3:8b'];
						case 'list_openai_models':
							return ['local-model'];
						case 'export_settings':
							return;
						case 'read_settings_file':
							return { llm_temperature: '0.3', max_parallel_fetches: '4' };
						case 'get_app_paths':
							return {
								data_dir: '/home/demo/' + 'long-folder/'.repeat(15),
								database_file: '/home/demo/app/data.db',
								log_dir: '/home/demo/app/logs',
							};
						case 'list_ollama_cloud_models':
							return ['deepseek-test', ...Array.from({ length: 20 }, (_, i) => `model-${i}`)];
						case 'list_openrouter_models':
							return ['openai/test-model', 'anthropic/test-model'];
						case 'list_runs':
							return fixture.runs;
						case 'get_run_schema':
							return { columns: fixture.schema, confirmed: true };
						case 'get_run_rows':
							return fixture.rows;
						case 'get_run_issues':
							return [];
						case 'get_row_sources':
							return [
								{
									id: 'source',
									row_id: args.rowId,
									url: 'https://example.com/evidence',
									title: 'Evidence title',
									snippet: 'This source supports the saved result.',
								},
							];
						case 'get_image_results':
							return fixture.images;
						case 'get_link_results':
							return [
								{
									id: 'link',
									url: 'https://example.com/' + 'very-long-url-'.repeat(20),
									title: 'Robotics ' + 'resource'.repeat(40),
									description: 'A useful robotics resource.',
									relevance_score: 0.8,
								},
							];
						case 'get_research_result':
							return {
								answer_markdown:
									'# Robotics\n\n- First item\n  - Nested item\n\n[Sources](#sources)\n\n| ' +
									Array(12).fill('Column').join(' | ') +
									' |\n| ' +
									Array(12).fill('---').join(' | ') +
									' |\n| ' +
									Array(12).fill('Value').join(' | ') +
									' |\n\n```\n' +
									'long code '.repeat(100) +
									'\n```\n\n## Sources\n\n[Website](https://example.com)\n\n<script>window.unsafe=true</script>',
								steps: Array.from({ length: 12 }, (_, i) => ({
									id: 'step-' + i,
									step_index: i,
									step_type: 'search',
									content: 'A full research step. '.repeat(60),
									url: 'https://example.com',
								})),
							};
						case 'proxy_image':
							return fixture.delayImage
								? new Promise<string>((resolve) => fixture.pendingImages.push(resolve))
								: imageData;
						case 'delete_run':
							if (fixture.failDelete) throw new Error('sqlite: database is locked');
							fixture.runs = fixture.runs.filter((run) => run.id !== args.runId);
							return;
						case 'start_run':
							return { run_id: 'live' };
						case 'copy_text':
							if (fixture.failCopy) throw new Error('clipboard unavailable');
							fixture.clipboard = args.text;
							return;
						case 'paste_text':
							return fixture.clipboard;
						case 'plugin:dialog|save':
							return '/tmp/query2table-fixture.csv';
						case 'get_run':
							return fixture.runs.find((run: any) => run.id === args.runId) ?? null;
						case 'list_history': {
							// Mirrors the backend: search, filters, pinned first, newest first.
							const f = args.filter;
							const search = (f.search ?? '').toLowerCase();
							const matching = fixture.runs.filter(
								(run: any) =>
									!run.deleted &&
									(!search || `${run.query} ${run.title ?? ''}`.toLowerCase().includes(search)) &&
									(!f.status ||
										(f.status === 'active'
											? ['pending', 'running', 'paused', 'schema_review'].includes(run.status)
											: run.status === f.status))
							);
							const counts: Record<string, number> = {};
							for (const run of matching) counts[run.run_type] = (counts[run.run_type] ?? 0) + 1;
							const sorted = matching
								.filter((run: any) => !f.run_type || run.run_type === f.run_type)
								.sort(
									(a: any, b: any) =>
										(b.pinned_at ?? 0) - (a.pinned_at ?? 0) ||
										(f.sort === 'oldest' ? a.created_at - b.created_at : b.created_at - a.created_at)
								);
							return { runs: sorted.slice(f.offset, f.offset + f.limit), counts };
						}
						case 'delete_runs':
							if (fixture.failDelete) throw new Error('sqlite: database is locked');
							for (const run of fixture.runs as any[]) if (args.runIds.includes(run.id)) run.deleted = true;
							return;
						case 'restore_runs':
							for (const run of fixture.runs as any[]) if (args.runIds.includes(run.id)) run.deleted = false;
							return;
						case 'purge_runs':
							fixture.runs = fixture.runs.filter((run: any) => !(run.deleted && args.runIds.includes(run.id)));
							return;
						case 'rename_run':
							for (const run of fixture.runs as any[]) if (run.id === args.runId) run.title = args.title;
							return;
						case 'pin_run':
							for (const run of fixture.runs as any[])
								if (run.id === args.runId) run.pinned_at = args.pinned ? Date.now() : null;
							return;
						case 'export_runs':
							return args.runIds.map((id: string) => `${args.dir}/${id}.${args.format}`);
						case 'plugin:dialog|open':
							return args?.options?.directory === false ? '/tmp/settings.json' : '/tmp/exports';
						case 'export_run':
							return fixture.delayExport
								? new Promise<void>((resolve) => (fixture.finishExport = resolve))
								: undefined;
						default:
							return;
					}
				},
			};
		});
		await use(page);
		expect(pageErrors).toEqual([]);
	},
});
export { expect };
const FIXTURE_TYPES = ['table', 'images', 'links', 'research'];
/** Opens a saved fixture run from History: 0 table, 1 images, 2 links, 3 research. */
export async function viewRun(page: Page, index: number) {
	await page.goto('/history');
	await page.getByRole('link', { name: `Saved ${FIXTURE_TYPES[index]} research`, exact: true }).click();
}
export async function emit(page: Page, event: string, payload: unknown) {
	await page.evaluate(({ event, payload }) => (window as any).__uiFixture.emit(event, payload), {
		event,
		payload,
	});
}

/** Picks an option of the app's Select by its visible label. */
export async function choose(select: Locator, option: string | RegExp) {
	await select.click();
	await select.page().getByRole('listbox').getByRole('option', { name: option, exact: typeof option === 'string' }).click();
}
