<script lang="ts">
	import { onMount } from 'svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import {
		FileIcon,
		FileSpreadsheetIcon,
		FileTextIcon,
		LoaderCircleIcon,
		PaperclipIcon,
		TriangleAlertIcon,
		XIcon,
	} from '@lucide/svelte';
	import { t } from '$lib/i18n';
	import { tooltip } from '$lib/actions/tooltip';
	import { openAttachment } from '$lib/api/tauri';
	import { settings } from '$lib/stores/settings';
	import { toast } from '$lib/stores/toasts';
	import { MAX_ATTACHMENTS, queryFiles, type AttachmentDraft, type DraftAttachment } from '$lib/stores/attachments';
	import { ATTACHMENT_EXTENSIONS, attachmentMeta, privacyNote } from '$lib/utils/attachments';
	import { refreshVisionStatus, visionStatus } from '$lib/stores/vision';
	import { errorText } from '$lib/utils/errors';

	let {
		draft = queryFiles,
		disabled = false,
		dragging = $bindable(false),
		compact = false,
	}: {
		/** Which question the files belong to. */
		draft?: AttachmentDraft;
		disabled?: boolean;
		/** True while files are dragged over the window. */
		dragging?: boolean;
		/** Without the hint line (inside the follow-up box). */
		compact?: boolean;
	} = $props();
	let draftAttachments = $derived(draft.items);

	let provider = $derived($settings.get('llm_provider') ?? 'openrouter');
	let note = $derived(privacyNote(provider, $settings));
	// Recheck who reads images whenever the saved model settings change.
	let visionKey = $derived(
		['llm_provider', 'openrouter_model', 'ollama_model', 'ollama_cloud_model', 'openai_model', 'llm_vision', 'vision_model']
			.map((key) => $settings.get(key) ?? '')
			.join('|')
	);
	$effect(() => {
		void visionKey;
		void refreshVisionStatus();
	});
	/** No model can read pictures or scans; unknown status does not warn. */
	let noReader = $derived(!!$visionStatus && !$visionStatus.reader);
	const blindPicture = (item: DraftAttachment) => noReader && item.info?.kind === 'image';
	const blindScan = (item: DraftAttachment) => noReader && item.info?.status === 'needs_vision';
	let needsReader = $derived($draftAttachments.some((d) => blindPicture(d) || blindScan(d)));

	function tooMany(refused: number) {
		if (refused) toast(t('attachments.limit', { count: MAX_ATTACHMENTS }), 'error');
	}

	async function choose() {
		const all = [...ATTACHMENT_EXTENSIONS.documents, ...ATTACHMENT_EXTENSIONS.spreadsheets, ...ATTACHMENT_EXTENSIONS.images];
		const picked = await open({
			multiple: true,
			directory: false,
			title: t('attachments.choose'),
			filters: [
				{ name: t('attachments.supported'), extensions: all },
				{ name: t('attachments.documents'), extensions: ATTACHMENT_EXTENSIONS.documents },
				{ name: t('attachments.spreadsheets'), extensions: ATTACHMENT_EXTENSIONS.spreadsheets },
				{ name: t('attachments.images'), extensions: ATTACHMENT_EXTENSIONS.images },
			],
		});
		const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
		if (paths.length) tooMany(await draft.attachPaths(paths));
	}

	async function openFile(item: DraftAttachment) {
		if (!item.info) return;
		try {
			await openAttachment(item.info.id);
		} catch (error) {
			toast(errorText(error), 'error');
		}
	}

	onMount(() => {
		void draft.restore();
		let unlisten: (() => void) | undefined;
		let disposed = false;
		void import('@tauri-apps/api/webview')
			.then(({ getCurrentWebview }) =>
				getCurrentWebview().onDragDropEvent(async (event) => {
					const payload = event.payload;
					if (payload.type === 'enter' || payload.type === 'over') dragging = !disabled;
					else if (payload.type === 'leave') dragging = false;
					else if (payload.type === 'drop') {
						dragging = false;
						if (!disabled && payload.paths.length) tooMany(await draft.attachPaths(payload.paths));
					}
				})
			)
			.then((stop) => (disposed ? stop() : (unlisten = stop)))
			.catch(() => {
				// Outside the desktop app (tests in a plain browser) there is nothing to drop.
			});
		return () => {
			disposed = true;
			unlisten?.();
		};
	});
