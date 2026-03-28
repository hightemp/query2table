<script lang="ts">
	import { settings } from '$lib/stores/settings';
	import type { SettingGroup, SettingDef } from '$lib/types';
	import { EyeIcon, EyeOffIcon, SaveIcon } from '@lucide/svelte';

	let settingsMap: Map<string, string> = $state(new Map());
	let dirty = $state(new Set<string>());
	let saving = $state(false);
	let showPasswords = $state(new Set<string>());

	settings.subscribe((v) => {
		// Don't overwrite local edits while saving
		if (saving) return;
		const newMap = new Map(v);
		// Preserve any unsaved local changes
		for (const key of dirty) {
			const localVal = settingsMap.get(key);
			if (localVal !== undefined) {
				newMap.set(key, localVal);
			}
		}
		settingsMap = newMap;
	});

	const groups: SettingGroup[] = [
		{
			label: 'LLM Provider',
			description: 'Configure your LLM API connection',
			settings: [
				{ key: 'llm_provider', label: 'Provider', description: 'Which LLM service to use', type: 'select', options: [{ label: 'OpenRouter', value: 'openrouter' }, { label: 'Ollama (Local)', value: 'ollama' }] },
				{ key: 'openrouter_api_key', label: 'OpenRouter API Key', description: 'Your OpenRouter API key', type: 'password', placeholder: 'sk-or-...' },
				{ key: 'openrouter_model', label: 'Model', description: 'Model identifier', type: 'text', placeholder: 'openai/gpt-4.1-mini' },
				{ key: 'ollama_url', label: 'Ollama URL', description: 'Local Ollama server URL', type: 'text', placeholder: 'http://localhost:11434' },
				{ key: 'ollama_model', label: 'Ollama Model', description: 'Local model name', type: 'text', placeholder: 'llama3' },
				{ key: 'llm_temperature', label: 'Temperature', description: 'LLM temperature (0.0 - 1.0)', type: 'number' },
				{ key: 'llm_max_tokens', label: 'Max Tokens', description: 'Maximum tokens per LLM request', type: 'number' },
			]
		},
		{
			label: 'Search Provider',
			description: 'Configure web search APIs',
			settings: [
				{ key: 'search_provider', label: 'Primary Search', description: 'Which search API to use', type: 'select', options: [{ label: 'Brave Search', value: 'brave' }, { label: 'Serper (Google)', value: 'serper' }] },
				{ key: 'brave_api_key', label: 'Brave Search API Key', description: 'Your Brave Search API key', type: 'password', placeholder: 'BSA...' },
				{ key: 'serper_api_key', label: 'Serper API Key', description: 'Your Serper API key', type: 'password' },
			]
		},
		{
			label: 'Execution',
			description: 'Pipeline execution parameters',
			settings: [
				{ key: 'max_parallel_fetches', label: 'Max Parallel Fetches', description: 'Concurrent page fetches (1-20)', type: 'number' },
				{ key: 'fetch_timeout_seconds', label: 'Fetch Timeout (s)', description: 'HTTP fetch timeout in seconds', type: 'number' },
				{ key: 'search_results_per_query', label: 'Results per Query', description: 'Search results to fetch per query', type: 'number' },
				{ key: 'max_pages_per_query', label: 'Pages per Query', description: 'Max pages to fetch per search query', type: 'number' },
			]
		},
		{
			label: 'Quality',
			description: 'Result quality thresholds',
			settings: [
				{ key: 'precision_recall', label: 'Precision / Recall', description: 'Balance between accuracy and coverage', type: 'select', options: [{ label: 'Favor Recall', value: 'recall' }, { label: 'Balanced', value: 'balanced' }, { label: 'Favor Precision', value: 'precision' }] },
				{ key: 'evidence_strictness', label: 'Evidence Strictness', description: 'How strictly to require source evidence', type: 'select', options: [{ label: 'Low', value: 'low' }, { label: 'Moderate', value: 'moderate' }, { label: 'Strict', value: 'strict' }] },
				{ key: 'dedup_similarity_threshold', label: 'Dedup Threshold', description: 'Similarity threshold for deduplication (0.0-1.0)', type: 'number' },
			]
		},
		{
			label: 'Content Processing',
			description: 'Configure document truncation and size limits',
			settings: [
				{ key: 'enable_content_truncation', label: 'Enable Truncation', description: 'Enable or disable document text truncation', type: 'select', options: [{ label: 'Enabled', value: 'true' }, { label: 'Disabled', value: 'false' }] },
				{ key: 'max_extraction_text_chars', label: 'Max Extraction Text (chars)', description: 'Max characters of page text sent to LLM for extraction', type: 'number' },
				{ key: 'max_pdf_text_chars', label: 'Max PDF Text (chars)', description: 'Max characters extracted from PDF documents', type: 'number' },
				{ key: 'max_page_size_kb', label: 'Max Page Size (KB)', description: 'Max download size for a single page in kilobytes', type: 'number' },
			]
		},

	];

	function getValue(key: string): string {
		return settingsMap.get(key) ?? '';
	}

	function handleChange(key: string, value: string) {
		settingsMap.set(key, value);
		settingsMap = new Map(settingsMap);
		dirty.add(key);
		dirty = new Set(dirty);
	}

	function togglePassword(key: string) {
		if (showPasswords.has(key)) {
			showPasswords.delete(key);
		} else {
			showPasswords.add(key);
		}
		showPasswords = new Set(showPasswords);
	}

	async function saveAll() {
		saving = true;
		try {
			// Snapshot values before saving to avoid subscription race
			const toSave = new Map<string, string>();
			for (const key of dirty) {
				const val = settingsMap.get(key);
				if (val !== undefined) {
					toSave.set(key, val);
				}
			}
			for (const [key, val] of toSave) {
				await settings.save(key, val);
			}
			dirty = new Set();
		} catch (e) {
			console.error('Failed to save settings:', e);
		} finally {
			saving = false;
		}
	}
