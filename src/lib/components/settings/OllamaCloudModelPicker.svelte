<script lang="ts">
	import { listOllamaCloudModels } from '$lib/api/tauri';

	let { id, value, baseUrl, apiKey, onchange }: {
		id: string;
		value: string;
		baseUrl: string;
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
	const listId = $derived(`${id}-options`);
	const filtered = $derived(models.filter((model) =>
		!filtering || model.toLowerCase().includes(filter.trim().toLowerCase())
	));

	$effect(() => {
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
				const result = await listOllamaCloudModels(url, key);
				if (current) models = result;
			} catch (e) {
				if (current) error = `Could not load models: ${String(e)}`;
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
			activeIndex = activeIndex < 0
				? (direction > 0 ? 0 : filtered.length - 1)
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
				{id}
				role="combobox"
				type="text"
				autocomplete="off"
				aria-autocomplete="list"
				aria-expanded={open}
				aria-controls={listId}
				aria-activedescendant={open && filtered[activeIndex] ? `${listId}-${activeIndex}` : undefined}
				aria-describedby={`${id}-status`}
				value={open ? filter : value}
				placeholder="Type to filter models…"
				onfocus={openList}
				onclick={openList}
				onblur={() => { open = false; }}
				oninput={(event) => {
					filter = event.currentTarget.value;
					filtering = true;
					activeIndex = -1;
					open = true;
				}}
				onkeydown={handleKeydown}
			/>
			{#if open}
				<div class="dropdown">
					<div id={listId} role="listbox" aria-label="Ollama Cloud models" aria-busy={loading}>
						{#each filtered as model, index (model)}
							<button
								id={`${listId}-${index}`}
								type="button"
								role="option"
								tabindex="-1"
								aria-selected={model === value}
								class:active={activeIndex === index}
								onmousedown={(event) => event.preventDefault()}
								onclick={() => select(model)}
							>{model}</button>
						{/each}
					</div>
					{#if !loading && !error && models.length > 0 && filtered.length === 0}
						<p>No matching models</p>
					{/if}
				</div>
			{/if}
		</div>
		<button class="reload" type="button" onclick={() => { reload += 1; }} disabled={loading}>
			Refresh
		</button>
	</div>
	<p id={`${id}-status`} class:error role="status">
		{#if loading}Loading models…
		{:else if error}{error}
		{:else if models.length === 0}The server returned no models.
		{:else}{models.length} models available. Type to filter, then select a model.
		{/if}
	</p>
</div>

<style>
	.model-picker, .combobox { min-width: 0; }
	.controls { display: flex; gap: 8px; }
	.combobox { position: relative; flex: 1; }
	input, .reload {
		padding: 8px 12px;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 6px;
		background: var(--color-surface-200-800);
		color: inherit;
		font-size: 0.95rem;
	}
	input { width: 100%; box-sizing: border-box; }
	input:focus { outline: none; border-color: var(--color-primary-500); }
	.reload { cursor: pointer; }
	.reload:disabled { opacity: 0.5; cursor: wait; }
	.dropdown {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		right: 0;
		z-index: 20;
		max-height: 240px;
		overflow-y: auto;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 6px;
		background: var(--color-surface-100-900);
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
	[role='option']:hover, [role='option'].active { background: var(--color-surface-200-800); }
	[aria-selected='true'] { font-weight: 600; color: var(--color-primary-500); }
	p { margin: 6px 0 0; font-size: 0.82rem; color: var(--color-surface-600-400); overflow-wrap: anywhere; }
	.dropdown p { padding: 8px 12px; margin: 0; }
	p.error { color: var(--color-error-500); }
</style>
