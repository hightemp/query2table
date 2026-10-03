<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { LlmIssueCode } from '$lib/types';
	import { presentError, type ErrorContext } from '$lib/utils/errors';

	let {
		error,
		context = 'run',
		code,
		children,
		liveRole = 'alert',
	}: {
		error: unknown;
		context?: ErrorContext;
		code?: LlmIssueCode;
		children?: Snippet;
		liveRole?: 'alert' | 'status';
	} = $props();
	let explanation = $derived(presentError(error, context, code));
	let detailsOpen = $state(false);
	$effect(() => {
		void error;
		detailsOpen = false;
	});
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
	.error-notice {
		flex-shrink: 0;
		min-width: 0;
		margin: 8px 0;
		padding: 12px 14px;
		border: 1px solid color-mix(in srgb, var(--app-danger) 60%, var(--app-border));
		border-radius: var(--app-radius);
		background: color-mix(in srgb, var(--app-danger) 5%, var(--app-panel));
		font-size: var(--app-text-md);
		overflow-wrap: anywhere;
	}
	h3 {
		margin: 0 0 5px;
		font-size: var(--app-text-base);
		font-weight: 650;
		color: var(--app-danger);
	}
	p {
		margin: 5px 0;
		line-height: 1.45;
	}
	.action {
		font-weight: 500;
	}
	a {
		display: inline-block;
		margin-top: 4px;
		color: var(--app-accent);
		text-decoration: underline;
	}
	details {
		margin-top: 9px;
	}
	summary {
		cursor: pointer;
		color: var(--app-muted);
	}
	pre {
		max-height: 160px;
		overflow: auto;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		font: var(--app-text-xs) / 1.4 var(--app-font-mono);
	}
</style>
