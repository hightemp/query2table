<script lang="ts">
	import { copyText } from '$lib/api/tauri';
	import { toast } from '$lib/stores/toasts';
	import { CopyIcon, CheckIcon } from '@lucide/svelte';
	let { text, label = 'Copy value' }: { text: string; label?: string } = $props();
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
			toast('Could not copy. Select the text and copy it manually.', 'error');
		}
	}
</script>

<button
	class="icon-button"
	aria-label={copied ? 'Copied' : label}
	title={copied ? 'Copied' : label}
	onclick={copy}
	>{#if copied}<CheckIcon size={15} />{:else}<CopyIcon size={15} />{/if}</button
>
