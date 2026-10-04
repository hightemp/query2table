<script lang="ts">
	import type { ResearchStep } from '$lib/types';
	import { SearchIcon, FileTextIcon, LightbulbIcon, TriangleAlertIcon } from '@lucide/svelte';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import { describeStep, type StepView } from '$lib/utils/research';

	let { steps, running = false }: { steps: ResearchStep[]; running?: boolean } = $props();
	const icons = {
		search: SearchIcon,
		fetch: FileTextIcon,
		think: LightbulbIcon,
		error: TriangleAlertIcon,
	} as const;
	let views = $derived(steps.map(describeStep));
	let expanded = $state<string[]>([]);
	let rawShown = $state<string[]>([]);
	let current = $derived.by(() => {
		const last = views.at(-1);
		if (!running) return null;
		if (!last) return 'Starting research…';
		if (last.step.step_type === 'search') return `Searching: ${last.summary}`;
		if (last.step.step_type === 'fetch') return `Reading ${last.domain ?? last.summary}`;
		return 'Analyzing findings…';
	});
	function toggle(list: string[], id: string) {
		return list.includes(id) ? list.filter((item) => item !== id) : [...list, id];
	}
	// Summaries are cut to one line; "Show more" appears only when text is actually cut.
	let truncated = $state<string[]>([]);
	function expandable(view: StepView) {
		return truncated.includes(view.step.id) || expanded.includes(view.step.id);
	}
	function measure(element: HTMLElement, stepId: string) {
		const check = () => {
			const cut = element.scrollWidth > element.clientWidth + 1;
			if (cut !== truncated.includes(stepId))
				truncated = cut ? [...truncated, stepId] : truncated.filter((item) => item !== stepId);
		};
		const observer = new ResizeObserver(() => {
			if (!expanded.includes(stepId)) check();
		});
		observer.observe(element);
		check();
		return { destroy: () => observer.disconnect() };
	}
</script>

{#if current}<p class="current-step" role="status"><span class="pulse"></span>{current}</p>{/if}
{#if !views.length}
	<p class="empty">No research steps yet.</p>
{:else}
	<ol class="steps">
		{#each views as view (view.step.id)}
			{@const Icon = icons[view.step.step_type as keyof typeof icons] ?? LightbulbIcon}
			{@const open = expanded.includes(view.step.id)}
			<li class="step {view.step.step_type}">
				<span class="step-number">{view.number}</span>
				<span class="icon"><Icon size={15} /></span>
				<div class="step-body">
					<div class="step-line">
						<strong>{view.label}</strong>
						<span
							class="summary"
							class:open
							title={open ? undefined : view.summary}
							use:measure={view.step.id}>{view.summary}</span
						>
					</div>
					{#if view.step.url && view.step.step_type === 'fetch'}<ExternalLink
							href={view.step.url}
							class="step-url"
						/>{/if}
					{#if expandable(view) || view.rawText}
						<div class="step-actions">
							{#if expandable(view)}<button
									class="button ghost sm"
									aria-expanded={open}
									onclick={() => (expanded = toggle(expanded, view.step.id))}
									>{open ? 'Show less' : 'Show more'}</button
								>{/if}
							{#if view.rawText}<button
									class="button ghost sm"
									aria-expanded={rawShown.includes(view.step.id)}
									onclick={() => (rawShown = toggle(rawShown, view.step.id))}
									>{rawShown.includes(view.step.id)
											? view.rawKind === 'page' ? 'Hide page text' : 'Hide details'
											: view.rawKind === 'page' ? 'Show page text' : 'Show details'}</button
								>{/if}
						</div>
					{/if}
					{#if view.rawText && rawShown.includes(view.step.id)}<pre>{view.rawText}</pre>{/if}
				</div>
			</li>
		{/each}
	</ol>
{/if}

<style>
	.current-step {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 12px;
		padding: 10px 12px;
		border-radius: var(--app-radius);
		background: color-mix(in srgb, var(--app-accent) 8%, var(--app-panel));
		font-weight: 600;
	}
	.pulse {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--app-accent);
		animation: pulse 1.6s ease-in-out infinite;
	}
	@keyframes pulse {
		50% {
			opacity: 0.35;
		}
	}
	.empty {
		color: var(--app-muted);
	}
	.steps {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.step {
		display: grid;
		grid-template-columns: 28px 20px minmax(0, 1fr);
		gap: 6px;
		align-items: start;
		padding: 9px 0;
		border-bottom: 1px solid var(--app-border);
		font-size: var(--app-text-md);
	}
	.step-number {
		color: var(--app-muted);
		text-align: right;
		font-variant-numeric: tabular-nums;
		line-height: 1.6;
	}
	.icon {
		display: flex;
		padding-top: 3px;
		color: var(--app-muted);
	}
	.error .icon,
	.error strong {
		color: var(--app-danger);
	}
	.step-body {
		min-width: 0;
	}
	.step-line {
		display: flex;
		gap: 8px;
		min-width: 0;
		line-height: 1.6;
	}
	.step-line strong {
		flex-shrink: 0;
		font-weight: 600;
	}
	.summary {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--app-muted);
	}
	.summary.open {
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	.step :global(.step-url) {
		font-size: var(--app-text-sm);
	}
	.step-actions {
		display: flex;
		gap: 4px;
		margin-top: 4px;
	}
	pre {
		max-height: 260px;
		overflow: auto;
		margin-top: 6px;
		padding: 10px;
		border-radius: var(--app-radius-sm);
		background: var(--app-bg);
		font-size: var(--app-text-sm);
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
</style>
