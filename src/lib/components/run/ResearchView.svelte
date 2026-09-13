<script lang="ts">
	import type { ResearchStep } from '$lib/types';
	import Markdown from '$lib/components/common/Markdown.svelte';
	import ExternalLink from '$lib/components/common/ExternalLink.svelte';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
	let {
		steps,
		answer,
		running = false,
	}: { steps: ResearchStep[]; answer: string | null; running?: boolean } = $props();
	let activityOpen = $state(true);
	$effect(() => {
		if (answer && !running) activityOpen = false;
	});
	const labels: Record<string, string> = {
		search: 'Search',
		fetch: 'Read page',
		think: 'Analysis',
		error: 'Request issue',
	};
</script>

<div class="research-view">
	{#if answer}<section class="answer" aria-label="Research answer">
			<h2>Answer</h2>
			<Markdown content={answer} />
		</section>{/if}
	{#if steps.length || running}
		<details class="activity" bind:open={activityOpen}>
			<summary>Activity <span>{steps.length} steps{running ? ' · Working…' : ''}</span></summary>
			<ol>
				{#each steps as step (step.id)}<li>
						<details>
							<summary
								>{labels[step.step_type] ?? step.step_type}<span
									>{step.content.slice(0, 100)}{step.content.length > 100 ? '…' : ''}</span
								></summary
							>{#if step.url}<ExternalLink
									href={step.url}
								/>{/if}{#if step.step_type === 'error'}<ErrorNotice
									error={step.content}
								/>{:else}<div class="step-content">{step.content}</div>{/if}
						</details>
					</li>{/each}
			</ol>
			{#if running && !answer}<p class="working" role="status">
					Research is in progress. New activity will appear here.
				</p>{/if}
		</details>
	{:else if !answer}<div class="empty-state">No research output yet.</div>{/if}
</div>

<style>
	.research-view {
		min-height: 0;
		min-width: 0;
		overflow: auto;
		flex: 1;
		scrollbar-gutter: stable;
		padding: 0 12px 16px 0;
	}
	.answer,
	.activity {
		border: 1px solid var(--app-border);
		background: var(--app-panel);
		border-radius: 12px;
		padding: 20px;
		margin-bottom: 16px;
	}
	h2 {
		font-size: 12px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--app-muted);
		margin-bottom: 16px;
	}
	summary {
		cursor: pointer;
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	summary span {
		color: var(--app-muted);
		font-size: 12px;
		font-weight: 400;
		margin-left: 8px;
	}
	ol {
		list-style: decimal;
		padding-left: 24px;
		margin-top: 12px;
	}
	li {
		padding: 10px 0;
		border-top: 1px solid var(--app-border);
	}
	.step-content {
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		font-size: 13px;
		margin: 10px 0;
	}
	.working {
		color: var(--app-muted);
		margin-top: 12px;
		font-size: 13px;
	}
</style>
