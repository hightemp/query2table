<script lang="ts">
	import { tooltip } from '$lib/actions/tooltip';
	import { t } from '$lib/i18n';
	import { copyText } from '$lib/api/tauri';
	import { toast } from '$lib/stores/toasts';
	import { CopyIcon, CheckIcon } from '@lucide/svelte';
	let { text, label = '' }: { text: string; label?: string } = $props();
	let copied = $state(false);
	let timer: ReturnType<typeof setTimeout> | undefined;
	$effect(() => {
		void text;
		copied = false;
		return () => clearTimeout(timer);
	});
	async function copy() {
		try {
			await copyText(text);
			copied = true;
			clearTimeout(timer);
			timer = setTimeout(() => (copied = false), 2000);
		} catch {
			toast(t('common.copyFailed'), 'error');
		}
	}
</script>

<button
	class="icon-button"
	aria-label={copied ? t('common.copied') : label || t('common.copy')}
	use:tooltip={copied ? t('common.copied') : label || t('common.copy')}
	onclick={copy}
	>{#if copied}<CheckIcon size={15} />{:else}<CopyIcon size={15} />{/if}</button
>
