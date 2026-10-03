<script lang="ts">
	import { settings } from '$lib/stores/settings';
	import { faviconUrl, siteHue } from '$lib/utils/linkResults';
	let { domain, host, size = 16 }: { domain: string; host: string; size?: number } = $props();
	let failed = $state(false);
	$effect(() => {
		void host;
		failed = false;
	});
	let enabled = $derived($settings.get('show_site_icons') !== 'false');
	let letter = $derived((domain.match(/[\p{L}\p{N}]/u)?.[0] ?? '?').toUpperCase());
</script>

{#if enabled && host && !failed}<img
		class="site-icon"
		src={faviconUrl(host)}
		alt=""
		width={size}
		height={size}
		loading="lazy"
		referrerpolicy="no-referrer"
		onerror={() => (failed = true)}
	/>{:else}<span
		class="site-letter"
		aria-hidden="true"
		style={`--size:${size}px;--hue:${siteHue(domain)}`}>{letter}</span
	>{/if}

<style>
	.site-icon {
		flex-shrink: 0;
		border-radius: 3px;
	}
	.site-letter {
		display: inline-grid;
		place-items: center;
		flex-shrink: 0;
		width: var(--size);
		height: var(--size);
		border-radius: 50%;
		background: hsl(var(--hue) 55% 45%);
		color: #fff;
		font-size: calc(var(--size) * 0.6);
		font-weight: 700;
		line-height: 1;
	}
</style>
