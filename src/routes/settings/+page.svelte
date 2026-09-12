<script lang="ts">
	import { settings } from '$lib/stores/settings';
	import { onDestroy } from 'svelte';
	import LlmModelPicker from '$lib/components/settings/LlmModelPicker.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import { errorText } from '$lib/utils/errors';
	import type { SettingGroup, SettingDef } from '$lib/types';
	import { EyeIcon, EyeOffIcon, SaveIcon, TrashIcon, PlusIcon } from '@lucide/svelte';

	let settingsMap: Map<string, string> = $state(new Map());
	let dirty = $state(new Set<string>());
	let saving = $state(false);
	let saveError = $state('');
	let showPasswords = $state(new Set<string>());

	const unsubscribe = settings.subscribe((v) => {
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

	onDestroy(unsubscribe);

	const groups: SettingGroup[] = [
		{
			label: 'LLM Provider',
			description: 'Configure your LLM API connection',
			settings: [
				{ key: 'llm_provider', label: 'Provider', description: 'Which LLM service to use', type: 'select', options: [{ label: 'OpenRouter', value: 'openrouter' }, { label: 'Ollama (Local)', value: 'ollama' }, { label: 'Ollama Cloud', value: 'ollama_cloud' }, { label: 'OpenAI-compatible (llama.cpp, etc.)', value: 'openai_compatible' }] },
				{ key: 'openrouter_api_key', provider: 'openrouter', label: 'OpenRouter API Key', description: 'Your OpenRouter API key', type: 'password', placeholder: 'sk-or-...' },
				{ key: 'openrouter_model', provider: 'openrouter', label: 'OpenRouter Model', description: 'Search and select a model from the server', type: 'text' },
				{ key: 'ollama_url', provider: 'ollama', label: 'Ollama URL', description: 'Local Ollama server URL', type: 'text', placeholder: 'http://localhost:11434' },
				{ key: 'ollama_model', provider: 'ollama', label: 'Ollama Model', description: 'Local model name', type: 'text', placeholder: 'llama3' },
				{ key: 'ollama_cloud_url', provider: 'ollama_cloud', label: 'Ollama Cloud URL', description: 'Cloud host URL', type: 'text', placeholder: 'https://ollama.com' },
				{ key: 'ollama_cloud_api_key', provider: 'ollama_cloud', label: 'Ollama Cloud API Key', description: 'Required; create a key at ollama.com/settings/keys', type: 'password' },
				{ key: 'ollama_cloud_model', provider: 'ollama_cloud', label: 'Ollama Cloud Model', description: 'Search and select a model from the server', type: 'text' },
				{ key: 'openai_base_url', provider: 'openai_compatible', label: 'API Base URL', description: 'API base including /v1; e.g. http://localhost:8080/v1 for llama.cpp', type: 'text', placeholder: 'http://localhost:8080/v1' },
				{ key: 'openai_api_key', provider: 'openai_compatible', label: 'API Key (optional)', description: 'Leave empty if your server does not require authentication', type: 'password' },
				{ key: 'openai_model', provider: 'openai_compatible', label: 'Model', description: 'Required model ID from your server; use the loaded model name or alias in llama.cpp', type: 'text', placeholder: 'Your loaded model ID' },
				{ key: 'openai_json_mode', provider: 'openai_compatible', label: 'JSON Mode', description: 'Disable if your server does not support response_format; prompts still request JSON', type: 'select', options: [{ label: 'Enabled', value: 'true' }, { label: 'Disabled', value: 'false' }] },
				{ key: 'llm_temperature', label: 'Temperature', description: 'LLM temperature (0.0 - 1.0)', type: 'number' },
				{ key: 'llm_reasoning_effort', label: 'Thinking / reasoning effort', description: 'Controls model thinking before its answer; supported levels depend on the provider and model', type: 'select', options: [{ label: 'Auto', value: 'auto' }, { label: 'Provider default', value: 'default' }, { label: 'Off', value: 'off' }, { label: 'On', value: 'on' }, { label: 'Low', value: 'low' }, { label: 'Medium', value: 'medium' }, { label: 'High', value: 'high' }, { label: 'Max', value: 'max' }] },
				{ key: 'llm_max_tokens', label: 'Max output tokens', description: 'Requested output cap per model call, potentially shared by thinking and the answer. Separate from run cost/time limits.', type: 'number' },
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
		return settingsMap.get(key) ?? (key === 'llm_reasoning_effort' ? 'auto' : '');
	}

	let activeModel = $derived(settingsMap.get({
		openrouter: 'openrouter_model', ollama: 'ollama_model',
		ollama_cloud: 'ollama_cloud_model', openai_compatible: 'openai_model',
	}[settingsMap.get('llm_provider') || 'openrouter'] ?? 'openrouter_model') ?? '');
	let isGptOss = $derived(['ollama', 'ollama_cloud'].includes(settingsMap.get('llm_provider') ?? '') && /(?:^|\/)gpt-oss(?:[:\-]|$)/i.test(activeModel));
	let invalidReasoning = $derived(isGptOss && ['off', 'max'].includes(getValue('llm_reasoning_effort')));

	function visibleSettings(group: SettingGroup): SettingDef[] {
		const provider = settingsMap.get('llm_provider') || 'openrouter';
		return group.settings.filter((setting) => !setting.provider || setting.provider === provider);
	}

	function handleChange(key: string, value: string) {
		saveError = '';
		settingsMap.set(key, value);
		settingsMap = new Map(settingsMap);
		dirty.add(key);
		dirty = new Set(dirty);
	}

	// --- Proxy management ---
	interface ProxyEntry {
		name: string;
		url: string;
	}

	let proxies = $derived.by<ProxyEntry[]>(() => {
		const raw = settingsMap.get('proxy_list') ?? '[]';
		try {
			const parsed = JSON.parse(raw);
			if (Array.isArray(parsed)) {
				return parsed
					.filter((p) => p && typeof p === 'object')
					.map((p) => ({ name: String(p.name ?? ''), url: String(p.url ?? '') }));
			}
		} catch {
			// ignore malformed JSON
		}
		return [];
	});

	let activeProxy = $derived(settingsMap.get('active_proxy_url') ?? '');

	function persistProxies(list: ProxyEntry[]) {
		handleChange('proxy_list', JSON.stringify(list));
	}

	function addProxy() {
		persistProxies([...proxies, { name: '', url: '' }]);
	}

	function updateProxy(index: number, field: 'name' | 'url', value: string) {
		const list = proxies.map((p, i) => (i === index ? { ...p, [field]: value } : p));
		const removed = proxies[index];
		persistProxies(list);
		// If the active proxy URL changed, keep the selection in sync.
		if (field === 'url' && removed.url === activeProxy) {
			handleChange('active_proxy_url', value);
		}
	}

	function removeProxy(index: number) {
		const removed = proxies[index];
		persistProxies(proxies.filter((_, i) => i !== index));
		if (removed.url === activeProxy) {
			handleChange('active_proxy_url', '');
		}
	}

	function selectActiveProxy(url: string) {
		handleChange('active_proxy_url', url);
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
		if (invalidReasoning) return;
		saving = true;
		saveError = '';
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
			saveError = errorText(e);
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
			<button class="btn-save" onclick={saveAll} disabled={saving || invalidReasoning}>
				<SaveIcon size={16} />
				{saving ? 'Saving...' : `Save (${dirty.size})`}
			</button>
		{/if}
	</div>
	{#if saveError}<ErrorNotice error={saveError} context="settings" />{/if}

	{#each groups as group}
		<section class="settings-section">
			<h2>{group.label}</h2>
			<p class="section-description">{group.description}</p>

			<div class="settings-grid">
				{#each visibleSettings(group) as setting (setting.key)}
					<div class="setting-item">
						<label for={setting.key}>
							<span class="setting-label">{setting.label}</span>
							<span class="setting-description">{setting.description}</span>
						</label>

						{#if setting.key === 'openrouter_model'}
							<LlmModelPicker
								id={setting.key}
								provider="openrouter"
								value={getValue(setting.key)}
								apiKey={getValue('openrouter_api_key')}
								onchange={(model) => handleChange(setting.key, model)}
							/>
						{:else if setting.key === 'ollama_cloud_model'}
							<LlmModelPicker
								id={setting.key}
								provider="ollama_cloud"
								value={getValue(setting.key)}
								baseUrl={settingsMap.get('ollama_cloud_url') ?? 'https://ollama.com'}
								apiKey={getValue('ollama_cloud_api_key')}
								onchange={(model) => handleChange(setting.key, model)}
							/>
						{:else if setting.type === 'select'}
							<select
								id={setting.key}
								value={getValue(setting.key)}
								onchange={(e) => handleChange(setting.key, (e.target as HTMLSelectElement).value)}
							>
								{#each setting.options ?? [] as opt}
									<option value={opt.value} disabled={setting.key === 'llm_reasoning_effort' && isGptOss && ['off', 'max'].includes(opt.value)}>{opt.label}</option>
								{/each}
							</select>
							{#if setting.key === 'llm_reasoning_effort'}
								<p class="reasoning-help">Auto disables thinking for structured Ollama requests (Low for GPT-OSS) and uses the provider default elsewhere. Provider default leaves thinking unchanged. Other choices request that level explicitly. Changes apply after Save and affect new runs.</p>
								{#if isGptOss}<p class="reasoning-help">GPT-OSS cannot use Off or Max. Choose Low, Medium, or High; Auto uses Low with Ollama.</p>{/if}
								{#if invalidReasoning}<ErrorNotice error="GPT-OSS does not support the selected reasoning effort. Choose Auto, Provider default, On, Low, Medium, or High." code="unsupported_setting" context="settings" />{/if}
							{/if}
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

	<section class="settings-section">
		<h2>Network / Proxy</h2>
		<p class="section-description">
			Configure one or more proxies and pick which one to route all requests through.
			Supports <code>http://</code>, <code>https://</code> and <code>socks5://</code> URLs,
			optionally with credentials (e.g. <code>http://user:pass@host:port</code>).
		</p>

		<div class="proxy-list">
			<label class="proxy-radio">
				<input
					type="radio"
					name="active-proxy"
					checked={activeProxy === ''}
					onchange={() => selectActiveProxy('')}
				/>
				<span class="proxy-radio-label">Direct connection (no proxy)</span>
			</label>

			{#each proxies as proxy, i (i)}
				<div class="proxy-row">
					<input
						type="radio"
						name="active-proxy"
						checked={proxy.url !== '' && activeProxy === proxy.url}
						disabled={proxy.url === ''}
						onchange={() => selectActiveProxy(proxy.url)}
						aria-label="Use this proxy"
					/>
					<input
						class="proxy-name"
						type="text"
						placeholder="Name (optional)"
						value={proxy.name}
						oninput={(e) => updateProxy(i, 'name', (e.target as HTMLInputElement).value)}
					/>
					<input
						class="proxy-url"
						type="text"
						placeholder="http://user:pass@host:port"
						value={proxy.url}
						oninput={(e) => updateProxy(i, 'url', (e.target as HTMLInputElement).value)}
					/>
					<button class="btn-remove-proxy" type="button" onclick={() => removeProxy(i)} aria-label="Remove proxy">
						<TrashIcon size={16} />
					</button>
				</div>
			{/each}

			<button class="btn-add-proxy" type="button" onclick={addProxy}>
				<PlusIcon size={16} />
				Add proxy
			</button>
		</div>
	</section>
</div>

<style>
	.reasoning-help { font-size: 0.8rem; line-height: 1.45; color: var(--color-surface-600-400); margin: 6px 0 0; }
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

	.proxy-list {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.proxy-radio {
		display: flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
	}

	.proxy-radio-label {
		font-size: 0.9rem;
	}

	.proxy-row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.proxy-row input[type='text'] {
		padding: 8px 12px;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 6px;
		background: var(--color-surface-200-800);
		color: inherit;
		font-size: 0.9rem;
	}

	.proxy-row input:focus {
		outline: none;
		border-color: var(--color-primary-500);
	}

	.proxy-name {
		flex: 0 0 30%;
	}

	.proxy-url {
		flex: 1;
	}

	.btn-remove-proxy {
		border: none;
		background: transparent;
		cursor: pointer;
		color: var(--color-error-500);
		padding: 6px;
		display: flex;
		align-items: center;
	}

	.btn-add-proxy {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		align-self: flex-start;
		padding: 8px 14px;
		border: 1px dashed var(--color-surface-300-700);
		border-radius: 6px;
		background: transparent;
		color: inherit;
		cursor: pointer;
		font-size: 0.9rem;
	}

	.btn-add-proxy:hover {
		border-color: var(--color-primary-500);
		color: var(--color-primary-500);
	}

	.section-description code {
		background: var(--color-surface-200-800);
		padding: 1px 5px;
		border-radius: 4px;
		font-size: 0.85em;
	}
</style>
