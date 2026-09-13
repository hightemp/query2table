<script lang="ts">
	import type { LlmIssueEvent } from '$lib/types';
	import { presentError } from '$lib/utils/errors';
	import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';

	let { issues, runStatus }: { issues: LlmIssueEvent[]; runStatus: string } = $props();
	const labels: Record<string, string> = {
		interpreter: 'Analyzing query',
		planner: 'Planning schema',
		schema_planner: 'Planning schema',
		search_planner: 'Planning search queries',
		query_expander: 'Expanding search queries',
		extractor: 'Extracting data',
		image_ranker: 'Ranking images',
		link_ranker: 'Ranking links',
		research: 'Research',
		setup: 'Setting up the run',
		image_search_planner: 'Planning image searches',
		link_search_planner: 'Planning link searches',
	};
	function count(value: number) {
		return value.toLocaleString('en-US');
	}
</script>

{#if issues.length}
	<section class="llm-issues" aria-label="Model request issues">
		<details class="issues-disclosure">
			<summary
				><strong>Model request issues ({issues.length})</strong><span
					>{presentError(issues[issues.length - 1].message, 'run', issues[issues.length - 1].code)
						.title}</span
				></summary
			>
			<p class="intro">
				{runStatus === 'completed'
					? 'The run completed. Review these request failures for possible missing results.'
					: 'Request failures and automatic retries are recorded here.'}
			</p>
			<div class="issue-list">
				{#each [...issues].reverse() as issue}
					<ErrorNotice error={issue.message} code={issue.code} liveRole="status">
						<p class="model">
							{[
								issue.provider,
								issue.model,
								issue.stage ? (labels[issue.stage] ?? issue.stage) : null,
							]
								.filter(Boolean)
								.join(' · ')}
						</p>
						<dl>
							<div>
								<dt>Requested output cap</dt>
								<dd>{count(issue.max_tokens)} tokens per request</dd>
							</div>
							{#if issue.prompt_tokens !== null}<div>
									<dt>Reported input</dt>
									<dd>{count(issue.prompt_tokens)} tokens</dd>
								</div>{/if}
							{#if issue.completion_tokens !== null}<div>
									<dt>Reported output</dt>
									<dd>{count(issue.completion_tokens)} tokens</dd>
								</div>{/if}
							{#if issue.reasoning_tokens !== null}<div>
									<dt>Reported thinking</dt>
									<dd>{count(issue.reasoning_tokens)} tokens</dd>
								</div>{/if}
						</dl>
						{#if issue.prompt_tokens === null && issue.completion_tokens === null && issue.reasoning_tokens === null}<p
							>
								Token usage was not reported by the provider.
							</p>{/if}
						<p class="retry">
							Attempt {issue.attempt} of {issue.max_attempts}.
							{#if issue.will_retry}An automatic retry was scheduled{#if issue.retry_after_ms !== null}{' after '}{count(
										issue.retry_after_ms / 1000
									)} s{/if}.
							{:else}No automatic retry was scheduled for this request.{/if}
						</p>
					</ErrorNotice>
				{/each}
			</div>
		</details>
		<p class="latest-action">
			{presentError(issues[issues.length - 1].message, 'run', issues[issues.length - 1].code)
				.action}
		</p>
	</section>
{/if}

<style>
	summary {
		cursor: pointer;
		display: flex;
		flex-wrap: wrap;
		gap: 4px 12px;
		color: var(--color-warning-500);
	}
	summary span {
		color: var(--app-muted);
		font-size: 12px;
	}
	.latest-action {
		font-size: 13px;
		margin: 8px 0;
	}
	.llm-issues {
		flex-shrink: 0;
		margin: 8px 0 12px;
		min-width: 0;
	}

	.intro {
		font-size: 0.82rem;
		margin: 0;
		color: var(--color-surface-600-400);
	}
	.issue-list {
		max-height: 220px;
		overflow-y: auto;
		padding-right: 4px;
	}
	.model {
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	dl {
		margin: 8px 0;
		display: flex;
		gap: 6px 18px;
		flex-wrap: wrap;
	}
	dl div {
		display: flex;
		flex-direction: column;
	}
	dt {
		color: var(--color-surface-600-400);
		font-size: 0.78rem;
	}
	dd {
		margin: 0;
	}
	.retry {
		font-weight: 500;
	}
</style>
