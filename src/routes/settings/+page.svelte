<script lang="ts">
	import { onDestroy, onMount, tick } from 'svelte';
	import { beforeNavigate, goto } from '$app/navigation';
	import { open as openFile, save as saveFile } from '@tauri-apps/plugin-dialog';
	import {
		SaveIcon,
		TrashIcon,
		PlusIcon,
		SearchIcon,
		ChevronDownIcon,
		ChevronRightIcon,
		RotateCcwIcon,
		PlugIcon,
		DownloadIcon,
		UploadIcon,
		CircleCheckIcon,
		TriangleAlertIcon,
		SunIcon,
		MoonIcon,
		MonitorIcon,
	} from '@lucide/svelte';
	import { settings } from '$lib/stores/settings';
	import { language, setLanguage, t, type LanguagePreference, type MessageKey } from '$lib/i18n';
	import {
		darkTheme,
		lightTheme,
		setTheme,
		setThemeChoice,
		setUiScale,
		themePreference,
		uiScale,
		type ThemePreference,
	} from '$lib/stores/ui';
	import { UI_SCALES, type UiScale } from '$lib/themes';
	import ThemePicker from '$lib/components/settings/ThemePicker.svelte';
	import { toast } from '$lib/stores/toasts';
	import Dialog from '$lib/components/common/Dialog.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import LlmModelPicker from '$lib/components/settings/LlmModelPicker.svelte';
	import ModelPricing from '$lib/components/settings/ModelPricing.svelte';
	import AppFiles from '$lib/components/settings/AppFiles.svelte';
	import SettingField from '$lib/components/settings/SettingField.svelte';
	import {
		exportSettings,
		readSettingsFile,
		testLlmConnection,
		testProxy,
		testSearchConnection,
		type ConnectionReport,
	} from '$lib/api/tauri';
	import {
		SECTIONS,
		fieldsIn,
		groupLabel,
		sectionDescription,
		sectionLabel,
		isDefault,
		maskProxyUrl,
		matchingFields,
		sectionResetValues,
		validationErrors,
		type FieldDef,
		type SectionId,
	} from '$lib/settings/schema';
	import { configurationProblems } from '$lib/utils/runConfig';
	import { validPricingOverrides } from '$lib/utils/pricing';
	import { debugUi } from '$lib/utils/diagnostics';
	import { errorText, presentError } from '$lib/utils/errors';
	import { hasMod, modKey } from '$lib/utils/shortcuts';

	let settingsMap: Map<string, string> = $state(new Map());
	let dirty = $state(new Set<string>());
	let saving = $state(false);
	let savedValues = new Map<string, string>();
	let savedFeedback = $state(false);
	let leaveAction = $state<(() => void | Promise<void>) | null>(null);
	let allowLeave = false;
	let saveError = $state('');
	let query = $state('');
	let advancedOpen = $state<string[]>([]);
	let activeSection = $state<SectionId>('llm');
	let scroller = $state<HTMLDivElement>();
	let searchInput = $state<HTMLInputElement>();
	let importMessage = $state('');
	type Check = { running: boolean; report: ConnectionReport | null; error: string };
	let checks = $state<Record<string, Check>>({});
	let proxyFocus = $state<number | null>(null);

	const unsubscribe = settings.subscribe((v) => {
		// Don't overwrite local edits while saving
		if (saving) return;
		savedValues = new Map(v);
		const next = new Map(v);
		// Preserve any unsaved local changes
		for (const key of dirty) {
			const local = settingsMap.get(key);
			if (local !== undefined) next.set(key, local);
		}
		settingsMap = next;
	});
	onDestroy(unsubscribe);

	function value(field: FieldDef | string): string {
		const key = typeof field === 'string' ? field : field.key;
		const def = typeof field === 'string' ? '' : field.default;
		return settingsMap.get(key) ?? def;
	}

	function handleChange(key: string, next: string) {
		saveError = '';
		settingsMap.set(key, next);
		settingsMap = new Map(settingsMap);
		savedFeedback = false;
		if (next === (savedValues.get(key) ?? '')) dirty.delete(key);
		else dirty.add(key);
		dirty = new Set(dirty);
	}

	// --- Derived state ---
	let errors = $derived(validationErrors(settingsMap));
	let problems = $derived(configurationProblems(settingsMap));
	let provider = $derived(settingsMap.get('llm_provider') || 'openrouter');
	let modelKey = $derived(
		({ openrouter: 'openrouter_model', ollama: 'ollama_model', ollama_cloud: 'ollama_cloud_model', openai_compatible: 'openai_model' } as Record<string, string>)[provider] ??
			'openrouter_model'
	);
	let activeModel = $derived(settingsMap.get(modelKey) ?? '');
	let activeEndpoint = $derived(
		provider === 'openrouter'
			? 'https://openrouter.ai/api/v1'
			: provider === 'ollama'
				? (settingsMap.get('ollama_url') ?? 'http://localhost:11434')
				: provider === 'ollama_cloud'
					? (settingsMap.get('ollama_cloud_url') ?? 'https://ollama.com')
					: (settingsMap.get('openai_base_url') ?? 'http://localhost:8080/v1')
	);
	let isGptOss = $derived(['ollama', 'ollama_cloud'].includes(provider) && /(?:^|\/)gpt-oss(?:[:\-]|$)/i.test(activeModel));
	let invalidReasoning = $derived(isGptOss && ['off', 'max'].includes(settingsMap.get('llm_reasoning_effort') ?? 'auto'));
	let invalidPricing = $derived(!validPricingOverrides(settingsMap.get('llm_pricing_overrides') ?? '{}'));
	// Values saved earlier are flagged but only edited ones block saving.
	let blocked = $derived(invalidReasoning || invalidPricing || Object.keys(errors).some((key) => dirty.has(key)));
	let searching = $derived(!!query.trim());
	let matches = $derived(new Set(matchingFields(query, settingsMap).map((f) => f.key)));

	/** Error marks beat unsaved-change marks in the section list. */
	function sectionState(id: SectionId): 'error' | 'changed' | undefined {
		const fields = fieldsIn(id, settingsMap);
		if (fields.some((f) => errors[f.key]) || (id === 'llm' && (invalidReasoning || invalidPricing))) return 'error';
		const keys = id === 'network' ? ['proxy_list', 'active_proxy_url'] : id === 'llm' ? [...fields.map((f) => f.key), 'llm_pricing_overrides'] : fields.map((f) => f.key);
		return keys.some((key) => dirty.has(key)) ? 'changed' : undefined;
	}

	function visible(field: FieldDef): boolean {
		return searching ? matches.has(field.key) : true;
	}
	function advancedShown(section: SectionId): boolean {
		if (searching || advancedOpen.includes(section)) return true;
		// Opens by itself when something inside needs attention.
		return fieldsIn(section, settingsMap).some(
			(f) => f.advanced && (errors[f.key] || dirty.has(f.key))
		);
	}
	function toggleAdvanced(section: SectionId) {
		advancedOpen = advancedOpen.includes(section) ? advancedOpen.filter((s) => s !== section) : [...advancedOpen, section];
	}
	function sectionVisible(id: SectionId): boolean {
		return !searching || fieldsIn(id, settingsMap).some((f) => matches.has(f.key));
	}

	// --- Navigation ---
	function goToSection(id: string) {
		document.getElementById(`settings-${id}`)?.scrollIntoView({ block: 'start' });
		activeSection = id as SectionId;
	}
	function updateActive() {
		if (!scroller) return;
		const top = scroller.getBoundingClientRect().top + 48;
		let current: SectionId = SECTIONS[0].id;
		for (const section of SECTIONS) {
			const element = document.getElementById(`settings-${section.id}`);
			if (element && element.getBoundingClientRect().top <= top) current = section.id;
		}
		if (scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 2)
			current = SECTIONS.filter((s) => sectionVisible(s.id)).at(-1)?.id ?? current;
		activeSection = current;
	}
	async function focusField(key: string, section: string) {
		const field = fieldsIn(section as SectionId, settingsMap).find((f) => f.key === key);
		if (field?.advanced && !advancedOpen.includes(section)) advancedOpen = [...advancedOpen, section];
		query = '';
		await tick();
		const element = document.getElementById(key);
		element?.scrollIntoView({ block: 'center' });
		element?.focus();
	}

	// --- Reset ---
	function resetSection(id: SectionId) {
		for (const [key, next] of Object.entries(sectionResetValues(id, settingsMap))) handleChange(key, next);
	}

	// --- Connection checks ---
	function formValues(): Record<string, string> {
		return Object.fromEntries(settingsMap);
	}
	async function runCheck(name: string, check: () => Promise<ConnectionReport>) {
		checks = { ...checks, [name]: { running: true, report: null, error: '' } };
		try {
			const report = await check();
			checks = { ...checks, [name]: { running: false, report, error: '' } };
		} catch (error) {
			checks = { ...checks, [name]: { running: false, report: null, error: errorText(error) } };
		}
	}

	// --- Proxies ---
	interface ProxyEntry {
		name: string;
		url: string;
	}
	let proxies = $derived.by<ProxyEntry[]>(() => {
		try {
			const parsed = JSON.parse(settingsMap.get('proxy_list') ?? '[]');
			if (Array.isArray(parsed))
				return parsed
					.filter((p) => p && typeof p === 'object')
					.map((p) => ({ name: String(p.name ?? ''), url: String(p.url ?? '') }));
		} catch {
			// ignore malformed JSON
		}
		return [];
	});
	let activeProxy = $derived(settingsMap.get('active_proxy_url') ?? '');
	function persistProxies(list: ProxyEntry[]) {
		handleChange('proxy_list', JSON.stringify(list));
	}
	function updateProxy(index: number, key: 'name' | 'url', next: string) {
		const previous = proxies[index];
		persistProxies(proxies.map((p, i) => (i === index ? { ...p, [key]: next } : p)));
		// Keep the active proxy pointing at the edited entry.
		if (key === 'url' && previous.url === activeProxy && previous.url) handleChange('active_proxy_url', next);
	}
	function removeProxy(index: number) {
		const removed = proxies[index];
		persistProxies(proxies.filter((_, i) => i !== index));
		if (removed.url === activeProxy) handleChange('active_proxy_url', '');
	}

	// --- Theme ---
	const themes: { value: ThemePreference; label: MessageKey; icon: typeof SunIcon }[] = [
		{ value: 'light', label: 'theme.light', icon: SunIcon },
		{ value: 'dark', label: 'theme.dark', icon: MoonIcon },
		{ value: 'system', label: 'theme.system', icon: MonitorIcon },
	];
	async function changeLanguage(next: LanguagePreference) {
		try {
			await setLanguage(next);
		} catch {
			toast(t('settings.languageFailed'), 'error');
		}
	}
	/** Connection results with a code are shown in the interface language. */
	function reportText(report: ConnectionReport): string {
		if (!report.code) return report.message;
		const params: Record<string, string | number> = {};
		for (const [key, text] of Object.entries(report.params ?? {}))
			params[key] = /^\d+$/.test(text) ? Number(text) : text;
		if (params.service === 'the server') params.service = t('connection.theServer');
		const note = report.params?.note === 'cloudKeyNote' ? t('connection.cloudKeyNote') : '';
		return t(`connection.${report.code}` as MessageKey, params) + note;
	}
	async function changeTheme(next: ThemePreference) {
		try {
			await setTheme(next);
		} catch {
			toast(t('theme.saveFailed'), 'error');
		}
	}
	async function chooseTheme(id: string) {
		try {
			await setThemeChoice(id);
		} catch {
			toast(t('theme.saveFailed'), 'error');
		}
	}
	async function changeScale(next: UiScale) {
		try {
			await setUiScale(next);
		} catch {
			toast(t('theme.scaleFailed'), 'error');
		}
	}

	// --- Import / export ---
	async function exportToFile() {
		try {
			const path = await saveFile({
				defaultPath: 'query2table-settings.json',
				filters: [{ name: t('settings.fileFilter'), extensions: ['json'] }],
			});
			if (!path) return;
			await exportSettings(path);
			toast(t('settings.exported'), 'success');
		} catch (error) {
			toast(errorText(error), 'error');
		}
	}
	async function importFromFile() {
		importMessage = '';
		try {
			const path = await openFile({
				directory: false,
				multiple: false,
				filters: [{ name: t('settings.fileFilter'), extensions: ['json'] }],
			});
			if (!path || Array.isArray(path)) return;
			const values = await readSettingsFile(path);
			const entries = Object.entries(values);
			for (const [key, next] of entries) handleChange(key, next);
			importMessage = t('settings.imported', { count: entries.length });
		} catch (error) {
			toast(errorText(error), 'error');
		}
	}

	// --- Save ---
	async function saveAll(): Promise<boolean> {
		if (saving || blocked || !dirty.size) return !dirty.size;
		saving = true;
		saveError = '';
		const snapshot = Object.fromEntries([...dirty].map((key) => [key, settingsMap.get(key) ?? '']));
		debugUi('settings_save_started', { count: Object.keys(snapshot).length });
		try {
			await settings.saveMany(snapshot);
			for (const [key, saved] of Object.entries(snapshot)) {
				savedValues.set(key, saved);
				// An edit made while saving stays unsaved.
				if (settingsMap.get(key) === saved) dirty.delete(key);
			}
			dirty = new Set(dirty);
			savedFeedback = dirty.size === 0;
			importMessage = '';
			debugUi('settings_save_finished', { remaining: dirty.size });
			return dirty.size === 0;
		} catch (error) {
			saveError = errorText(error);
			debugUi('settings_save_failed', { remaining: dirty.size });
			return false;
		} finally {
			saving = false;
		}
	}
	function discard() {
		if (saving) return;
		settingsMap = new Map(savedValues);
		dirty = new Set();
		saveError = '';
		savedFeedback = false;
		importMessage = '';
	}
	async function leave(save: boolean) {
		if (save && !(await saveAll())) return;
		if (!save) discard();
		const action = leaveAction;
		leaveAction = null;
		allowLeave = true;
		try {
			await action?.();
		} finally {
			allowLeave = false;
		}
	}
	beforeNavigate((navigation) => {
		if (!dirty.size || allowLeave) return;
		navigation.cancel();
		if (navigation.to?.url) leaveAction = () => goto(navigation.to!.url.href);
	});
	onMount(() => {
		// Links such as /settings#settings-search open the matching section.
		if (location.hash) {
			const id = location.hash.slice(1).replace(/^settings-/, '');
			// Older links used the previous section names.
			const moved: Record<string, string> = { execution: 'runs', quality: 'runs', content: 'runs', files: 'app' };
			document.getElementById(`settings-${moved[id] ?? id}`)?.scrollIntoView();
		}
		let disposed = false;
		let unlisten: (() => void) | undefined;
		// Browser fixtures have no native window metadata. Register only in the desktop shell.
		if ((window as any).__TAURI_INTERNALS__?.metadata?.currentWindow) {
			void import('@tauri-apps/api/window')
				.then(async ({ getCurrentWindow }) => {
					const appWindow = getCurrentWindow();
					const off = await appWindow.onCloseRequested((event) => {
						if (dirty.size && !allowLeave) {
							event.preventDefault();
							leaveAction = () => appWindow.destroy();
						}
					});
					if (disposed) off();
					else unlisten = off;
				})
				.catch((error) => {
					if (!disposed) saveError = errorText(error);
				});
		}
		return () => {
			disposed = true;
			unlisten?.();
		};
	});

	const pickerProviders = ['openrouter_model', 'ollama_model', 'ollama_cloud_model', 'openai_model'];
