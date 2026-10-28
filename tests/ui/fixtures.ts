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
				attachment_count: 0,
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
			const usageTotals = (overrides: Record<string, number> = {}) => ({
				cost_usd: 0,
				reported_usd: 0,
				estimated_usd: 0,
				unpriced_calls: 0,
				llm_calls: 0,
				prompt_tokens: 0,
				completion_tokens: 0,
				reasoning_tokens: 0,
				cached_tokens: 0,
				cache_write_tokens: 0,
				searches: 0,
				downloads: 0,
				web_bytes: 0,
				model_bytes_sent: 0,
				model_bytes_received: 0,
				runs: 0,
				...overrides,
			});
			const usage = {
				totals: usageTotals({
					cost_usd: 1.25,
					reported_usd: 1,
					estimated_usd: 0.25,
					unpriced_calls: 2,
					llm_calls: 10,
					prompt_tokens: 10_000,
					completion_tokens: 2_000,
					cached_tokens: 3_000,
					searches: 4,
					downloads: 7,
					web_bytes: 2 * 1024 * 1024,
					model_bytes_sent: 1.5 * 1024 * 1024,
					model_bytes_received: 512 * 1024,
					runs: 3,
				}),
				previous: usageTotals({ cost_usd: 1, reported_usd: 1, llm_calls: 6, runs: 2 }),
				point: usageTotals({ cost_usd: 0.1, reported_usd: 0.1, llm_calls: 1, prompt_tokens: 800, completion_tokens: 100, web_bytes: 50_000 }),
				by_model: [
					{ provider: 'openrouter', model: 'openai/gpt-4.1-mini', calls: 8, prompt_tokens: 8_000, completion_tokens: 1_500, cached_tokens: 3_000, cost_usd: 1, unpriced_calls: 0 },
					{ provider: 'anthropic', model: 'claude-sonnet-5-5', calls: 2, prompt_tokens: 2_000, completion_tokens: 500, cached_tokens: 0, cost_usd: 0.25, unpriced_calls: 2 },
				],
				by_search: [
					{ provider: 'brave', calls: 3, cost_usd: 0, bytes: 30_000 },
					{ provider: 'serper', calls: 1, cost_usd: 0.001, bytes: 9_000 },
				],
				by_mode: [
					{ run_type: 'research', runs: 1, llm_calls: 2, searches: 1, cost_usd: 0.25 },
					{ run_type: 'table', runs: 2, llm_calls: 8, searches: 3, cost_usd: 1 },
				],
				top_runs: [
					{ run_id: 'table', run_type: 'table', title: 'Robot channels', started_at: 1000, cost_usd: 1, llm_calls: 8, exists: true },
					{ run_id: 'gone', run_type: 'research', title: 'Old idea', started_at: 900, cost_usd: 0.25, llm_calls: 2, exists: false },
				],
			};
			const fixture = {
				rows,
				usage,
				usageTotals,
				budget: { limit_usd: 0, spent_usd: 1.25, month_start: 0 },
				runs,
				schema,
				images,
				values,
				calls: [] as { command: string; args: any; options?: any }[],
				failSave: false,
				importValues: null as Record<string, string> | null,
				clipboard: '',
				runAttachments: [] as unknown[],
				runQueries: [] as unknown[],
				update: null as Record<string, unknown> | null,
				updateCheckError: null as Record<string, unknown> | null,
				updateResult: 'restart' as string | Record<string, unknown>,
				restarted: false,
				researchImages: [] as Record<string, unknown>[],
				components: {
					supported: true,
					error: null as string | null,
					components: [
						{ component: 'ytdlp', installed: false, version: null as string | null, bundled: false },
						{ component: 'deno', installed: true, version: 'v2.9.8' as string | null, bundled: false },
					],
				},
				componentUpdates: [
					{ component: 'ytdlp', current: null as string | null, latest: '2026.10.26', update_available: true },
					{ component: 'deno', current: 'v2.9.8' as string | null, latest: 'v2.9.9', update_available: true },
				],
				failInstall: null as string | null,
				rowSources: null as unknown[] | null,
				visionDetected: false as boolean | null,
				dialogFiles: ['/docs/report.pdf', '/docs/prices.xlsx'] as string[] | null,
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
			/** What the app learns about an attached file, made up from its name. */
			const attachment = (path: string) => {
				const name = path.split('/').pop()!;
				const ext = name.split('.').pop()!;
				const kind = ['png', 'jpg', 'jpeg'].includes(ext) ? 'image' : ['xlsx', 'csv'].includes(ext) ? 'spreadsheet' : 'document';
				return {
					id: `att-${name}`,
					file_name: name,
					kind,
					mime: 'application/octet-stream',
					size: 2_500_000,
					status: name.includes('scan') ? 'needs_vision' : 'ready',
					page_count: kind === 'document' ? 12 : null,
					sheet_count: kind === 'spreadsheet' ? 2 : null,
					char_count: kind === 'image' ? 0 : 48_000,
					scanned_pages: name.includes('scan') ? 12 : 0,
					thumbnail: kind === 'image' ? 'data:image/gif;base64,R0lGODlhAQABAIAAAP///wAAACwAAAAAAQABAAACAkQBADs=' : null,
					created_at: 0,
				};
			};
			const attach = (path: string) =>
				path.includes('broken')
					? { attachment: null, error: { file_name: path.split('/').pop(), code: 'damaged', message: 'bad zip' } }
					: { attachment: attachment(path), error: null };
			(window as any).__TAURI_INTERNALS__ = {
				metadata: {
					currentWindow: { label: 'main' },
					currentWebview: { windowLabel: 'main', label: 'main' },
				},
				transformCallback: (callback: any) => {
					callbacks.set(++nextId, callback);
					return nextId;
				},
				unregisterCallback: (id: number) => callbacks.delete(id),
				invoke: async (command: string, args: any = {}, options?: any) => {
					fixture.calls.push({ command, args, options });
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
						case 'list_anthropic_models':
							return ['claude-haiku-5-5', 'claude-sonnet-5-5'];
						case 'export_settings':
							return;
						case 'read_settings_file':
							return fixture.importValues ?? { llm_temperature: '0.3', max_parallel_fetches: '4' };
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
							if (fixture.rowSources) return fixture.rowSources;
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
						case 'check_update':
							if (fixture.updateCheckError) throw fixture.updateCheckError;
							return fixture.update;
						case 'install_update':
							fixture.emit('update:progress', { downloaded: 4 * 1024 * 1024, total: 8 * 1024 * 1024 });
							await new Promise((resolve) => setTimeout(resolve, 300));
							if (typeof fixture.updateResult !== 'string') throw fixture.updateResult;
							return fixture.updateResult;
						case 'restart_app':
							fixture.restarted = true;
							return;
						case 'plugin:app|version':
							return '0.9.0';
						case 'get_research_images':
							return fixture.researchImages.map((image) => ({ run_id: args.runId, turn_index: 0, media_type: 'image/png', ...image }));
						case 'get_research_image':
							return fixture.researchImages.some((image) => image.url === args.url) ? imageData : null;
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
						case 'save_file_data':
							return args.path;
						case 'get_usage':
							return {
								totals: fixture.usage.totals,
								previous: fixture.usage.previous,
								series: args.request.buckets.map((start: number) => ({ start, totals: fixture.usage.point })),
								by_model: fixture.usage.by_model,
								by_search: fixture.usage.by_search,
								by_mode: fixture.usage.by_mode,
								top_runs: fixture.usage.top_runs,
							};
						case 'get_usage_first_at':
							return new Date(new Date().getFullYear() - 1, 0, 15).getTime() / 1000;
						case 'get_storage_usage':
							return { database_bytes: 5 * 1024 * 1024, attachments_bytes: 1024 * 1024, answer_images_bytes: 200 * 1024 };
						case 'get_components':
							return fixture.components;
						case 'check_component_updates':
							return fixture.componentUpdates;
						case 'install_component': {
							const update = fixture.componentUpdates.find((u) => u.component === args.component)!;
							fixture.emit('components:progress', { component: args.component, stage: 'download', file: 'yt-dlp_linux', downloaded: 15_000_000, total: 40_000_000 });
							await new Promise((resolve) => setTimeout(resolve, 300));
							if (fixture.failInstall) throw fixture.failInstall;
							const status = fixture.components.components.find((c) => c.component === args.component)!;
							Object.assign(status, { installed: true, version: update.latest });
							return update.latest;
						}
						case 'remove_component': {
							const status = fixture.components.components.find((c) => c.component === args.component)!;
							Object.assign(status, { installed: false, version: null });
							return;
						}
						case 'get_monthly_budget':
							return fixture.budget;
						case 'reprice_usage':
							return fixture.usage.totals.unpriced_calls;
						case 'clear_usage': {
							const empty = fixture.usageTotals();
							Object.assign(fixture.usage, { totals: empty, previous: empty, point: empty, by_model: [], by_search: [], by_mode: [], top_runs: [] });
							return;
						}
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
							if (args?.options?.multiple) return fixture.dialogFiles;
							return args?.options?.directory === false ? '/tmp/settings.json' : '/tmp/exports';
						case 'get_vision_status': {
							const all = { ...values, ...(args.overrides ?? {}) };
							const mode = all.llm_vision ?? 'auto';
							const main_sees = mode === 'on' || (mode === 'auto' && fixture.visionDetected === true);
							const model = all.ollama_cloud_model;
							return {
								main_sees,
								detected: fixture.visionDetected,
								reader: main_sees ? model : all.vision_model || null,
							};
						}
						case 'add_attachments':
							return args.paths.map(attach);
						case 'add_attachment_data':
							return attach(args?.name ?? decodeURIComponent(options?.headers?.['x-file-name'] ?? 'pasted'));
						case 'get_attachments':
							return args.ids.map((id: string) => attachment(`/${id.replace(/^att-/, '')}`));
						case 'remove_attachment':
						case 'open_attachment':
							return;
						case 'get_attachment_fragment': {
							const [id, query = ''] = String(args.url).replace('attachment://', '').split('?');
							const params = new URLSearchParams(query);
							return {
								attachment_id: id,
								file_name: id.replace(/^att-/, ''),
								locator: params.has('page') ? { page: Number(params.get('page')) } : {},
								text: `Passage of ${id.replace(/^att-/, '')}: revenue grew to 12 million.`,
							};
						}
						case 'get_run_attachments':
							return fixture.runAttachments;
						case 'get_run_queries':
							return fixture.runQueries;
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
