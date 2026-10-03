<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { Tone } from '$lib/utils/status';
	let { tone = 'neutral', children }: { tone?: Tone; children: Snippet } = $props();
</script>

<span class="badge {tone}">{@render children()}</span>

<style>
	.badge {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 2px 9px;
		border-radius: var(--app-radius-pill);
		background: var(--app-subtle);
		color: var(--app-muted);
		font-size: var(--app-text-xs);
		font-weight: 600;
		line-height: 1.6;
		white-space: nowrap;
	}
	.accent {
		color: var(--app-accent);
		background: color-mix(in srgb, var(--app-accent) 14%, transparent);
	}
	.success {
		color: var(--app-success);
		background: color-mix(in srgb, var(--app-success) 14%, transparent);
	}
	.warning {
		color: var(--app-warning);
		background: color-mix(in srgb, var(--app-warning) 14%, transparent);
	}
	.danger {
		color: var(--app-danger);
		background: color-mix(in srgb, var(--app-danger) 14%, transparent);
	}
</style>
