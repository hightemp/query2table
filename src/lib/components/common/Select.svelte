<script lang="ts" module>
	export interface SelectOption<T> {
		value: T;
		label: string;
		disabled?: boolean;
	}

	/** Lists this long get a search field. */
	export const SEARCH_FROM = 8;
	let nextId = 0;
</script>

<script lang="ts" generics="T extends string | number">
	import { tick } from 'svelte';
	import { CheckIcon, ChevronDownIcon } from '@lucide/svelte';
	import { t } from '$lib/i18n';

	let {
		value = $bindable(),
		options,
		onchange,
		label,
		id = `select-${++nextId}`,
		size = 'md',
		disabled = false,
		class: className = '',
		'aria-describedby': describedBy,
	}: {
		value: T;
		options: SelectOption<T>[];
		onchange?: (value: T) => void;
		/** Accessible name, when no `<label for={id}>` names the control. */
		label?: string;
		id?: string;
		size?: 'sm' | 'md';
		disabled?: boolean;
		class?: string;
		'aria-describedby'?: string;
	} = $props();

	let open = $state(false);
	let query = $state('');
	let activeIndex = $state(-1);
	let trigger: HTMLButtonElement;
	let search = $state<HTMLInputElement>();
	let popupStyle = $state('');
	let typed = '';
	let typedAt = 0;

	const listId = $derived(`${id}-listbox`);
	const searchable = $derived(options.length >= SEARCH_FROM);
	const selected = $derived(options.find((option) => option.value === value));
	const shown = $derived.by(() => {
		const needle = query.trim().toLocaleLowerCase();
		return needle ? options.filter((option) => option.label.toLocaleLowerCase().includes(needle)) : options;
	});
	const activeId = $derived(open && activeIndex >= 0 && shown[activeIndex] ? `${id}-option-${activeIndex}` : undefined);

	function enabledFrom(start: number, step: 1 | -1): number {
		for (let i = start; i >= 0 && i < shown.length; i += step) if (!shown[i].disabled) return i;
		return -1;
	}

	async function show() {
		if (disabled || open) return;
		query = '';
		const current = shown.findIndex((option) => option.value === value);
		activeIndex = current >= 0 ? current : enabledFrom(0, 1);
		open = true;
		await tick();
		if (searchable) search?.focus();
		scrollActive();
	}

	function hide(restoreFocus = true) {
		if (!open) return;
		open = false;
		if (restoreFocus) trigger?.focus();
	}

	function choose(index: number) {
		const option = shown[index];
		if (!option || option.disabled) return;
		hide();
		if (option.value === value) return;
		value = option.value;
		onchange?.(option.value);
	}

	function move(index: number) {
		if (index < 0) return;
		activeIndex = index;
		void tick().then(scrollActive);
	}

	function scrollActive() {
		if (activeId) document.getElementById(activeId)?.scrollIntoView?.({ block: 'nearest' });
	}

	/** Jumps to the next option starting with the typed letters, like a native select. */
	function typeAhead(key: string) {
		const now = Date.now();
		typed = now - typedAt < 600 ? typed + key.toLocaleLowerCase() : key.toLocaleLowerCase();
		typedAt = now;
		const start = typed.length === 1 ? activeIndex + 1 : Math.max(activeIndex, 0);
		for (let n = 0; n < shown.length; n++) {
			const i = (start + n) % shown.length;
			if (!shown[i].disabled && shown[i].label.toLocaleLowerCase().startsWith(typed)) return move(i);
		}
	}

	function handleKey(event: KeyboardEvent) {
		const fromSearch = event.currentTarget === search;
		if (!open) {
			if (['ArrowDown', 'ArrowUp', 'Enter', ' '].includes(event.key)) {
				event.preventDefault();
				void show();
			} else if (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey && !searchable) {
				void show().then(() => typeAhead(event.key));
			}
			return;
		}
		switch (event.key) {
			case 'ArrowDown':
				event.preventDefault();
				move(enabledFrom(activeIndex + 1, 1));
				break;
			case 'ArrowUp':
				event.preventDefault();
				move(enabledFrom(activeIndex - 1, -1));
				break;
			case 'Home':
			case 'End':
				if (fromSearch) return;
				event.preventDefault();
				move(event.key === 'Home' ? enabledFrom(0, 1) : enabledFrom(shown.length - 1, -1));
				break;
			case 'PageDown':
			case 'PageUp':
				event.preventDefault();
				move(
					event.key === 'PageDown'
						? (enabledFrom(Math.min(activeIndex + 8, shown.length - 1), -1) )
						: enabledFrom(Math.max(activeIndex - 8, 0), 1)
				);
				break;
			case 'Enter':
				event.preventDefault();
				choose(activeIndex);
				break;
			case ' ':
				if (fromSearch) return;
				event.preventDefault();
				choose(activeIndex);
				break;
			case 'Escape':
				event.preventDefault();
				event.stopPropagation();
				hide();
				break;
			case 'Tab':
				hide(false);
				break;
			default:
				if (!fromSearch && event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey)
					typeAhead(event.key);
		}
	}

	$effect(() => {
		if (!open || !trigger) return;
		const popup = document.getElementById(listId)?.parentElement;
		function position() {
			const rect = trigger.getBoundingClientRect();
			const below = window.innerHeight - rect.bottom - 12;
			const above = rect.top - 12;
			const upwards = below < 220 && above > below;
			const height = Math.max(80, Math.min(320, upwards ? above : below));
			const width = Math.min(Math.max(rect.width, 160), window.innerWidth - 24);
			const left = Math.max(12, Math.min(rect.left, window.innerWidth - width - 12));
			popupStyle = `left:${left}px;min-width:${width}px;max-width:${Math.max(width, Math.min(420, window.innerWidth - left - 12))}px;max-height:${height}px;${upwards ? `bottom:${window.innerHeight - rect.top + 4}px` : `top:${rect.bottom + 4}px`}`;
		}
		function outside(event: Event) {
			const target = event.target as Node;
			if (!trigger.contains(target) && !popup?.contains(target)) hide(false);
		}
		function scrolled(event: Event) {
			if (popup?.contains(event.target as Node)) return;
			position();
		}
		const blur = () => hide(false);
		position();
		window.addEventListener('pointerdown', outside, true);
		window.addEventListener('resize', position);
		window.addEventListener('scroll', scrolled, true);
		window.addEventListener('blur', blur);
		return () => {
			window.removeEventListener('pointerdown', outside, true);
			window.removeEventListener('resize', position);
			window.removeEventListener('scroll', scrolled, true);
			window.removeEventListener('blur', blur);
		};
	});

	$effect(() => {
		// Keep a valid active option while the search narrows the list.
		void query;
		if (open && searchable && (activeIndex < 0 || activeIndex >= shown.length || shown[activeIndex]?.disabled))
			activeIndex = enabledFrom(0, 1);
	});
