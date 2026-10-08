<script lang="ts">
	import { ExternalLinkIcon } from '@lucide/svelte';
	import { t } from '$lib/i18n';
	import Dialog from './Dialog.svelte';
	import ErrorNotice from './ErrorNotice.svelte';
	import { getAttachmentFragment, openAttachment } from '$lib/api/tauri';
	import { attachmentPreview } from '$lib/stores/attachmentPreview';
	import { errorText } from '$lib/utils/errors';
	import { placeLabel } from '$lib/utils/attachments';
	import type { AttachmentFragment } from '$lib/types';

	let { url }: { url: string } = $props();
	let fragment = $state<AttachmentFragment | null>(null);
	let loading = $state(true);
	let error = $state('');

	$effect(() => {
		const current = url;
		loading = true;
		error = '';
		fragment = null;
		getAttachmentFragment(current)
			.then((found) => {
				if (current !== url) return;
				fragment = found;
				if (!found) error = t('attachments.gone');
			})
			.catch((reason) => (error = errorText(reason)))
			.finally(() => (loading = false));
	});

	async function openFile() {
		if (!fragment) return;
		try {
			await openAttachment(fragment.attachment_id);
			close();
		} catch (reason) {
			error = errorText(reason);
		}
	}
	const close = () => attachmentPreview.set(null);
</script>

<Dialog title={fragment ? placeLabel(fragment.file_name, fragment.locator) : t('attachments.preview')} onclose={close}>
	{#if loading}<p class="muted" role="status">{t('attachments.reading')}</p>
	{:else if error}<ErrorNotice {error} />
	{:else if fragment}<pre class="fragment">{fragment.text}</pre>{/if}
	{#snippet footer()}
		<button class="button" onclick={close}>{t('common.close')}</button>
		<button class="button primary" onclick={openFile} disabled={!fragment}><ExternalLinkIcon size={15} />{t('attachments.openFile')}</button>
	{/snippet}
</Dialog>

<style>
	.fragment {
		max-height: min(60vh, 520px);
		overflow: auto;
		margin: 0;
		padding: 12px;
		border-radius: var(--app-radius);
		background: var(--app-subtle);
		font: var(--app-text-md) / 1.6 var(--app-font-sans);
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	.muted {
		color: var(--app-muted);
	}
</style>
