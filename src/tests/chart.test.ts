import { describe, expect, it } from 'vitest';
import examples from '../../src-tauri/tests/fixtures/charts.json';
import { chartColors, chartOption, parseChart } from '$lib/utils/chart';
import { themeById } from '$lib/themes';

/** The block text of an example; `"x": "LABELS_<n>"` stands for n labels. */
function source(example: { source?: string; chart?: Record<string, unknown> }): string {
	if (example.source !== undefined) return example.source;
	const chart = { ...example.chart };
	const many = typeof chart.x === 'string' && /^LABELS_(\d+)$/.exec(chart.x);
	if (many) chart.x = Array.from({ length: Number(many[1]) }, (_, i) => `L${i}`);
	return JSON.stringify(chart);
}

describe('chart blocks', () => {
	it('are checked like on the backend', () => {
		expect(examples.length).toBeGreaterThan(30);
		for (const example of examples) {
			const result = parseChart(source(example));
			if (example.error === null) expect('error' in result ? result.error : null, example.name).toBeNull();
			else expect('error' in result ? result.error : '', example.name).toMatch(new RegExp(`^${example.error.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`));
		}
	});

	it('give a normalized chart', () => {
		const result = parseChart('{"type":"bar","x":[2020,2021],"series":[{"data":[1,null]},{"name":"B","type":"line","axis":"right","data":[2,3]}],"unit":"M"}');
		expect(result).toEqual({
			chart: {
				type: 'bar',
				title: '',
				x: ['2020', '2021'],
				y: [],
				series: [
					{ name: '', type: null, axis: 'left', data: [1, null] },
					{ name: 'B', type: 'line', axis: 'right', data: [2, 3] },
				],
				stacked: false,
				horizontal: false,
				unit: 'M',
				unitRight: '',
				xTitle: '',
				yTitle: '',
			},
		});
	});
});

describe('chart options', () => {
	const colors = chartColors('default-light');
	const option = (source: Record<string, unknown>) => {
		const parsed = parseChart(JSON.stringify(source));
		if ('error' in parsed) throw new Error(parsed.error);
		return chartOption(parsed.chart, colors) as any;
	};

	it('take validated series colors for the theme mode and text colors from the theme', () => {
		expect(colors.series.slice(0, 3)).toEqual(['#2a78d6', '#eb6834', '#1baf7a']);
		expect(chartColors('dracula').series[0]).toBe('#3987e5');
		expect(colors.text).toBe(themeById('default-light')!.colors['--app-text']);
		expect(chartColors('nord').muted).toBe(themeById('nord')!.colors['--app-muted']);
		expect(chartColors('unknown').series[0]).toBe('#2a78d6');
	});

	it('draw bars with rounded ends, units and a legend for several series', () => {
		const bars = option({ type: 'bar', title: 'Sales', x: ['a', 'b'], series: [{ name: 'A', data: [1, 2] }, { name: 'B', data: [3, null] }], unit: 'M', stacked: true });
		expect(bars.title.text).toBe('Sales');
		expect(bars.color).toEqual(colors.series);
		expect(bars.xAxis[0]).toMatchObject({ type: 'category', data: ['a', 'b'] });
		expect(bars.yAxis[0].type).toBe('value');
		expect(bars.yAxis[0].axisLabel.formatter(5)).toBe('5 M');
		expect(bars.series.map((s: any) => [s.type, s.name, s.stack])).toEqual([
			['bar', 'A', 'total'],
			['bar', 'B', 'total'],
		]);
		expect(bars.series[0].data).toEqual([1, 2]);
		expect(bars.series[1].itemStyle.borderRadius).toEqual([4, 4, 0, 0]);
		expect(bars.legend.show).toBe(true);
		expect(bars.tooltip.trigger).toBe('axis');
		expect(bars.tooltip.valueFormatter(2)).toBe('2 M');
		expect(option({ type: 'bar', x: ['a'], series: [{ data: [1] }] }).legend.show).toBe(false);
	});

	it('swap the axes of horizontal bars', () => {
		const bars = option({ type: 'bar', horizontal: true, x: ['a'], series: [{ data: [1] }] });
		expect(bars.yAxis[0]).toMatchObject({ type: 'category', data: ['a'] });
		expect(bars.xAxis[0].type).toBe('value');
		expect(bars.series[0].itemStyle.borderRadius).toEqual([0, 4, 4, 0]);
	});

	it('draw lines, areas and bars with a line on a second axis', () => {
		const lines = option({ type: 'line', x: ['a', 'b'], series: [{ data: [1, null] }] });
		expect(lines.series[0]).toMatchObject({ type: 'line', connectNulls: false, lineStyle: { width: 2 } });
		expect(lines.series[0].areaStyle).toBeUndefined();
		expect(option({ type: 'area', x: ['a'], series: [{ data: [1] }] }).series[0]).toMatchObject({ type: 'line', areaStyle: {} });

		const mixed = option({
			type: 'bar',
			x: ['Q1', 'Q2'],
			series: [{ name: 'Revenue', data: [10, 12] }, { name: 'Margin', type: 'line', axis: 'right', data: [20, 25] }],
			unit: '$M',
			unit_right: '%',
		});
		expect(mixed.yAxis).toHaveLength(2);
		expect(mixed.yAxis[1].axisLabel.formatter(20)).toBe('20 %');
		expect(mixed.series.map((s: any) => [s.type, s.yAxisIndex])).toEqual([
			['bar', 0],
			['line', 1],
		]);
	});

	it('draw pies, donuts, scatter, radar and heatmaps', () => {
		const pie = option({ type: 'pie', x: ['a', 'b'], series: [{ data: [3, 1] }] });
		expect(pie.series[0]).toMatchObject({ type: 'pie', data: [{ name: 'a', value: 3 }, { name: 'b', value: 1 }] });
		expect(pie.xAxis).toBeUndefined();
		expect(pie.legend.show).toBe(true);
		expect(pie.tooltip.trigger).toBe('item');
		expect(option({ type: 'donut', x: ['a'], series: [{ data: [1] }] }).series[0].radius).toEqual(['45%', '70%']);

		const scatter = option({ type: 'scatter', series: [{ data: [[1, 2]] }], x_title: 'Weight', y_title: 'MPG' });
		expect(scatter.xAxis[0]).toMatchObject({ type: 'value', name: 'Weight' });
		expect(scatter.yAxis[0]).toMatchObject({ type: 'value', name: 'MPG' });
		expect(scatter.series[0]).toMatchObject({ type: 'scatter', data: [[1, 2]], symbolSize: 8 });

		const radar = option({ type: 'radar', x: ['a', 'b', 'c'], series: [{ name: 'A', data: [1, 2, 3] }] });
		expect(radar.radar.indicator.map((i: any) => i.name)).toEqual(['a', 'b', 'c']);
		expect(radar.series[0].data).toEqual([{ name: 'A', value: [1, 2, 3] }]);

		const heat = option({ type: 'heatmap', x: ['Mon', 'Tue'], y: ['AM', 'PM'], series: [{ data: [[1, 2], [null, 4]] }] });
		expect(heat.yAxis[0]).toMatchObject({ type: 'category', data: ['AM', 'PM'], inverse: true });
		expect(heat.series[0].data).toEqual([[0, 0, 1], [1, 0, 2], [1, 1, 4]]);
		expect(heat.visualMap).toMatchObject({ min: 1, max: 4, inRange: { color: colors.ramp } });
	});
});
