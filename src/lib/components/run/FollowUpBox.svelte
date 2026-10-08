<script lang="ts" module>
	import type { AttachmentInfo as Info } from '$lib/types';
	import type { SourceMode as Mode } from '$lib/api/tauri';
	/** Files sent with a follow-up question. */
	export interface FollowUpFiles {
		attachments: Info[];
		sourceMode: Mode;
	}
</script>

<script lang="ts">
	import { t } from '$lib/i18n';
	import NumberInput from '$lib/components/common/NumberInput.svelte';
	import { SendIcon, ChevronDownIcon, ChevronUpIcon } from '@lucide/svelte';
	import type { SourceMode, StopConditions } from '$lib/api/tauri';
	import AttachmentBar from '$lib/components/query/AttachmentBar.svelte';
	import Checkbox from '$lib/components/common/Checkbox.svelte';
	import { followUpFiles } from '$lib/stores/attachments';
	import { hasMod, modKey } from '$lib/utils/shortcuts';
	import { errorText } from '$lib/utils/errors';
	import {
		formatMinutes,
		formatUsd,
		parseStopConditions,
		type StopConditionInput,
	} from '$lib/utils/stopConditions';

	let {
		suggestions = [],
		disabled = false,
		limits,
		hasFiles = false,
		sourceMode = 'web',
		onask,
	}: {
		suggestions?: string[];
		/** A question is being answered. */
		disabled?: boolean;
		/** Limits of the conversation's first question, used for the next one. */
		limits: Required<StopConditions>;
		/** Files were attached earlier in the conversation. */
		hasFiles?: boolean;
		/** Where the conversation looked so far. */
		sourceMode?: SourceMode;
		onask: (question: string, limits: Required<StopConditions>, files: FollowUpFiles) => Promise<void>;
	} = $props();
	const pending = followUpFiles.items;
	let searchWeb = $state(true);
	$effect(() => {
		searchWeb = sourceMode !== 'files';
	});
	let withFiles = $derived(hasFiles || $pending.some((d) => d.state === 'ready'));
	let readingFiles = $derived($pending.some((d) => d.state === 'reading'));

	let question = $state('');
	let sending = $state(false);
	let error = $state('');
	let editLimits = $state(false);
	let input = $state<StopConditionInput>({ target: '100', budget: '1', duration: '10' });
	let edited = $state(false);
	$effect(() => {
		if (edited) return;
		input = {
			target: String(limits.target_row_count),
			budget: String(limits.max_budget_usd),
			duration: String(Math.max(1, Math.round(limits.max_duration_seconds / 60))),
		};
	});
	let parsed = $derived(parseStopConditions(input, 'research'));
	let summary = $derived(
		parsed.conditions
			? `${t('units.step', { count: parsed.conditions.target_row_count })} · ${formatUsd(parsed.conditions.max_budget_usd)} · ${formatMinutes(parsed.conditions.max_duration_seconds)}`
			: t('query.checkValues')
	);
	let canSend = $derived(!!question.trim() && !disabled && !sending && !!parsed.conditions && !readingFiles);

	async function send() {
		if (!canSend || !parsed.conditions) return;
		sending = true;
		error = '';
		try {
			const attachments = followUpFiles.take();
			try {
				await onask(question.trim(), parsed.conditions, {
					attachments,
					sourceMode: withFiles && !searchWeb ? 'files' : 'web',
				});
			} catch (reason) {
				// Keep the files for another try.
				followUpFiles.items.set(attachments.map((info, i) => ({ key: `retry-${i}-${info.id}`, name: info.file_name, state: 'ready', info })));
				throw reason;
			}
			question = '';
		} catch (reason) {
			error = errorText(reason);
		} finally {
			sending = false;
		}
	}
	function field(key: keyof StopConditionInput, value: string) {
		edited = true;
		input = { ...input, [key]: value };
	}
</script>

