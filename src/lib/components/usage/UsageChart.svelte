<script lang="ts">
	import type { Chart } from '$lib/utils/chart';
	import { chartColors } from '$lib/utils/chart';
	import { diagramLabel } from '$lib/utils/diagrams';
	import { appliedTheme } from '$lib/stores/ui';

	let { chart }: { chart: Chart } = $props();
	let element: HTMLDivElement;

	// Drawn with the research answers' chart renderer, loaded on first use; again on theme change.
	$effect(() => {
		const current = chart;
		const colors = chartColors($appliedTheme);
		let dispose: (() => void) | null = null;
		let live = true;
		import('$lib/utils/chartRender').then(({ drawChart }) => {
			if (!live) return;
			element.replaceChildren();
			dispose = drawChart(element, current, colors).dispose;
		});
		return () => {
			live = false;
			dispose?.();
		};
	});
</script>

<figure class="usage-chart" role="img" aria-label={diagramLabel('', chart)}>
	<div bind:this={element} class="view"></div>
</figure>

<style>
	.usage-chart {
		margin: 0;
		padding: 12px;
		background: var(--app-panel);
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		min-width: 0;
	}
	.view {
		width: 100%;
	}
</style>
