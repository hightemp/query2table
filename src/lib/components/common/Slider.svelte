<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	let {
		value,
		min = 0,
		max = 100,
		class: className = '',
		oninput,
		...rest
	}: {
		value: string | number;
		min?: number;
		max?: number;
		class?: string;
	} & Omit<HTMLInputAttributes, 'type' | 'value' | 'min' | 'max' | 'class'> = $props();

	let live = $state<number | null>(null);
	const fill = $derived.by(() => {
		const n = live ?? Number(value);
		if (!Number.isFinite(n) || max <= min) return 0;
		return Math.round(((Math.min(max, Math.max(min, n)) - min) / (max - min)) * 1000) / 10;
	});
	$effect(() => {
		void value;
		live = null;
	});
</script>

<input
	{...rest}
	type="range"
	class="slider {className}"
	{min}
	{max}
	{value}
	style:--fill={`${fill}%`}
	oninput={(event) => {
		live = Number(event.currentTarget.value);
		oninput?.(event);
	}}
/>

<style>
	.slider {
		--track: 4px;
		--thumb: 16px;
		appearance: none;
		-webkit-appearance: none;
		height: var(--thumb);
		margin: 0;
		background: transparent;
		cursor: pointer;
	}
	.slider::-webkit-slider-runnable-track {
		height: var(--track);
		border-radius: 999px;
		background: linear-gradient(
			to right,
			var(--app-accent-solid) var(--fill),
			var(--app-border) var(--fill)
		);
	}
	.slider::-moz-range-track {
		height: var(--track);
		border-radius: 999px;
		background: var(--app-border);
	}
	.slider::-moz-range-progress {
		height: var(--track);
		border-radius: 999px;
		background: var(--app-accent-solid);
	}
	.slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		width: var(--thumb);
		height: var(--thumb);
		margin-top: calc((var(--track) - var(--thumb)) / 2);
		border: 2px solid var(--app-accent-solid);
		border-radius: 50%;
		background: var(--app-panel);
		box-shadow: 0 1px 3px rgb(0 0 0 / 25%);
		transition: transform 0.1s;
	}
	.slider::-moz-range-thumb {
		width: var(--thumb);
		height: var(--thumb);
		border: 2px solid var(--app-accent-solid);
		border-radius: 50%;
		background: var(--app-panel);
		box-sizing: border-box;
	}
	.slider:hover::-webkit-slider-thumb,
	.slider:active::-webkit-slider-thumb {
		transform: scale(1.15);
	}
	.slider:focus-visible {
		outline: none;
	}
	.slider:focus-visible::-webkit-slider-thumb {
		outline: 2px solid var(--app-accent);
		outline-offset: 2px;
	}
	.slider:focus-visible::-moz-range-thumb {
		outline: 2px solid var(--app-accent);
		outline-offset: 2px;
	}
</style>
