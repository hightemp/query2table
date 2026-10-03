<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	let {
		icon: Icon,
		title,
		children,
		action,
		role,
	}: {
		icon?: Component<{ size?: number }>;
		title?: string;
		children?: Snippet;
		action?: Snippet;
		role?: 'status';
	} = $props();
</script>

<div class="empty-state" {role}>
	{#if Icon}<span class="icon"><Icon size={22} /></span>{/if}
	{#if title}<p class="title">{title}</p>{/if}
	{#if children}<div class="body">{@render children()}</div>{/if}
	{#if action}<div class="action">{@render action()}</div>{/if}
</div>

<style>
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 32px 20px;
		text-align: center;
		color: var(--app-muted);
		font-size: var(--app-text-md);
	}
	.icon {
		display: grid;
		place-items: center;
		width: 44px;
		height: 44px;
		margin-bottom: 4px;
		border-radius: var(--app-radius-pill);
		background: var(--app-subtle);
	}
	.title {
		color: var(--app-text);
		font-size: var(--app-text-base);
		font-weight: 600;
	}
	.body {
		max-width: 48ch;
	}
	.action {
		margin-top: 8px;
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		justify-content: center;
	}
</style>