<section class="follow-up" aria-label={t('followUp.region')}>
	{#if suggestions.length && !disabled}
		<div class="suggestions" role="group" aria-label={t('followUp.suggested')}>
			{#each suggestions as suggestion}<button
					class="chip"
					type="button"
					onclick={() => (question = suggestion)}>{suggestion}</button
				>{/each}
		</div>
	{/if}
	<div class="composer" class:disabled>
		<textarea
			class="input"
			rows="2"
			aria-label={t('followUp.ask')}
			placeholder={disabled ? t('followUp.answering') : t('followUp.placeholder')}
			{disabled}
			bind:value={question}
			onpaste={async (event) => {
				const files = [...(event.clipboardData?.files ?? [])];
				if (!files.length) return;
				event.preventDefault();
				await followUpFiles.attachFiles(files);
			}}
			onkeydown={(event) => {
				if (event.key === 'Enter' && hasMod(event)) {
					event.preventDefault();
					void send();
				}
			}}></textarea>
		<button class="button primary" disabled={!canSend} onclick={send} aria-keyshortcuts="Control+Enter"
			><SendIcon size={15} />{t('followUp.send')}</button
		>
	</div>
	<div class="files">
		<AttachmentBar draft={followUpFiles} disabled={disabled} compact />
		{#if withFiles}<span class="source-mode"><Checkbox bind:checked={searchWeb}>{t('attachments.searchWeb')}</Checkbox></span>{/if}
	</div>
	<div class="meta">
		<button
			class="limits-toggle"
			type="button"
			aria-expanded={editLimits}
			aria-label={t('followUp.limits', { summary })}
			onclick={() => (editLimits = !editLimits)}
			>{#if editLimits}<ChevronUpIcon size={14} />{:else}<ChevronDownIcon size={14} />{/if}<span
				class:invalid={!parsed.conditions}>{summary}</span
			></button
		>
		<span class="hint" aria-hidden="true">{t('followUp.hint', { shortcut: `${modKey}+Enter` })}</span>
	</div>
	{#if editLimits}
		<div class="limits">
			<label
				>{t('mode.research.target')}<NumberInput size="sm"
					min="1"
					max="200"
					value={input.target}
					oninput={(e) => field('target', e.currentTarget.value)}
				/></label
			>
			<label
				>{t('query.maxCost')}<NumberInput size="sm"
					min="0.01"
					step="0.01"
					value={input.budget}
					oninput={(e) => field('budget', e.currentTarget.value)}
				/></label
			>
			<label
				>{t('query.maxDuration')}<NumberInput size="sm"
					min="1"
					value={input.duration}
					oninput={(e) => field('duration', e.currentTarget.value)}
				/></label
			>
			{#if !parsed.conditions}<p class="error">
					{Object.values(parsed.errors).join(' ')}
				</p>{/if}
		</div>
	{/if}
	{#if error}<p class="error" role="alert">{error}</p>{/if}
</section>

<style>
	.follow-up {
		position: sticky;
		bottom: 0;
		z-index: 3;
		padding: 12px 0 4px;
		background: linear-gradient(transparent, var(--app-bg) 16px);
	}
	.files {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-top: 4px;
	}
	.files :global(.attachments) {
		margin-top: 0;
	}
	.source-mode {
		font-size: var(--app-text-sm);
	}
	.suggestions {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin-bottom: 8px;
	}
	.chip {
		padding: 4px 12px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius-pill);
		background: var(--app-panel);
		font-size: var(--app-text-md);
		text-align: left;
	}
	.chip:hover {
		border-color: var(--app-accent);
		color: var(--app-accent);
	}
	.composer {
		display: flex;
		align-items: flex-end;
		gap: 8px;
		padding: 8px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius-lg);
		background: var(--app-panel);
		box-shadow: var(--app-shadow-popover);
	}
	.composer:focus-within {
		border-color: var(--app-accent);
	}
	.composer textarea {
		flex: 1;
		min-height: 44px;
		max-height: 200px;
		border: 0;
		background: transparent;
		resize: none;
	}
	.composer textarea:focus {
		outline: none;
	}
	.meta {
		display: flex;
		justify-content: space-between;
		gap: 8px;
		margin-top: 4px;
		font-size: var(--app-text-sm);
		color: var(--app-muted);
	}
	.limits-toggle {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--app-muted);
	}
	.invalid {
		color: var(--app-danger);
	}
	.limits {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 6px;
		font-size: var(--app-text-sm);
		color: var(--app-muted);
	}
	.limits label {
		display: flex;
		flex-direction: column;
		gap: 3px;
		width: 140px;
	}
	.error {
		width: 100%;
		color: var(--app-danger);
		font-size: var(--app-text-sm);
	}
</style>
