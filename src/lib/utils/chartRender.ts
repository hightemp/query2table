/**
 * Draws ```chart blocks with ECharts (only the chart kinds and parts we use, SVG output).
 * Loaded on demand (`import()`).
 */
import * as echarts from 'echarts/core';
import { BarChart, HeatmapChart, LineChart, PieChart, RadarChart, ScatterChart } from 'echarts/charts';
import { GridComponent, LegendComponent, TitleComponent, TooltipComponent, VisualMapComponent } from 'echarts/components';
import { SVGRenderer } from 'echarts/renderers';
import { chartOption, type Chart, type ChartColors } from './chart';

echarts.use([
	BarChart,
	HeatmapChart,
	LineChart,
	PieChart,
	RadarChart,
	ScatterChart,
	GridComponent,
	LegendComponent,
	TitleComponent,
	TooltipComponent,
	VisualMapComponent,
	SVGRenderer,
]);

export interface ChartView {
	/** The chart as a standalone SVG document. */
	svg(): string;
	dispose(): void;
}

/** Height that fits the chart's kind and size. */
function heightOf(chart: Chart): number {
	if (chart.type === 'heatmap') return Math.min(640, 120 + chart.y.length * 28);
	if (chart.type === 'bar' && chart.horizontal) return Math.min(640, 100 + chart.x.length * chart.series.length * 22);
	if (chart.type === 'radar') return 360;
	return chart.type === 'pie' || chart.type === 'donut' ? 300 : 320;
}

/** Draws the chart into `element` and keeps it sized to the element's width. */
export function drawChart(element: HTMLElement, chart: Chart, colors: ChartColors): ChartView {
	element.style.height = `${heightOf(chart)}px`;
	const instance = echarts.init(element, null, { renderer: 'svg' });
	// No animation: the drawn picture is final at once, for saving too.
	instance.setOption({ ...chartOption(chart, colors), animation: false });
	const resize = new ResizeObserver(() => instance.resize());
	resize.observe(element);
	return {
		svg: () => instance.renderToSVGString(),
		dispose: () => {
			resize.disconnect();
			instance.dispose();
		},
	};
}
