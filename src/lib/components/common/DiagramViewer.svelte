<script lang="ts">
	import { t } from '$lib/i18n';
	import DOMPurify from 'dompurify';
	import { MaximizeIcon, ZoomInIcon, ZoomOutIcon } from '@lucide/svelte';
	import Dialog from './Dialog.svelte';
	import { tooltip } from '$lib/actions/tooltip';

	let { title, svg, onclose }: { title: string; svg: string; onclose: () => void } = $props();

	const STEPS = [0.5, 0.75, 1, 1.25, 1.5, 2, 3, 4];
	let zoom = $state(1);
	let area: HTMLDivElement;
	const markup = $derived(DOMPurify.sanitize(svg, { USE_PROFILES: { svg: true, svgFilters: true }, ADD_TAGS: ['style'] }));

	function step(direction: 1 | -1) {
		const next = direction > 0 ? STEPS.find((s) => s > zoom + 0.001) : [...STEPS].reverse().find((s) => s < zoom - 0.001);
		if (next) zoom = next;
	}

	// Dragging pans the picture when it is larger than the window.
	let drag = $state<{ x: number; y: number; left: number; top: number } | null>(null);
	function down(event: PointerEvent) {
		if (event.button !== 0) return;
		drag = { x: event.clientX, y: event.clientY, left: area.scrollLeft, top: area.scrollTop };
		area.setPointerCapture(event.pointerId);
	}
	function move(event: PointerEvent) {
		if (!drag) return;
		area.scrollLeft = drag.left - (event.clientX - drag.x);
		area.scrollTop = drag.top - (event.clientY - drag.y);
	}
	function wheel(event: WheelEvent) {
		if (!event.ctrlKey) return;
		event.preventDefault();
		step(event.deltaY < 0 ? 1 : -1);
	}
</script>

<Dialog {title} variant="image" {onclose}>
	<div class="viewer">
		<div class="tools">
			<button type="button" class="button" aria-label={t('research.zoomOut')} use:tooltip={{ text: t('research.zoomOut') }} onclick={() => step(-1)} disabled={zoom <= STEPS[0]}>
				<ZoomOutIcon size={16} />
			</button>
			<span class="zoom" role="status">{Math.round(zoom * 100)}%</span>
			<button type="button" class="button" aria-label={t('research.zoomIn')} use:tooltip={{ text: t('research.zoomIn') }} onclick={() => step(1)} disabled={zoom >= STEPS[STEPS.length - 1]}>
				<ZoomInIcon size={16} />
			</button>
			<button type="button" class="button" aria-label={t('research.zoomFit')} use:tooltip={{ text: t('research.zoomFit') }} onclick={() => (zoom = 1)}>
				<MaximizeIcon size={16} />
			</button>
		</div>
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			bind:this={area}
			class="area"
			class:dragging={drag}
			onpointerdown={down}
			onpointermove={move}
			onpointerup={() => (drag = null)}
			onpointercancel={() => (drag = null)}
			onwheel={wheel}
		>
			<div class="picture" style:width={`${zoom * 100}%`}>{@html markup}</div>
		</div>
	</div>
</Dialog>

<style>
	.viewer {
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-height: 0;
		width: min(92vw, 1400px);
		max-width: 100%;
	}
	.tools {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.zoom {
		min-width: 4.5ch;
		text-align: center;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		font-variant-numeric: tabular-nums;
	}
	.area {
		overflow: auto;
		max-height: 78vh;
		background: var(--app-panel);
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		cursor: grab;
		touch-action: pan-x pan-y pinch-zoom;
	}
	.area.dragging {
		cursor: grabbing;
	}
	.picture {
		min-width: 100%;
		padding: 12px;
		box-sizing: border-box;
	}
	.picture :global(svg) {
		display: block;
		width: 100% !important;
		max-width: none !important;
		height: auto;
	}
</style>
