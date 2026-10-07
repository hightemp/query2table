<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { CheckIcon, MinusIcon } from '@lucide/svelte';

	let {
		checked = $bindable(false),
		indeterminate = false,
		disabled = false,
		label,
		onchange,
		class: className = '',
		children,
		...rest
	}: {
		checked?: boolean;
		/** Drawn as a dash, e.g. for "select all" when only some are selected. */
		indeterminate?: boolean;
		disabled?: boolean;
		/** Accessible name when there is no visible text. */
		label?: string;
		onchange?: (checked: boolean) => void;
		class?: string;
		children?: Snippet;
	} & Omit<HTMLInputAttributes, 'type' | 'checked' | 'onchange' | 'class' | 'children'> = $props();
</script>

<label class="checkbox {className}" class:disabled>
	<input
		{...rest}
		type="checkbox"
		bind:checked
		{indeterminate}
		{disabled}
		aria-label={label}
		onchange={(event) => onchange?.(event.currentTarget.checked)}
	/>
	<span class="box" aria-hidden="true">
		{#if indeterminate}<MinusIcon size={12} strokeWidth={3} />{:else if checked}<CheckIcon size={12} strokeWidth={3} />{/if}
	</span>
	{#if children}<span class="text">{@render children()}</span>{/if}
</label>

<style>
	.checkbox {
		position: relative;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
		user-select: none;
	}
	.checkbox.disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}
	/* The real input covers the box invisibly: it keeps keyboard, focus and screen readers. */
	input {
		position: absolute;
		top: 50%;
		left: 0;
		width: 16px;
		height: 16px;
		margin: -8px 0 0;
		opacity: 0;
		cursor: inherit;
	}
	.box {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		width: 16px;
		height: 16px;
		border: 1.5px solid color-mix(in srgb, var(--app-muted) 70%, var(--app-border));
		border-radius: 4px;
		background: var(--app-panel);
		color: var(--app-on-accent);
		transition:
			background-color 0.12s,
			border-color 0.12s;
	}
	.checkbox:hover:not(.disabled) .box {
		border-color: var(--app-accent);
	}
	input:checked + .box,
	input:indeterminate + .box {
		background: var(--app-accent-solid);
		border-color: var(--app-accent-solid);
	}
	input:focus-visible + .box {
		outline: 2px solid var(--app-accent);
		outline-offset: 2px;
	}
	.text {
		min-width: 0;
	}
</style>
