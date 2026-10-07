<script lang="ts">
	import { onDestroy } from 'svelte';
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { MinusIcon, PlusIcon } from '@lucide/svelte';
	import { t } from '$lib/i18n';

	let {
		value = '',
		min,
		max,
		step,
		size = 'md',
		class: className = '',
		disabled = false,
		...rest
	}: {
		value?: string | number | null;
		min?: string | number;
		max?: string | number;
		step?: string | number;
		size?: 'sm' | 'md';
		/** Classes for the whole control (width and layout). */
		class?: string;
		disabled?: boolean;
	} & Omit<HTMLInputAttributes, 'type' | 'value' | 'min' | 'max' | 'step' | 'class' | 'size'> = $props();

	let input: HTMLInputElement;
	let current = $state('');
	$effect(() => {
		current = String(value ?? '');
	});

	const lower = $derived(min === undefined || min === '' ? -Infinity : Number(min));
	const upper = $derived(max === undefined || max === '' ? Infinity : Number(max));
	const increment = $derived(step === undefined || step === 'any' || !(Number(step) > 0) ? 1 : Number(step));
	const decimals = $derived(Math.max(decimalsOf(increment), decimalsOf(lower)));
	const number = $derived(current.trim() === '' ? NaN : Number(current));
	const atMin = $derived(Number.isFinite(number) && number <= lower);
	const atMax = $derived(Number.isFinite(number) && number >= upper);

	function decimalsOf(n: number) {
		if (!Number.isFinite(n)) return 0;
		return (String(n).split('.')[1] ?? '').length;
	}

	function stepBy(direction: 1 | -1) {
		let next: number;
		if (!Number.isFinite(number)) next = Number.isFinite(lower) ? lower : 0;
		else next = number + direction * increment;
		next = Math.min(upper, Math.max(lower, Number(next.toFixed(decimals))));
		if (next === number) return false;
		input.value = String(next);
		current = input.value;
		// Report the change the same way typing does, so callers keep using `oninput`.
		input.dispatchEvent(new Event('input', { bubbles: true }));
		return true;
	}

	let delay: ReturnType<typeof setTimeout> | undefined;
	let repeat: ReturnType<typeof setInterval> | undefined;
	function stop() {
		clearTimeout(delay);
		clearInterval(repeat);
		delay = repeat = undefined;
	}
	function press(event: PointerEvent, direction: 1 | -1) {
		if (event.button !== 0) return;
		event.preventDefault();
		if (!stepBy(direction)) return;
		stop();
		delay = setTimeout(() => {
			repeat = setInterval(() => {
				if (!stepBy(direction)) stop();
			}, 60);
		}, 400);
	}
	onDestroy(stop);
</script>

<!-- The input comes first so a wrapping <label> names it, not a button; CSS puts − on the left. -->
<div class="number-input {size} {className}" class:disabled>
	<input
		{...rest}
		bind:this={input}
		type="number"
		{min}
		{max}
		{step}
		{disabled}
		value={value ?? ''}
		oninput={(event) => {
			current = event.currentTarget.value;
			rest.oninput?.(event);
		}}
	/>
	<button
		type="button"
		class="step decrease"
		tabindex="-1"
		aria-label={t('number.decrease')}
		disabled={disabled || atMin}
		onpointerdown={(event) => press(event, -1)}
		onpointerup={stop}
		onpointerleave={stop}
		onpointercancel={stop}><MinusIcon size={size === 'sm' ? 12 : 14} /></button
	>
	<button
		type="button"
		class="step"
		tabindex="-1"
		aria-label={t('number.increase')}
		disabled={disabled || atMax}
		onpointerdown={(event) => press(event, 1)}
		onpointerup={stop}
		onpointerleave={stop}
		onpointercancel={stop}><PlusIcon size={size === 'sm' ? 12 : 14} /></button
	>
</div>

<style>
	.number-input {
		display: flex;
		width: 100%;
		align-items: stretch;
		min-width: 0;
		min-height: 36px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		background: var(--app-input-bg);
		overflow: hidden;
	}
	.number-input:focus-within {
		border-color: var(--app-accent);
	}
	.number-input:has(input:focus-visible) {
		outline: 2px solid var(--app-accent);
		outline-offset: 3px;
	}
	.number-input:has(input[aria-invalid='true']) {
		border-color: var(--app-danger);
	}
	.number-input.disabled {
		opacity: 0.5;
	}
	input {
		flex: 1;
		width: 100%;
		min-width: 0;
		padding: 7px 4px;
		border: 0;
		background: transparent;
		color: var(--app-text);
		font-size: var(--app-text-base);
		line-height: 1.4;
		text-align: center;
		font-variant-numeric: tabular-nums;
		outline: none;
	}
	input:disabled {
		opacity: 1;
	}
	.step {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		width: 30px;
		border: 0;
		background: transparent;
		color: var(--app-muted);
		touch-action: manipulation;
		user-select: none;
	}
	.decrease {
		order: -1;
	}
	.step:hover:not(:disabled) {
		background: var(--app-subtle);
		color: var(--app-text);
	}
	.step:active:not(:disabled) {
		background: color-mix(in srgb, var(--app-accent) 14%, var(--app-subtle));
	}
	.step:disabled {
		opacity: 0.35;
	}
	.sm {
		min-height: 28px;
		border-radius: var(--app-radius-sm);
	}
	.sm input {
		padding: 3px 2px;
		font-size: var(--app-text-sm);
	}
	.sm .step {
		width: 24px;
	}
</style>
