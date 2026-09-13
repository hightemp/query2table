<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { XIcon } from '@lucide/svelte';
	let {
		title,
		onclose,
		busy = false,
		variant = 'modal',
		children,
		footer,
	}: {
		title: string;
		onclose: () => void;
		busy?: boolean;
		variant?: 'modal' | 'drawer' | 'image';
		children: Snippet;
		footer?: Snippet;
	} = $props();
	const titleId = $props.id();
	let dialog: HTMLDialogElement;
	function dismiss() {
		if (!busy) onclose();
	}
	onMount(() => {
		const previous = document.activeElement as HTMLElement | null;
		dialog.showModal();
		return () => {
			dialog.close();
			if (previous?.isConnected) previous.focus();
		};
	});
</script>

<!-- Native modal dialog supplies background inertness and keyboard focus containment. -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<dialog
	bind:this={dialog}
	class:drawer={variant === 'drawer'}
	class:image-dialog={variant === 'image'}
	aria-labelledby={titleId}
	oncancel={(event) => {
		event.preventDefault();
		dismiss();
	}}
	onclick={(event) => {
		if (event.target === dialog) {
			const rect = dialog.getBoundingClientRect();
			if (
				event.clientX < rect.left ||
				event.clientX > rect.right ||
				event.clientY < rect.top ||
				event.clientY > rect.bottom
			)
				dismiss();
		}
	}}
>
	<div class="dialog-frame">
		<header>
			<h2 id={titleId}>{title}</h2>
			<button class="icon-button" disabled={busy} onclick={dismiss} aria-label="Close"
				><XIcon size={18} /></button
			>
		</header>
		<div class="dialog-content">{@render children()}</div>
		{#if footer}<footer>{@render footer()}</footer>{/if}
	</div>
</dialog>

<style>
	dialog {
		padding: 0;
		margin: auto;
		width: min(480px, calc(100vw - 32px));
		max-width: calc(100vw - 32px);
		max-height: calc(100dvh - 32px);
		color: var(--app-text);
		background: var(--app-panel);
		border: 1px solid var(--app-border);
		border-radius: 12px;
		box-shadow: 0 24px 80px #0005;
		overflow: hidden;
	}
	dialog::backdrop {
		background: #0b122080;
		backdrop-filter: blur(3px);
	}
	.dialog-frame {
		display: flex;
		flex-direction: column;
		max-height: calc(100dvh - 34px);
		min-height: 0;
	}
	header,
	footer {
		display: flex;
		align-items: center;
		gap: 12px;
		flex-shrink: 0;
		padding: 16px 20px;
	}
	header {
		border-bottom: 1px solid var(--app-border);
		justify-content: space-between;
	}
	h2 {
		font-size: 17px;
		font-weight: 650;
		margin: 0;
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.dialog-content {
		padding: 20px;
		overflow: auto;
		min-height: 0;
		scrollbar-gutter: stable;
		overflow-wrap: anywhere;
	}
	footer {
		border-top: 1px solid var(--app-border);
		justify-content: flex-end;
		flex-wrap: wrap;
	}
	dialog.drawer {
		margin: 0 0 0 auto;
		width: min(520px, calc(100vw - 24px));
		height: 100dvh;
		max-height: 100dvh;
		border-radius: 12px 0 0 12px;
	}
	.drawer .dialog-frame {
		height: 100%;
		max-height: 100dvh;
	}
	.drawer .dialog-content {
		flex: 1;
	}
	dialog.image-dialog {
		width: min(1040px, calc(100vw - 32px));
	}
</style>
