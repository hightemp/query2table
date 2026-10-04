<script lang="ts">
	import type { Accounting, LlmIssueEvent } from '$lib/types';
	import {
		InfoIcon,
		TriangleAlertIcon,
		CircleXIcon,
		ChevronDownIcon,
		ChevronUpIcon,
		XIcon,
	} from '@lucide/svelte';
	import {
		STAGE_LABELS,
		buildNotices,
		dismissalOf,
		isDismissed,
		noticesSummary,
		type Notice,
		type NoticeLevel,
	} from '$lib/utils/notices';

	let {
		issues,
		accounting,
		runStatus,
		turnCount = 1,
		dismissed = null,
		ondismiss,
	}: {
		issues: LlmIssueEvent[];
		accounting: Accounting | null;
		runStatus: string;
		/** Questions in a research conversation; labels notices once there is more than one. */
		turnCount?: number;
		dismissed?: Record<string, number> | null;
		ondismiss?: (dismissed: Record<string, number>) => void;
	} = $props();

	const icons = { info: InfoIcon, warning: TriangleAlertIcon, error: CircleXIcon } as const;
	let expanded = $state(false);
	let notices = $derived(buildNotices(issues, accounting, runStatus));
	let summary = $derived(noticesSummary(notices));
	let quiet = $derived(isDismissed(notices, dismissed));
	let SummaryIcon = $derived(icons[summary.level]);

	function count(value: number) {
		return value.toLocaleString('en-US');
	}
	function time(at: number | null | undefined) {
		return at ? new Date(at).toLocaleTimeString() : null;
	}
	function technicalLine(notice: Notice) {
		const last = notice.attempts.at(-1)!;
		return [last.provider, last.model, last.stage ? (STAGE_LABELS[last.stage] ?? last.stage) : null]
			.filter(Boolean)
			.join(' · ');
	}
	function levelName(level: NoticeLevel) {
		return { info: 'Note', warning: 'Warning', error: 'Error' }[level];
	}
</script>

