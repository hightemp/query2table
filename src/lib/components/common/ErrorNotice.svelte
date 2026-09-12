<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { LlmIssueCode } from '$lib/types';
	import { presentError, type ErrorContext } from '$lib/utils/errors';

	let { error, context = 'run', code, children, liveRole = 'alert' }: {
		error: unknown;
		context?: ErrorContext;
		code?: LlmIssueCode;
		children?: Snippet;
		liveRole?: 'alert' | 'status';
	} = $props();
	let explanation = $derived(presentError(error, context, code));
	let detailsOpen = $state(false);
	$effect(() => { void error; detailsOpen = false; });
</script>

<div class="error-notice" role={liveRole}>
	<h3>{explanation.title}</h3>
	<p>{explanation.cause}</p>
	{#if children}{@render children()}{/if}
	<p class="action">{explanation.action}</p>
	{#if explanation.settingsHref}<a href={explanation.settingsHref}>Open Settings</a>{/if}
	<details bind:open={detailsOpen}>
		<summary>Technical details</summary>
		<pre>{explanation.details}</pre>
	</details>
</div>

<style>
	.error-notice { flex-shrink: 0; min-width: 0; margin: 8px 0; padding: 12px 14px; border: 1px solid var(--color-error-500); border-radius: 8px; background: color-mix(in srgb, var(--color-error-500) 5%, var(--color-surface-50-950)); font-size: 0.86rem; overflow-wrap: anywhere; }
	h3 { margin: 0 0 5px; font-size: 0.95rem; color: var(--color-error-500); }
	p { margin: 5px 0; line-height: 1.45; }
	.action { font-weight: 500; }
	a { display: inline-block; margin-top: 4px; color: var(--color-primary-500); text-decoration: underline; }
	details { margin-top: 9px; }
	summary { cursor: pointer; color: var(--color-surface-600-400); }
	pre { max-height: 160px; overflow: auto; white-space: pre-wrap; overflow-wrap: anywhere; font: 0.78rem/1.4 monospace; }
</style>
