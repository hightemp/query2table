<script lang="ts">
	import { copyText } from '$lib/api/tauri';
	import { CopyIcon, CheckIcon } from '@lucide/svelte';
	let { text, label = 'Copy value' }: { text: string; label?: string } = $props();
	let copied = $state(false);
	let failed = $state(false);
	$effect(() => {
		void text;
		copied = false;
		failed = false;
	});
	async function copy() {
		failed = false;
		try {
			await copyText(text);
			copied = true;
		} catch {
			failed = true;
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
{#if failed}<span role="alert">Could not copy. Select the text and copy it manually.</span>{/if}

<style>
	span {
		display: block;
		font-size: 12px;
		color: var(--color-error-500);
	}
</style>
