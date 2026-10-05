<script lang="ts">
	import { t } from '$lib/i18n';
	import { XIcon, CircleCheckIcon, CircleAlertIcon, InfoIcon } from '@lucide/svelte';
	import { toasts, dismissToast } from '$lib/stores/toasts';
	const icons = { info: InfoIcon, success: CircleCheckIcon, error: CircleAlertIcon };
</script>

<!-- Errors are announced assertively, other messages politely. -->
<div class="toaster" aria-live="polite">
	{#each $toasts as item (item.id)}
		{@const Icon = icons[item.tone]}
		<div class="toast {item.tone}" role={item.tone === 'error' ? 'alert' : undefined}>
			<span class="icon"><Icon size={16} /></span>
			<p>{item.message}</p>
			{#if item.action}{@const action = item.action}<button
					class="button ghost sm toast-action"
					onclick={() => {
						dismissToast(item.id);
						void action.run();
					}}>{action.label}</button
				>{/if}
			<button class="icon-button ghost sm" aria-label={t('common.dismiss')} onclick={() => dismissToast(item.id)}
				><XIcon size={14} /></button
			>
		</div>
	{/each}
</div>

<style>
	.toaster {
		position: fixed;
		right: 16px;
		bottom: 52px;
		z-index: 100;
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 8px;
		max-width: min(420px, calc(100vw - 32px));
		pointer-events: none;
	}
	.toast {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 10px 8px 10px 12px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		background: var(--app-panel);
		box-shadow: var(--app-shadow-popover);
		font-size: var(--app-text-md);
		pointer-events: auto;
		animation: enter 0.15s ease-out;
	}
	.icon {
		padding-top: 2px;
		color: var(--app-accent);
	}
	.success .icon {
		color: var(--app-success);
	}
	.error {
		border-color: color-mix(in srgb, var(--app-danger) 50%, var(--app-border));
	}
	.error .icon {
		color: var(--app-danger);
	}
	p {
		flex: 1;
		min-width: 0;
		padding-top: 1px;
		overflow-wrap: anywhere;
	}
	.toast-action {
		color: var(--app-accent);
		margin-top: -3px;
	}
	@keyframes enter {
		from {
			opacity: 0;
			transform: translateY(6px);
		}
	}
</style>