{#if notices.length}
	<section
		class="run-notices"
		aria-label="Run notices"
		data-level={summary.level}
		data-dismissed={quiet ? 'true' : undefined}
	>
		<div class="strip">
			<button class="toggle" aria-expanded={expanded} onclick={() => (expanded = !expanded)}>
				<SummaryIcon size={15} />
				<strong>{summary.count} {summary.count === 1 ? 'notice' : 'notices'}</strong>
				<span class="summary-text" role="status">{summary.text}</span>
				{#if quiet}<span class="dismissed-label">· dismissed</span>{/if}
				{#if expanded}<ChevronUpIcon size={15} />{:else}<ChevronDownIcon size={15} />{/if}
			</button>
			{#if ondismiss && !quiet}
				<button
					class="icon-button dismiss"
					aria-label="Dismiss notices"
					title="Mark these notices as read"
					onclick={() => ondismiss(dismissalOf(notices))}><XIcon size={15} /></button
				>
			{/if}
		</div>
		{#if expanded}
			<div class="list">
				{#each notices as notice (notice.key)}
					{@const Icon = icons[notice.level]}
					{@const last = notice.attempts.at(-1)}
					<article class="notice" data-level={notice.level} aria-label={notice.title}>
						<span class="notice-icon" title={levelName(notice.level)}><Icon size={16} /></span>
						<div class="notice-body">
							<p class="notice-title">
								<strong>{notice.title}</strong>
								{#if notice.count > 1}<span class="badge">×{notice.count}</span>{/if}
								{#if turnCount > 1 && notice.turnIndex !== null}<span class="badge"
										>Question {notice.turnIndex + 1}</span
									>{/if}
							</p>
							<p>{notice.consequence}</p>
							{#if notice.action}<a href={notice.action.href}>{notice.action.label}</a>{/if}
							{#if last}
								<details>
									<summary>Technical details</summary>
									<p class="technical">{technicalLine(notice)}</p>
									<dl>
										<div>
											<dt>Requested output cap</dt>
											<dd>{count(last.max_tokens)} tokens per request</dd>
										</div>
										{#if last.prompt_tokens !== null}<div>
												<dt>Reported input</dt>
												<dd>{count(last.prompt_tokens)} tokens</dd>
											</div>{/if}
										{#if last.completion_tokens !== null}<div>
												<dt>Reported output</dt>
												<dd>{count(last.completion_tokens)} tokens</dd>
											</div>{/if}
										{#if last.reasoning_tokens !== null}<div>
												<dt>Reported thinking</dt>
												<dd>{count(last.reasoning_tokens)} tokens</dd>
											</div>{/if}
									</dl>
									<ul class="attempts">
										{#each notice.attempts as attempt}
											<li>
												{attempt.outcome === 'recovered'
													? `Attempt ${attempt.attempt} succeeded`
													: `Attempt ${attempt.attempt} of ${attempt.max_attempts}`}{#if time(attempt.at)}{' · '}{time(
														attempt.at
													)}{/if}{#if attempt.outcome !== 'recovered'}{' · '}{attempt.message}{/if}
											</li>
										{/each}
									</ul>
								</details>
							{/if}
						</div>
					</article>
				{/each}
			</div>
		{/if}
	</section>
{/if}

<style>
	.run-notices {
		--notice-color: var(--app-accent);
		flex-shrink: 0;
		min-width: 0;
		margin: 6px 0 10px;
		border: 1px solid var(--app-border);
		border-left: 3px solid var(--notice-color);
		border-radius: var(--app-radius);
		background: var(--app-panel);
		font-size: var(--app-text-md);
	}
	[data-level='warning'] {
		--notice-color: var(--app-warning);
	}
	[data-level='error'] {
		--notice-color: var(--app-danger);
	}
	.run-notices[data-dismissed] {
		--notice-color: var(--app-border);
		background: transparent;
		color: var(--app-muted);
	}
	.strip {
		display: flex;
		align-items: center;
		gap: 4px;
		padding-right: 4px;
	}
	.toggle {
		display: flex;
		flex: 1;
		align-items: center;
		gap: 8px;
		min-width: 0;
		padding: 6px 10px;
		text-align: left;
	}
	.toggle > :global(svg:first-child) {
		flex-shrink: 0;
		color: var(--notice-color);
	}
	.run-notices[data-dismissed] .toggle > :global(svg:first-child) {
		color: var(--app-muted);
	}
	.toggle strong {
		flex-shrink: 0;
		font-weight: 600;
	}
	.summary-text {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--app-muted);
	}
	.dismissed-label {
		flex-shrink: 0;
		color: var(--app-muted);
	}
	.toggle > :global(svg:last-child) {
		flex-shrink: 0;
		margin-left: auto;
		color: var(--app-muted);
	}
	.list {
		max-height: 40vh;
		overflow: auto;
		border-top: 1px solid var(--app-border);
	}
	.notice {
		display: grid;
		grid-template-columns: 20px minmax(0, 1fr);
		gap: 8px;
		padding: 10px 12px;
		border-bottom: 1px solid var(--app-border);
	}
	.notice:last-child {
		border-bottom: 0;
	}
	.notice[data-level='info'] {
		--notice-color: var(--app-accent);
	}
	.notice[data-level='warning'] {
		--notice-color: var(--app-warning);
	}
	.notice[data-level='error'] {
		--notice-color: var(--app-danger);
	}
	.notice-icon {
		display: flex;
		padding-top: 2px;
		color: var(--notice-color);
	}
	.notice-body {
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.notice-body p {
		margin: 2px 0;
		line-height: 1.45;
	}
	.notice-title {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
	}
	.badge {
		padding: 0 7px;
		border-radius: var(--app-radius-pill);
		background: var(--app-subtle);
		color: var(--app-muted);
		font-size: var(--app-text-xs);
		line-height: 1.7;
	}
	a {
		display: inline-block;
		margin-top: 2px;
		color: var(--app-accent);
		text-decoration: underline;
	}
	details {
		margin-top: 6px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	summary {
		cursor: pointer;
	}
	.technical {
		font-weight: 600;
		color: var(--app-text);
	}
	dl {
		display: flex;
		flex-wrap: wrap;
		gap: 6px 18px;
		margin: 6px 0;
	}
	dl div {
		display: flex;
		flex-direction: column;
	}
	dt {
		font-size: var(--app-text-xs);
	}
	dd {
		margin: 0;
		color: var(--app-text);
	}
	.attempts {
		margin: 4px 0 0;
		padding-left: 18px;
	}
</style>
