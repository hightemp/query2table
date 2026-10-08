<script lang="ts">
	import { FileIcon, FileSpreadsheetIcon, FileTextIcon } from '@lucide/svelte';
	import { t } from '$lib/i18n';
	import { tooltip } from '$lib/actions/tooltip';
	import { openAttachment } from '$lib/api/tauri';
	import { toast } from '$lib/stores/toasts';
	import { errorText } from '$lib/utils/errors';
	import { attachmentMeta } from '$lib/utils/attachments';
	import type { AttachmentInfo } from '$lib/types';

	let { files }: { files: AttachmentInfo[] } = $props();

	async function open(file: AttachmentInfo) {
		try {
			await openAttachment(file.id);
		} catch (error) {
			toast(errorText(error), 'error');
		}
	}
</script>

{#if files.length}
	<ul class="file-chips" aria-label={t('attachments.list')}>
		{#each files as file (file.id)}
			<li>
				<button
					type="button"
					class="chip"
					aria-label={t('attachments.open', { name: file.file_name })}
					use:tooltip={attachmentMeta(file)}
					onclick={() => open(file)}
				>
					{#if file.thumbnail}<img src={file.thumbnail} alt="" />
					{:else if file.kind === 'spreadsheet'}<FileSpreadsheetIcon size={14} />
					{:else if file.kind === 'image'}<FileIcon size={14} />
					{:else}<FileTextIcon size={14} />{/if}
					<span class="name">{file.file_name}</span>
				</button>
			</li>
		{/each}
	</ul>
{/if}

<style>
	.file-chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin: 6px 0 10px;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		max-width: 260px;
		padding: 3px 10px 3px 6px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius-pill);
		background: var(--app-panel);
		color: var(--app-text);
		font-size: var(--app-text-sm);
	}
	.chip:hover {
		border-color: var(--app-accent);
		color: var(--app-accent);
	}
	.chip :global(svg) {
		flex-shrink: 0;
		color: var(--app-muted);
	}
	img {
		width: 18px;
		height: 18px;
		border-radius: 4px;
		object-fit: cover;
	}
	.name {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
