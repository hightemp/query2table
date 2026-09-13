<script lang="ts">
	import { listOllamaCloudModels, listOpenRouterModels } from '$lib/api/tauri';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	import { errorText } from '$lib/utils/errors';

	let {
		id,
		value,
		provider,
		baseUrl = '',
		apiKey,
		onchange,
	}: {
		id: string;
		value: string;
		provider: 'ollama_cloud' | 'openrouter';
		baseUrl?: string;
		apiKey: string;
		onchange: (model: string) => void;
	} = $props();

	let models = $state<string[]>([]);
	let loading = $state(true);
	let error = $state('');
	let reload = $state(0);
	let open = $state(false);
	let filter = $state('');
	let filtering = $state(false);
	let activeIndex = $state(-1);
	let input: HTMLInputElement;
	let popupStyle = $state('');
	$effect(() => {
		if (!open || !input) return;
		function position() {
			const rect = input.getBoundingClientRect();
			const boundary = input.closest('.settings-scroll')?.getBoundingClientRect();
			if (boundary && (rect.bottom < boundary.top || rect.top > boundary.bottom)) {
				open = false;
				return;
			}
			const below = window.innerHeight - rect.bottom - 12;
			const above = rect.top - 12;
			const upwards = below < 200 && above > below;
			const height = Math.max(40, Math.min(240, upwards ? above : below));
			const width = Math.min(rect.width, window.innerWidth - 24);
			const left = Math.max(12, Math.min(rect.left, window.innerWidth - width - 12));
			popupStyle = `left:${left}px;width:${width}px;max-height:${height}px;${upwards ? `bottom:${window.innerHeight - rect.top + 4}px;top:auto` : `top:${rect.bottom + 4}px;bottom:auto`}`;
		}
		position();
		window.addEventListener('resize', position);
		window.addEventListener('scroll', position, true);
		return () => {
			window.removeEventListener('resize', position);
			window.removeEventListener('scroll', position, true);
		};
	});
	const listId = $derived(`${id}-options`);
	const providerLabel = $derived(provider === 'openrouter' ? 'OpenRouter' : 'Ollama Cloud');
	const filtered = $derived(
		models.filter(
			(model) => !filtering || model.toLowerCase().includes(filter.trim().toLowerCase())
		)
	);

	$effect(() => {
		const backend = provider;
		const url = baseUrl.trim();
		const key = apiKey.trim();
		void reload;
		let current = true;
		models = [];
		loading = true;
		error = '';
		activeIndex = -1;
		// Debounce edits and discard responses for an old URL/key or an unmounted picker.
		const timer = setTimeout(async () => {
			try {
				const result =
					backend === 'openrouter'
						? await listOpenRouterModels(key)
						: await listOllamaCloudModels(url, key);
				if (current) models = result;
			} catch (e) {
				if (current) error = errorText(e);
			} finally {
				if (current) loading = false;
			}
		}, 300);
		return () => {
			current = false;
			clearTimeout(timer);
		};
	});

	function openList() {
		if (open) return;
		filter = value;
		filtering = false;
		activeIndex = -1;
		open = true;
	}

	function select(model: string) {
		onchange(model);
		open = false;
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			openList();
			if (!filtered.length) return;
			const direction = event.key === 'ArrowDown' ? 1 : -1;
			activeIndex =
				activeIndex < 0
					? direction > 0
						? 0
						: filtered.length - 1
					: (activeIndex + direction + filtered.length) % filtered.length;
			document.getElementById(`${listId}-${activeIndex}`)?.scrollIntoView?.({ block: 'nearest' });
		} else if (event.key === 'Enter' && open) {
			event.preventDefault();
			const model = filtered[activeIndex] ?? (filtering ? filtered[0] : undefined);
			if (model) select(model);
		} else if (event.key === 'Escape') {
			event.preventDefault();
			open = false;
		}
	}
</script>

<div class="model-picker">
	<div class="controls">
		<div class="combobox">
			<input
				bind:this={input}
				{id}
				role="combobox"
				type="text"
				autocomplete="off"
				aria-autocomplete="list"
				aria-expanded={open}
				aria-controls={listId}
				aria-activedescendant={open && filtered[activeIndex]
					? `${listId}-${activeIndex}`
					: undefined}
				aria-describedby={`${id}-status`}
				value={open ? filter : value}
				placeholder="Type to filter models…"
				onfocus={openList}
				onclick={openList}
				onblur={() => {
					open = false;
				}}
				oninput={(event) => {
					filter = event.currentTarget.value;
					filtering = true;
					activeIndex = -1;
					open = true;
				}}
				onkeydown={handleKeydown}
			/>
			{#if open}
				<div class="dropdown" style={popupStyle}>
					{#if loading}<p>Loading models…</p>{:else if error}<p>
							Could not load models. Close this list and use Refresh to retry.
						</p>{:else if models.length === 0}<p>No models available.</p>{/if}
					<div
						id={listId}
						role="listbox"
						aria-label={`${providerLabel} models`}
						aria-busy={loading}
					>
						{#each filtered as model, index (model)}
							<button
								id={`${listId}-${index}`}
								type="button"
								role="option"
								tabindex="-1"
								aria-selected={model === value}
								class:active={activeIndex === index}
								onmousedown={(event) => event.preventDefault()}
								onclick={() => select(model)}>{model}</button
							>
						{/each}
					</div>
					{#if !loading && !error && models.length > 0 && filtered.length === 0}
						<p>No matching models</p>
					{/if}
				</div>
			{/if}
		</div>
		<button
			class="reload"
			type="button"
			onclick={() => {
				reload += 1;
			}}
			disabled={loading}
		>
			Refresh
		</button>
	</div>
	<p id={`${id}-status`} role="status">
		{#if loading}Loading models…
		{:else if error}The model list could not be loaded.
		{:else if models.length === 0}The server returned no models.
		{:else}{models.length} models available. Type to filter, then select a model.
		{/if}
	</p>
	{#if error}<ErrorNotice {error} context="catalog" />{/if}
</div>

<style>
	.model-picker,
	.combobox {
		min-width: 0;
	}
	.controls {
		display: flex;
		gap: 8px;
	}
	.combobox {
		position: relative;
		flex: 1;
	}
	input,
	.reload {
		padding: 8px 12px;
		border: 1px solid var(--app-border);
		border-radius: 6px;
		background: var(--app-subtle);
		color: inherit;
		font-size: 0.95rem;
	}
	input {
		width: 100%;
		box-sizing: border-box;
	}
	input:focus {
		outline: none;
		border-color: var(--color-primary-500);
	}
	.reload {
		cursor: pointer;
	}
	.reload:disabled {
		opacity: 0.5;
		cursor: wait;
	}
	.dropdown {
		position: fixed;
		z-index: 50;
		max-height: 240px;
		overflow-y: auto;
		border: 1px solid var(--app-border);
		border-radius: 6px;
		background: var(--app-panel);
		box-shadow: 0 4px 12px rgb(0 0 0 / 15%);
	}
	[role='option'] {
		display: block;
		width: 100%;
		padding: 8px 12px;
		border: none;
		background: transparent;
		color: inherit;
		text-align: left;
		overflow-wrap: anywhere;
		cursor: pointer;
	}
	[role='option']:hover,
	[role='option'].active {
		background: var(--app-subtle);
	}
	[aria-selected='true'] {
		font-weight: 600;
		color: var(--color-primary-500);
	}
	p {
		margin: 6px 0 0;
		font-size: 0.82rem;
		color: var(--app-muted);
		overflow-wrap: anywhere;
	}
	.dropdown p {
		padding: 8px 12px;
		margin: 0;
	}
</style>