</script>

<div class="settings-page">
	<div class="settings-header">
		<div>
			<h1>Settings</h1>
			<p class="subtitle">Configure API keys, models, and execution parameters.</p>
		</div>
		{#if dirty.size > 0}
			<button class="btn-save" onclick={saveAll} disabled={saving}>
				<SaveIcon size={16} />
				{saving ? 'Saving...' : `Save (${dirty.size})`}
			</button>
		{/if}
	</div>

	{#each groups as group}
		<section class="settings-section">
			<h2>{group.label}</h2>
			<p class="section-description">{group.description}</p>

			<div class="settings-grid">
				{#each group.settings as setting}
					<div class="setting-item">
						<label for={setting.key}>
							<span class="setting-label">{setting.label}</span>
							<span class="setting-description">{setting.description}</span>
						</label>

						{#if setting.type === 'select'}
							<select
								id={setting.key}
								value={getValue(setting.key)}
								onchange={(e) => handleChange(setting.key, (e.target as HTMLSelectElement).value)}
							>
								{#each setting.options ?? [] as opt}
									<option value={opt.value}>{opt.label}</option>
								{/each}
							</select>
						{:else if setting.type === 'password'}
							<div class="password-field">
								<input
									id={setting.key}
									type={showPasswords.has(setting.key) ? 'text' : 'password'}
									value={getValue(setting.key)}
									placeholder={setting.placeholder}
									oninput={(e) => handleChange(setting.key, (e.target as HTMLInputElement).value)}
								/>
								<button class="btn-toggle-pw" onclick={() => togglePassword(setting.key)} type="button" aria-label="Toggle visibility">
									{#if showPasswords.has(setting.key)}
										<EyeOffIcon size={16} />
									{:else}
										<EyeIcon size={16} />
									{/if}
								</button>
							</div>
						{:else if setting.type === 'number'}
							<input
								id={setting.key}
								type="number"
								value={getValue(setting.key)}
								oninput={(e) => handleChange(setting.key, (e.target as HTMLInputElement).value)}
							/>
						{:else}
							<input
								id={setting.key}
								type="text"
								value={getValue(setting.key)}
								placeholder={setting.placeholder}
								oninput={(e) => handleChange(setting.key, (e.target as HTMLInputElement).value)}
							/>
						{/if}
					</div>
				{/each}
			</div>
		</section>
	{/each}
</div>

<style>
	.settings-page {
		max-width: 800px;
		margin: 0 auto;
		overflow-y: auto;
		flex: 1;
		min-height: 0;
	}

	.settings-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 24px;
	}

	h1 {
		font-size: 1.8rem;
		font-weight: 700;
		margin-bottom: 4px;
	}

	.subtitle {
		color: var(--color-surface-600-400);
	}

	.btn-save {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 20px;
		background: var(--color-primary-500);
		color: white;
		border: none;
		border-radius: 8px;
		font-weight: 600;
		cursor: pointer;
		transition: background 0.15s;
		flex-shrink: 0;
	}

	.btn-save:hover:not(:disabled) {
		background: var(--color-primary-600);
	}

	.btn-save:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.settings-section {
		margin-bottom: 32px;
		padding: 20px;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 12px;
		background: var(--color-surface-100-900);
	}

	h2 {
		font-size: 1.2rem;
		font-weight: 600;
		margin-bottom: 4px;
	}

	.section-description {
		color: var(--color-surface-600-400);
		font-size: 0.9rem;
		margin-bottom: 16px;
	}

	.settings-grid {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.setting-item {
		display: grid;
		grid-template-columns: 1fr 1fr;
		align-items: center;
		gap: 12px;
	}

	.setting-item label {
		display: flex;
		flex-direction: column;
	}

	.setting-label {
		font-weight: 500;
	}

	.setting-description {
		font-size: 0.82rem;
		color: var(--color-surface-600-400);
	}

	.setting-item input,
	.setting-item select {
		padding: 8px 12px;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 6px;
		background: var(--color-surface-200-800);
		color: inherit;
		font-size: 0.95rem;
	}

	.setting-item input:focus,
	.setting-item select:focus {
		outline: none;
		border-color: var(--color-primary-500);
	}

	.password-field {
		position: relative;
		display: flex;
	}

	.password-field input {
		flex: 1;
		padding-right: 36px;
	}

	.btn-toggle-pw {
		position: absolute;
		right: 8px;
		top: 50%;
		transform: translateY(-50%);
		border: none;
		background: transparent;
		cursor: pointer;
		color: var(--color-surface-600-400);
		padding: 4px;
	}
</style>