</script>

<svelte:window
	onkeydown={(event) => {
		if (!hasMod(event) || event.shiftKey) return;
		const key = event.key.toLowerCase();
		if (key === 's') {
			event.preventDefault();
			if (dirty.size) void saveAll();
		} else if (key === 'f') {
			event.preventDefault();
			searchInput?.focus();
		}
	}}
/>

{#snippet checkResult(name: string)}
	{@const check = checks[name]}
	{#if check?.running}<p class="check" role="status">{t('settings.checking')}</p>
	{:else if check?.report}<p class="check" class:ok={check.report.ok} class:warn={!check.report.ok} role="status">
			{#if check.report.ok}<CircleCheckIcon size={15} />{:else}<TriangleAlertIcon size={15} />{/if}{reportText(check.report)}
		</p>
	{:else if check?.error}<p class="check fail" role="status">
			<TriangleAlertIcon size={15} />{presentError(check.error, 'settings').title}. <span class="raw">{check.error}</span>
		</p>{/if}
{/snippet}

{#snippet fieldView(field: FieldDef)}
	<SettingField
		{field}
		value={value(field)}
		error={errors[field.key] ?? null}
		changed={dirty.has(field.key)}
		disabledOptions={field.key === 'llm_reasoning_effort' && isGptOss ? ['off', 'max'] : []}
		onchange={(next) => handleChange(field.key, next)}
	>
		{#snippet control()}
			{#if pickerProviders.includes(field.key)}
				<LlmModelPicker
					id={field.key}
					provider={field.provider ?? 'openrouter'}
					value={value(field)}
					baseUrl={field.key === 'ollama_model'
						? value('ollama_url') || 'http://localhost:11434'
						: field.key === 'openai_model'
							? value('openai_base_url') || 'http://localhost:8080/v1'
							: value('ollama_cloud_url') || 'https://ollama.com'}
					apiKey={field.key === 'openrouter_model'
						? value('openrouter_api_key')
						: field.key === 'openai_model'
							? value('openai_api_key')
							: value('ollama_cloud_api_key')}
					allowCustom={field.key === 'ollama_model' || field.key === 'openai_model'}
					onchange={(model) => handleChange(field.key, model)}
				/>
			{/if}
		{/snippet}
		{#snippet help()}
			{#if field.key === 'llm_reasoning_effort'}
				<p class="help">{t('settings.reasoningHelp')}</p>
				{#if isGptOss}<p class="help">{t('settings.gptOssHelp')}</p>{/if}
				{#if invalidReasoning}<ErrorNotice
						error={t('settings.gptOssError')}
						code="unsupported_setting"
						context="settings"
					/>{/if}
			{/if}
		{/snippet}
	</SettingField>
{/snippet}

<div class="settings-page">
	<header class="page-header settings-header">
		<div>
			<h1>{t('settings.title')}</h1>
			<p>{t('settings.subtitle')}</p>
		</div>
		<div class="save-actions">
			<span role="status"
				>{saving
					? t('settings.saving')
					: dirty.size
						? t('settings.unsavedChanges', { count: dirty.size })
						: savedFeedback
							? t('settings.saved')
							: t('settings.allSaved')}</span
			>
			<button class="button" onclick={discard} disabled={saving || !dirty.size}>{t('settings.discard')}</button>
			<button
				class="button primary"
				onclick={() => saveAll()}
				title={blocked ? t('settings.fixToSave') : t('settings.saveHint', { shortcut: `${modKey}+S` })}
				aria-keyshortcuts={modKey === '⌘' ? 'Meta+S' : 'Control+S'}
				disabled={saving || blocked || !dirty.size}
				><SaveIcon size={16} />{saving ? t('settings.savingShort') : t('settings.save')}</button
			>
		</div>
	</header>

	<div class="toolbar">
		<label class="search"
			><SearchIcon size={15} /><input
				bind:this={searchInput}
				type="search"
				class="input sm"
				placeholder={t('settings.searchPlaceholder')}
				aria-label={t('settings.search')}
				bind:value={query}
			/></label
		>
		<div class="readiness" role="status" aria-label={t('settings.setupStatus')}>
			{#if !settingsMap.size}
				<span class="muted">{t('settings.loading')}</span>
			{:else if problems.length}
				<TriangleAlertIcon size={15} />
				{#each problems as problem}<a
						href={`#${problem.key}`}
						onclick={(event) => {
							event.preventDefault();
							void focusField(problem.key, problem.section);
						}}>{problem.message}</a
					>{/each}
			{:else}<CircleCheckIcon size={15} /><span>{t('settings.ready')}</span>{/if}
		</div>
		<div class="file-actions">
			<button class="button sm ghost" onclick={importFromFile}><UploadIcon size={14} />{t('settings.import')}</button>
			<button class="button sm ghost" onclick={exportToFile}><DownloadIcon size={14} />{t('settings.export')}</button>
		</div>
	</div>
	{#if importMessage}<p class="import-message" role="status">{importMessage}</p>{/if}
	{#if saveError}<ErrorNotice error={saveError} context="settings" />{/if}

	<div class="settings-body">
		<nav class="section-nav" aria-label={t('settings.sections')}>
			{#each SECTIONS as section (section.id)}
				<a
					href={`#settings-${section.id}`}
					aria-current={activeSection === section.id ? 'true' : undefined}
					data-state={sectionState(section.id)}
					onclick={(event) => {
						event.preventDefault();
						goToSection(section.id);
					}}>{sectionLabel(section.id)}{#if sectionState(section.id)}<span class="marker" aria-hidden="true"></span>{/if}</a
				>
			{/each}
		</nav>
		<select
			class="input section-select"
			aria-label={t('settings.section')}
			value={activeSection}
			onchange={(event) => goToSection(event.currentTarget.value)}
		>
			{#each SECTIONS as section (section.id)}<option value={section.id}>{sectionLabel(section.id)}</option>{/each}
		</select>

		<div class="settings-scroll" tabindex="-1" bind:this={scroller} onscroll={updateActive}>
			{#if searching && !matches.size}
				<p class="no-match">{t('settings.noMatch', { query: query.trim() })}</p>
			{/if}
			{#each SECTIONS as section (section.id)}
				{@const fields = fieldsIn(section.id, settingsMap)}
				{@const advanced = fields.filter((f) => f.advanced && visible(f))}
				<section class="settings-section" id={`settings-${section.id}`} hidden={!sectionVisible(section.id) && searching}>
					<div class="section-head">
						<div>
							<h2>{sectionLabel(section.id)}</h2>
							<p class="section-description">{sectionDescription(section.id)}</p>
						</div>
						<div class="section-actions">
							{#if section.id === 'llm' || section.id === 'search'}
								<button
									class="button sm"
									disabled={checks[section.id]?.running}
									onclick={() =>
										runCheck(section.id, () =>
											section.id === 'llm' ? testLlmConnection(formValues()) : testSearchConnection(formValues())
										)}><PlugIcon size={14} />{t('settings.testConnection')}</button
								>
							{/if}
							{#if Object.keys(sectionResetValues(section.id, settingsMap)).length}
								<button class="button sm ghost" onclick={() => resetSection(section.id)}
									><RotateCcwIcon size={14} />{t('settings.resetSection')}</button
								>
							{/if}
						</div>
					</div>
					{#if section.id === 'search'}<p class="note">{t('settings.searchCost')}</p>{/if}
					{@render checkResult(section.id)}

					{#each section.groups as group}
						{@const groupFields = fields.filter((f) => f.group === group && !f.advanced && visible(f))}
						{@const custom =
							!searching &&
							((section.id === 'llm' && group === 'Pricing') ||
								section.id === 'network' ||
								(section.id === 'app' && (group === 'Appearance' || group === 'Files')))}
						{#if groupFields.length || custom}
							<div class="group">
								<h3>{groupLabel(group)}</h3>
								{#each groupFields as field (field.key)}{@render fieldView(field)}{/each}
								{#if custom && section.id === 'llm'}
									<ModelPricing
										provider={provider}
										endpoint={activeEndpoint}
										model={activeModel}
										value={settingsMap.get('llm_pricing_overrides') ?? '{}'}
										onchange={(next) => handleChange('llm_pricing_overrides', next)}
									/>
								{:else if custom && section.id === 'app' && group === 'Appearance'}
									<div class="theme-row">
										<span class="theme-label" id="theme-label">{t('settings.theme')}</span>
										<div class="theme-options" role="radiogroup" aria-labelledby="theme-label" aria-label={t('settings.theme')}>
											{#each themes as theme (theme.value)}
												{@const Icon = theme.icon}
												<label class:checked={$themePreference === theme.value}
													><input
														type="radio"
														name="theme"
														value={theme.value}
														checked={$themePreference === theme.value}
														onchange={() => changeTheme(theme.value)}
													/><Icon size={15} />{t(theme.label)}</label
												>
											{/each}
										</div>
										<p class="theme-hint">{t('theme.pairHint')}</p>
									</div>
									<ThemePicker mode="light" selected={$lightTheme} onselect={chooseTheme} />
									<ThemePicker mode="dark" selected={$darkTheme} onselect={chooseTheme} />
									<div class="theme-row">
										<span class="theme-label" id="scale-label">{t('theme.scale')}</span>
										<div class="theme-options" role="radiogroup" aria-labelledby="scale-label">
											{#each UI_SCALES as scale (scale)}
												<label class:checked={$uiScale === scale}
													><input
														type="radio"
														name="ui-scale"
														value={scale}
														checked={$uiScale === scale}
														onchange={() => changeScale(scale)}
													/>{t(`theme.scale.${scale}`)}</label
												>
											{/each}
										</div>
									</div>
									<div class="theme-row">
										<label class="theme-label" for="ui-language">{t('settings.language')}</label>
										<select
											id="ui-language"
											class="input language"
											value={language.preference}
											onchange={(event) => changeLanguage(event.currentTarget.value as LanguagePreference)}
										>
											<option value="system">{t('settings.languageSystem')}</option>
											<option value="en">English</option>
											<option value="ru">Русский</option>
										</select>
									</div>
								{:else if custom && section.id === 'app' && group === 'Files'}
									<AppFiles />
								{:else if custom && section.id === 'network'}
									<p class="section-description">
										{t('settings.proxyHelp')}
									</p>
									<div class="proxy-list">
										<label class="proxy-radio"
											><input
												type="radio"
												name="active-proxy"
												checked={activeProxy === ''}
												onchange={() => handleChange('active_proxy_url', '')}
											/>{t('settings.directConnection')}</label
										>
										{#each proxies as proxy, i (i)}
											<div class="proxy-row">
												<input
													type="radio"
													name="active-proxy"
													checked={proxy.url !== '' && activeProxy === proxy.url}
													disabled={proxy.url === ''}
													onchange={() => handleChange('active_proxy_url', proxy.url)}
													aria-label={t('settings.useProxy')}
												/>
												<input
													class="input proxy-name"
													aria-label={t('settings.proxyName')}
													placeholder={t('settings.proxyNamePlaceholder')}
													value={proxy.name}
													oninput={(e) => updateProxy(i, 'name', e.currentTarget.value)}
												/>
												<input
													class="input proxy-url"
													aria-label={t('settings.proxyUrl')}
													placeholder="http://user:pass@host:port"
													spellcheck="false"
													value={proxyFocus === i ? proxy.url : maskProxyUrl(proxy.url)}
													onfocus={() => (proxyFocus = i)}
													onblur={() => (proxyFocus = null)}
													oninput={(e) => updateProxy(i, 'url', e.currentTarget.value)}
												/>
												<button
													class="button sm"
													type="button"
													disabled={!proxy.url || checks[`proxy-${i}`]?.running}
													onclick={() => runCheck(`proxy-${i}`, () => testProxy(proxy.url))}>{t('settings.testProxy')}</button
												>
												<button
													class="icon-button ghost danger"
													type="button"
													onclick={() => removeProxy(i)}
													aria-label={t('settings.removeProxy')}><TrashIcon size={16} /></button
												>
											</div>
											{@render checkResult(`proxy-${i}`)}
										{/each}
										<button
											class="button dashed add-proxy"
											type="button"
											onclick={() => persistProxies([...proxies, { name: '', url: '' }])}
											><PlusIcon size={16} />{t('settings.addProxy')}</button
										>
									</div>
								{/if}
							</div>
						{/if}
					{/each}

					{#if fields.some((f) => f.advanced) && (!searching || advanced.length)}
						{@const shown = advancedShown(section.id)}
						<div class="advanced">
							{#if !searching}<button
									class="advanced-toggle"
									aria-expanded={shown}
									onclick={() => toggleAdvanced(section.id)}
									>{#if shown}<ChevronDownIcon size={15} />{:else}<ChevronRightIcon size={15} />{/if}{t('settings.advanced')}
									<span class="muted">({fields.filter((f) => f.advanced).length})</span>
									{#if fields.some((f) => f.advanced && !isDefault(f, value(f)) && !f.keepOnReset)}<span class="muted"
											>{t('settings.changedFromDefaults')}</span
										>{/if}</button
								>{/if}
							{#if shown}
								{#each advanced as field (field.key)}{@render fieldView(field)}{/each}
							{/if}
						</div>
					{/if}
				</section>
			{/each}
		</div>
	</div>
</div>

{#if leaveAction}
	<Dialog title={t('settings.leaveTitle')} busy={saving} onclose={() => (leaveAction = null)}>
		<p>{t('settings.leaveText')}</p>
		{#if saveError}<ErrorNotice error={saveError} context="settings" />{/if}
		{#snippet footer()}
			<button class="button" disabled={saving} onclick={() => (leaveAction = null)}>{t('settings.stay')}</button>
			<button class="button" disabled={saving} onclick={() => leave(false)}>{t('settings.discard')}</button>
			<button class="button primary" disabled={saving || blocked} onclick={() => leave(true)}>{t('settings.saveAndLeave')}</button>
		{/snippet}
	</Dialog>
{/if}

<style>
	.settings-page {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		min-width: 0;
		overflow: hidden;
		container: settings-page / inline-size;
	}
	.settings-header {
		align-items: center;
		padding-bottom: 8px;
	}
	.save-actions {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: 8px;
	}
	.save-actions span {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 16px;
		flex-shrink: 0;
		padding-bottom: 10px;
	}
	.search {
		display: flex;
		align-items: center;
		gap: 6px;
		flex: 0 1 280px;
		color: var(--app-muted);
	}
	.search input {
		flex: 1;
		min-width: 0;
	}
	.readiness {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 4px 10px;
		flex: 1 1 260px;
		font-size: var(--app-text-sm);
	}
	.readiness :global(svg) {
		flex-shrink: 0;
	}
	.readiness a {
		color: var(--app-warning);
		text-decoration: underline;
	}
	.readiness:has(a) :global(svg) {
		color: var(--app-warning);
	}
	.readiness:not(:has(a)) :global(svg) {
		color: var(--app-success);
	}
	.file-actions {
		display: flex;
		gap: 4px;
		margin-left: auto;
	}
	.import-message {
		flex-shrink: 0;
		margin-bottom: 8px;
		color: var(--app-accent);
		font-size: var(--app-text-sm);
	}
	.settings-body {
		display: grid;
		grid-template-columns: 160px minmax(0, 1fr);
		gap: 0 20px;
		flex: 1;
		min-height: 0;
	}
	.section-nav {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding-top: 4px;
	}
	.section-nav a {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 10px;
		border-radius: var(--app-radius-sm);
		color: var(--app-muted);
		text-decoration: none;
		font-weight: 500;
	}
	.section-nav a:hover {
		color: var(--app-text);
		background: var(--app-subtle);
	}
	.section-nav a[aria-current='true'] {
		color: var(--app-accent);
		background: color-mix(in srgb, var(--app-accent) 12%, transparent);
	}
	.marker {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--app-accent);
	}
	.section-nav a[data-state='error'] .marker {
		background: var(--app-danger);
	}
	.section-select {
		display: none;
	}
	@container settings-page (max-width: 760px) {
		.settings-body {
			grid-template-columns: minmax(0, 1fr);
			grid-template-rows: auto minmax(0, 1fr);
			gap: 8px;
		}
		.section-nav {
			display: none;
		}
		.section-select {
			display: block;
		}
	}
	.settings-scroll {
		min-height: 0;
		overflow: auto;
		scrollbar-gutter: stable;
		padding: 0 12px 24px 0;
		container: settings / inline-size;
	}
	.settings-section {
		margin-bottom: 20px;
		padding: 18px 20px 8px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius-lg);
		background: var(--app-panel);
		scroll-margin-top: 4px;
	}
	.section-head {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: flex-start;
		gap: 8px 16px;
	}
	.section-actions {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	h2 {
		font-size: var(--app-text-xl);
		font-weight: 650;
		margin: 0;
	}
	.section-description {
		margin-top: 4px;
		color: var(--app-muted);
		font-size: var(--app-text-md);
	}

	.note {
		margin-top: 6px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.check {
		display: flex;
		align-items: flex-start;
		gap: 6px;
		margin-top: 8px;
		font-size: var(--app-text-md);
		overflow-wrap: anywhere;
	}
	.check :global(svg) {
		flex-shrink: 0;
		margin-top: 2px;
	}
	.check.ok {
		color: var(--app-success);
	}
	.check.warn,
	.check.fail {
		color: var(--app-warning);
	}
	.check .raw {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.group {
		margin-top: 16px;
	}
	h3 {
		margin: 0 0 2px;
		color: var(--app-muted);
		font-size: var(--app-text-xs);
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
	}
	.help {
		margin-top: 6px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		line-height: 1.45;
	}
	.advanced {
		margin-top: 12px;
		border-top: 1px solid var(--app-border);
		padding: 8px 0;
	}
	.advanced-toggle {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 0;
		font-weight: 600;
	}
	.muted {
		color: var(--app-muted);
		font-weight: 400;
	}
	.no-match {
		padding: 24px;
		color: var(--app-muted);
		text-align: center;
	}
	.theme-row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 24px;
		padding: 12px 0;
		border-top: 1px solid var(--app-border);
	}
	.theme-label {
		font-weight: 600;
	}
	.theme-hint {
		flex-basis: 100%;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	select.language {
		width: auto;
		min-width: 180px;
	}
	.theme-options {
		display: flex;
		gap: 4px;
	}
	.theme-options label {
		position: relative;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 12px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius-pill);
		cursor: pointer;
	}
	.theme-options label.checked {
		border-color: var(--app-accent);
		color: var(--app-accent);
	}
	/* The radio covers its label invisibly, so the whole pill is the control. */
	.theme-options input {
		position: absolute;
		inset: 0;
		margin: 0;
		opacity: 0;
		cursor: pointer;
	}
	.theme-options label:has(input:focus-visible) {
		outline: 2px solid var(--app-accent);
		outline-offset: 2px;
	}
	.proxy-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 12px 0;
	}
	.proxy-radio {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.proxy-row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}
	.proxy-name {
		flex: 0 1 180px;
		min-width: 0;
	}
	.proxy-url {
		flex: 1 1 260px;
		min-width: 0;
	}
	.add-proxy {
		align-self: flex-start;
	}
	.settings-section :global(.app-files) {
		margin: 0;
		padding: 0;
		border: 0;
		background: transparent;
	}
	.settings-section :global(.app-files > h2) {
		display: none;
	}
</style>