</script>

<button
	bind:this={trigger}
	{id}
	type="button"
	class="input select {size} {className}"
	class:open
	role="combobox"
	data-value={String(value)}
	aria-label={label}
	aria-haspopup="listbox"
	aria-expanded={open}
	aria-controls={open ? listId : undefined}
	aria-activedescendant={searchable ? undefined : activeId}
	aria-describedby={describedBy}
	{disabled}
	onclick={() => (open ? hide() : void show())}
	onkeydown={handleKey}
>
	<span class="value">{selected?.label ?? ''}</span>
	<ChevronDownIcon size={size === 'sm' ? 14 : 16} aria-hidden="true" />
</button>
{#if open}
	<!-- A wrapping <label> would forward clicks inside the list to the trigger and reopen it. -->
	<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
	<div class="select-popup" style={popupStyle} onclick={(event) => event.preventDefault()}>

		{#if searchable}
			<input
				bind:this={search}
				class="input sm search"
				type="search"
				placeholder={t('select.search')}
				aria-label={t('select.search')}
				aria-controls={listId}
				aria-activedescendant={activeId}
				autocomplete="off"
				spellcheck="false"
				bind:value={query}
				onkeydown={handleKey}
			/>
		{/if}
		<ul id={listId} role="listbox" aria-label={label} tabindex="-1">
			{#each shown as option, index (option.value)}
				<!-- svelte-ignore a11y_click_events_have_key_events -- the combobox handles keys -->
				<li
					id={`${id}-option-${index}`}
					role="option"
					data-value={String(option.value)}
					aria-selected={option.value === value}
					aria-disabled={option.disabled || undefined}
					class:active={index === activeIndex}
					onpointermove={() => !option.disabled && (activeIndex = index)}
					onpointerdown={(event) => event.preventDefault()}
					onclick={() => choose(index)}
				>
					<span class="label">{option.label}</span>
					{#if option.value === value}<CheckIcon size={14} aria-hidden="true" />{/if}
				</li>
			{:else}
				<li class="empty" role="presentation">{t('select.nothingFound')}</li>
			{/each}
		</ul>
	</div>
{/if}

<style>
	.select {
		display: inline-flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		width: auto;
		text-align: left;
		cursor: pointer;
	}
	.select:hover:not(:disabled) {
		border-color: color-mix(in srgb, var(--app-muted) 50%, var(--app-border));
	}
	.select.open {
		border-color: var(--app-accent);
	}
	.select :global(svg) {
		color: var(--app-muted);
		transition: transform 0.15s;
	}
	.select.open :global(svg) {
		transform: rotate(180deg);
	}
	.value {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.select-popup {
		position: fixed;
		z-index: 1000;
		display: flex;
		flex-direction: column;
		padding: 4px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		background: var(--app-panel);
		box-shadow: var(--app-shadow-popover);
		animation: select-in 0.12s ease-out;
	}
	@keyframes select-in {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.select-popup {
			animation: none;
		}
	}
	.search {
		flex-shrink: 0;
		margin-bottom: 4px;
	}
	ul {
		overflow-y: auto;
		outline: none;
	}
	li {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 6px 8px;
		border-radius: var(--app-radius-sm);
		font-size: var(--app-text-base);
		cursor: pointer;
		user-select: none;
	}
	li.active {
		background: var(--app-subtle);
	}
	li[aria-selected='true'] {
		color: var(--app-accent);
		font-weight: 600;
	}
	li[aria-disabled='true'] {
		opacity: 0.45;
		cursor: not-allowed;
	}
	li .label {
		min-width: 0;
		overflow-wrap: anywhere;
	}
	li.empty {
		color: var(--app-muted);
		cursor: default;
	}
</style>
