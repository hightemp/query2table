<script lang="ts">
	import type { ResearchStep } from '$lib/types';
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import { SearchIcon, FileTextIcon, BrainIcon, AlertTriangleIcon } from '@lucide/svelte';

	let {
		steps,
		answer,
		running = false
	}: { steps: ResearchStep[]; answer: string | null; running?: boolean } = $props();

	const renderedAnswer = $derived(
		answer ? DOMPurify.sanitize(marked.parse(answer, { async: false }) as string) : ''
	);

	function stepIcon(type: string) {
		if (type === 'search') return SearchIcon;
		if (type === 'fetch') return FileTextIcon;
		if (type === 'error') return AlertTriangleIcon;
		return BrainIcon;
	}

	function stepLabel(type: string): string {
		if (type === 'search') return 'Search';
		if (type === 'fetch') return 'Fetch';
		if (type === 'think') return 'Think';
		if (type === 'error') return 'Error';
		return type;
	}

	function previewContent(step: ResearchStep): string {
		const text = step.content ?? '';
		const limit = step.step_type === 'fetch' ? 240 : 600;
		return text.length > limit ? text.slice(0, limit) + '…' : text;
	}
</script>

<div class="research-view">
	{#if steps.length > 0 || running}
		<section class="steps">
			<h2 class="section-title">Agent steps</h2>
			<ol class="timeline">
				{#each steps as step (step.id)}
					{@const Icon = stepIcon(step.step_type)}
					<li class="step step-{step.step_type}">
						<div class="step-icon"><Icon size={16} /></div>
						<div class="step-body">
							<div class="step-head">
								<span class="step-label">{stepLabel(step.step_type)}</span>
								{#if step.url}
									<a class="step-url" href={step.url} target="_blank" rel="noreferrer">{step.url}</a>
								{/if}
							</div>
							<div class="step-content">{previewContent(step)}</div>
						</div>
					</li>
				{/each}
				{#if running && !answer}
					<li class="step step-working">
						<div class="step-icon"><span class="spinner"></span></div>
						<div class="step-body">
							<div class="working-label">
								The research agent is working<span class="dots"><span>.</span><span>.</span><span>.</span></span>
							</div>
						</div>
					</li>
				{/if}
			</ol>
		</section>
	{/if}

	{#if answer}
		<section class="answer">
			<h2 class="section-title">Answer</h2>
			<!-- eslint-disable-next-line svelte/no-at-html-tags -- sanitized with DOMPurify -->
			<div class="markdown">{@html renderedAnswer}</div>
		</section>
	{:else if steps.length === 0 && !running}
		<div class="research-empty">The research agent is getting started…</div>
	{/if}
</div>

<style>
	.research-view {
		display: flex;
		flex-direction: column;
		gap: 24px;
		overflow-y: auto;
		padding-bottom: 16px;
	}

	.section-title {
		font-size: 0.95rem;
		font-weight: 700;
		margin: 0 0 10px;
		color: var(--color-surface-700-300);
	}

	.timeline {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.step {
		display: flex;
		gap: 10px;
		padding: 10px 12px;
		border: 1px solid var(--color-surface-300-700);
		border-radius: 8px;
		background: var(--color-surface-50-950);
	}

	.step-icon {
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: 6px;
		background: var(--color-surface-200-800);
		color: var(--color-surface-700-300);
	}

	.step-search .step-icon {
		color: var(--color-primary-500);
	}
	.step-fetch .step-icon {
		color: var(--color-tertiary-500);
	}
	.step-think .step-icon {
		color: var(--color-secondary-500);
	}

	.step-error {
		border-color: var(--color-error-500);
		background: color-mix(in srgb, var(--color-error-500) 8%, var(--color-surface-50-950));
	}
	.step-error .step-icon {
		color: var(--color-error-500);
		background: color-mix(in srgb, var(--color-error-500) 18%, transparent);
	}
	.step-error .step-label {
		color: var(--color-error-500);
	}

	/* Animated in-progress indicator */
	.step-working {
		border-style: dashed;
		animation: pulse-border 1.6s ease-in-out infinite;
	}

	.working-label {
		font-size: 0.88rem;
		font-weight: 600;
		color: var(--color-surface-600-400);
		display: flex;
		align-items: center;
	}

	.spinner {
		width: 16px;
		height: 16px;
		border: 2px solid var(--color-surface-300-700);
		border-top-color: var(--color-primary-500);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	.dots span {
		animation: blink 1.4s infinite both;
	}
	.dots span:nth-child(2) {
		animation-delay: 0.2s;
	}
	.dots span:nth-child(3) {
		animation-delay: 0.4s;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	@keyframes blink {
		0%,
		80%,
		100% {
			opacity: 0;
		}
		40% {
			opacity: 1;
		}
	}

	@keyframes pulse-border {
		0%,
		100% {
			border-color: var(--color-surface-300-700);
		}
		50% {
			border-color: var(--color-primary-500);
		}
	}

	.step-body {
		flex: 1;
		min-width: 0;
	}

	.step-head {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 4px;
	}

	.step-label {
		font-size: 0.8rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.03em;
		color: var(--color-surface-600-400);
	}

	.step-url {
		font-size: 0.8rem;
		color: var(--color-primary-500);
		text-decoration: none;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.step-content {
		font-size: 0.85rem;
		line-height: 1.45;
		color: var(--color-surface-700-300);
		white-space: pre-wrap;
		word-break: break-word;
	}

	.answer {
		border: 1px solid var(--color-surface-300-700);
		border-radius: 10px;
		padding: 16px 20px;
		background: var(--color-surface-50-950);
	}

	.markdown {
		font-size: 0.95rem;
		line-height: 1.6;
		color: var(--color-surface-900-100);
	}

	.markdown :global(h1),
	.markdown :global(h2),
	.markdown :global(h3) {
		font-weight: 700;
		margin: 1em 0 0.5em;
		line-height: 1.3;
	}

	.markdown :global(h1) {
		font-size: 1.4rem;
	}
	.markdown :global(h2) {
		font-size: 1.2rem;
	}
	.markdown :global(h3) {
		font-size: 1.05rem;
	}

	.markdown :global(p) {
		margin: 0.6em 0;
	}

	.markdown :global(ul),
	.markdown :global(ol) {
		margin: 0.6em 0;
		padding-left: 1.4em;
	}

	.markdown :global(li) {
		margin: 0.25em 0;
	}

	.markdown :global(a) {
		color: var(--color-primary-500);
		text-decoration: underline;
	}

	.markdown :global(code) {
		font-family: ui-monospace, monospace;
		font-size: 0.85em;
		background: var(--color-surface-200-800);
		padding: 0.1em 0.35em;
		border-radius: 4px;
	}

	.markdown :global(pre) {
		background: var(--color-surface-200-800);
		padding: 12px;
		border-radius: 8px;
		overflow-x: auto;
	}

	.markdown :global(pre code) {
		background: transparent;
		padding: 0;
	}

	.markdown :global(blockquote) {
		border-left: 3px solid var(--color-surface-400-600);
		margin: 0.6em 0;
		padding-left: 1em;
		color: var(--color-surface-600-400);
	}

	.markdown :global(table) {
		border-collapse: collapse;
		margin: 0.6em 0;
		width: 100%;
	}

	.markdown :global(th),
	.markdown :global(td) {
		border: 1px solid var(--color-surface-300-700);
		padding: 6px 10px;
		text-align: left;
	}

	.research-empty {
		padding: 24px;
		text-align: center;
		color: var(--color-surface-500);
		font-size: 0.9rem;
	}
</style>