</script>

<div class="attachments">
	<div class="bar">
		<button type="button" class="button ghost sm attach" onclick={choose} {disabled}>
			<PaperclipIcon size={15} />{t('attachments.attach')}
		</button>
		{#if !$draftAttachments.length && !compact}<span class="hint">{t('attachments.hint')}</span>{/if}
	</div>
	{#if $draftAttachments.length}
		<ul class="files" aria-label={t('attachments.list')}>
			{#each $draftAttachments as item (item.key)}
				<li class="file" class:error={item.state === 'error'}>
					<span class="icon">
						{#if item.state === 'reading'}<LoaderCircleIcon size={18} class="spin" />
						{:else if item.info?.thumbnail}<img src={item.info.thumbnail} alt={item.name} />
						{:else if item.info?.kind === 'spreadsheet'}<FileSpreadsheetIcon size={18} />
						{:else if item.info?.kind === 'document' || item.info?.kind === 'text'}<FileTextIcon size={18} />
						{:else}<FileIcon size={18} />{/if}
					</span>
					<span class="text">
						{#if item.info}<button
								type="button"
								class="name"
								aria-label={t('attachments.open', { name: item.name })}
								use:tooltip={{ text: item.name, whenTruncated: true }}
								onclick={() => openFile(item)}>{item.name}</button
							>{:else}<span class="name plain" use:tooltip={{ text: item.name, whenTruncated: true }}>{item.name}</span>{/if}
						<span class="meta">
							{#if item.state === 'reading'}{t('attachments.reading')}
							{:else if item.state === 'error'}{item.error}
							{:else if item.info && blindPicture(item)}<span class="warning" use:tooltip={t('attachments.blindHint')}
									><TriangleAlertIcon size={12} /></span
								>{t('attachments.blind')}
							{:else if item.info}{#if item.info.status === 'needs_vision'}<span
										class="warning"
										use:tooltip={blindScan(item) ? t('attachments.scanNoReader') : t('attachments.scanHint')}
										><TriangleAlertIcon size={12} /></span
									>{/if}{attachmentMeta(item.info)}{/if}
						</span>
					</span>
					<button
						type="button"
						class="icon-button ghost sm remove"
						aria-label={t('attachments.remove', { name: item.name })}
						onclick={() => draft.remove(item.key)}><XIcon size={14} /></button
					>
				</li>
			{/each}
		</ul>
		<p class="privacy">
			{note}
			{#if needsReader}<a href="/settings#settings-llm">{t('attachments.chooseVision')}</a>{/if}
		</p>
	{/if}
</div>

<style>
	.attachments {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 8px;
	}
	.bar {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}
	.attach {
		color: var(--app-muted);
	}
	.attach:hover:not(:disabled) {
		color: var(--app-text);
	}
	.hint,
	.privacy {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.privacy a {
		margin-left: 6px;
		color: var(--app-accent);
	}
	.files {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 8px;
	}
	.file {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
		padding: 6px 6px 6px 8px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		background: var(--app-bg);
	}
	.file.error {
		border-color: color-mix(in srgb, var(--app-danger) 50%, var(--app-border));
	}
	.icon {
		display: grid;
		place-items: center;
		flex-shrink: 0;
		width: 34px;
		height: 34px;
		border-radius: var(--app-radius-sm);
		background: var(--app-subtle);
		color: var(--app-muted);
		overflow: hidden;
	}
	.icon img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.icon :global(.spin) {
		animation: spin 1s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	.text {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
	}
	.name {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		padding: 0;
		border: 0;
		background: none;
		color: var(--app-text);
		font-size: var(--app-text-md);
		font-weight: 550;
		text-align: left;
	}
	button.name:hover {
		color: var(--app-accent);
		text-decoration: underline;
	}
	.meta {
		display: flex;
		align-items: center;
		gap: 4px;
		min-width: 0;
		color: var(--app-muted);
		font-size: var(--app-text-xs);
	}
	.error .meta {
		color: var(--app-danger);
		white-space: normal;
	}
	.warning {
		display: inline-flex;
		color: var(--app-warning);
	}
	.remove {
		width: 26px;
		min-height: 26px;
		padding: 4px;
		border: 0;
		background: transparent;
	}
	@media (prefers-reduced-motion: reduce) {
		.icon :global(.spin) {
			animation: none;
		}
	}
</style>
