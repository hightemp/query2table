<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLInputAttributes } from 'svelte/elements';

	let {
		checked = false,
		disabled = false,
		label,
		onchange,
		class: className = '',
		children,
		...rest
	}: {
		checked?: boolean;
		disabled?: boolean;
		/** Accessible name when there is no visible text. */
		label?: string;
		/** Called when this option becomes the chosen one. */
		onchange?: () => void;
		class?: string;
		children?: Snippet;
	} & Omit<HTMLInputAttributes, 'type' | 'checked' | 'onchange' | 'class' | 'children'> = $props();
</script>

<label class="radio {className}" class:disabled>
	<input {...rest} type="radio" {checked} {disabled} aria-label={label} onchange={() => onchange?.()} />
	<span class="dot" aria-hidden="true"></span>
	{#if children}<span class="text">{@render children()}</span>{/if}
</label>

<style>
	.radio {
		position: relative;
		display: inline-flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
		user-select: none;
	}
	.radio.disabled {
		cursor: not-allowed;
		opacity: 0.5;
	}
	/* The real input covers the dot invisibly: it keeps keyboard, focus and screen readers. */
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
	.dot {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		width: 16px;
		height: 16px;
		border: 1.5px solid color-mix(in srgb, var(--app-muted) 70%, var(--app-border));
		border-radius: 50%;
		background: var(--app-panel);
		transition: border-color 0.12s;
	}
	.dot::after {
		content: '';
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--app-accent-solid);
		transform: scale(0);
		transition: transform 0.12s;
	}
	.radio:hover:not(.disabled) .dot,
	input:checked + .dot {
		border-color: var(--app-accent-solid);
	}
	input:checked + .dot::after {
		transform: scale(1);
	}
	input:focus-visible + .dot {
		outline: 2px solid var(--app-accent);
		outline-offset: 2px;
	}
	.text {
		min-width: 0;
	}
	@media (prefers-reduced-motion: reduce) {
		.dot,
		.dot::after {
			transition: none;
		}
	}
</style>
