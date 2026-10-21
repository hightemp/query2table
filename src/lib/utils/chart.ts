/**
 * The ```chart block of research answers: a small JSON scheme drawn with ECharts. Mirrors
 * `check` in `src-tauri/src/roles/chart.rs` (same order of checks and messages, held together
 * by `src-tauri/tests/fixtures/charts.json`).
 */

import type { EChartsOption } from 'echarts';
import { DEFAULT_THEME, themeById, type ThemeMode } from '$lib/themes';

export const CHART_TYPES = ['bar', 'line', 'area', 'pie', 'donut', 'scatter', 'radar', 'heatmap'] as const;
export type ChartType = (typeof CHART_TYPES)[number];

export interface ChartSeries {
	name: string;
	/** Own kind in a bar or line chart (bars with a line); null follows the chart. */
	type: 'bar' | 'line' | null;
	axis: 'left' | 'right';
	/** Values per label; `[x, y]` points in a scatter chart; rows of values in a heatmap. */
	data: (number | null)[] | [number, number][] | (number | null)[][];
}

export interface Chart {
	type: ChartType;
	title: string;
	/** Category labels: bars, lines, pie slices, radar axes, heatmap columns. */
	x: string[];
	/** Heatmap rows. */
	y: string[];
	series: ChartSeries[];
	stacked: boolean;
	horizontal: boolean;
	unit: string;
	unitRight: string;
	xTitle: string;
	yTitle: string;
}

const MAX_SERIES = 12;
const MAX_LABELS = 200;
const MAX_POINTS = 2000;
const MAX_CELLS = 2500;

type Json = Record<string, unknown>;

class ChartError extends Error {}

function plural(count: number, word: string): string {
	return `${count} ${word}${count === 1 ? '' : 's'}`;
}

const isObject = (value: unknown): value is Json => typeof value === 'object' && value !== null && !Array.isArray(value);
const isNumber = (value: unknown): value is number => typeof value === 'number';
const absent = (value: unknown) => value === undefined || value === null;

function text(chart: Json, field: string): string {
	const value = chart[field];
	if (absent(value)) return '';
	if (typeof value !== 'string') throw new ChartError(`"${field}" must be text`);
	return value;
}

function flag(chart: Json, field: string): boolean {
	const value = chart[field];
	if (absent(value)) return false;
	if (typeof value !== 'boolean') throw new ChartError(`"${field}" must be true or false`);
	return value;
}

function labels(chart: Json, field: string, missing: string): string[] {
	const list = chart[field];
	if (!Array.isArray(list) || !list.length) throw new ChartError(missing);
	if (list.some((label) => typeof label !== 'string' && !isNumber(label))) throw new ChartError(`"${field}" must be a list of text labels`);
	if (list.length > MAX_LABELS) throw new ChartError(`at most ${MAX_LABELS} labels in "${field}"`);
	return list.map(String);
}

function check(source: string): Chart {
	let value: unknown;
	try {
		value = JSON.parse(source);
	} catch {
		throw new ChartError('not valid JSON');
	}
	if (!isObject(value)) throw new ChartError('must be a JSON object');
	const kind = value.type;
	if (typeof kind !== 'string') throw new ChartError('"type" is missing');
	if (!(CHART_TYPES as readonly string[]).includes(kind)) throw new ChartError(`unknown type "${kind}"; use one of ${CHART_TYPES.join(', ')}`);
	const type = kind as ChartType;
	const [title, xTitle, yTitle, unit, unitRight] = ['title', 'x_title', 'y_title', 'unit', 'unit_right'].map((field) => text(value, field));
	const [stacked, horizontal] = ['stacked', 'horizontal'].map((field) => flag(value, field));
	const items = value.series;
	if (!Array.isArray(items) || !items.length) throw new ChartError('"series" must list at least one series');
	if (items.length > MAX_SERIES) throw new ChartError(`at most ${MAX_SERIES} series`);
	const x = type === 'scatter' ? [] : labels(value, 'x', '"x" must list the labels');
	const y = type === 'heatmap' ? labels(value, 'y', '"y" must list the row labels') : [];
	if ((type === 'pie' || type === 'donut') && items.length !== 1) throw new ChartError('a pie chart takes one series');
	if (type === 'heatmap' && items.length !== 1) throw new ChartError('a heatmap takes one series');
	if (type === 'heatmap' && x.length * y.length > MAX_CELLS) throw new ChartError(`at most ${MAX_CELLS} cells`);
	if (type === 'radar' && x.length < 3) throw new ChartError('a radar chart needs at least 3 labels in "x"');

	const mixed = type === 'bar' || type === 'line';
	const series = items.map((item, index): ChartSeries => {
		const number = index + 1;
		if (!isObject(item)) throw new ChartError(`series ${number} must be an object with "data"`);
		if (!absent(item.name) && typeof item.name !== 'string') throw new ChartError(`series ${number}: "name" must be text`);
		const name = typeof item.name === 'string' ? item.name.trim() : '';
		const label = name ? `series "${name}"` : `series ${number}`;
		const data = item.data;
		if (!Array.isArray(data)) throw new ChartError(`${label}: "data" must be a list`);
		if (!absent(item.type)) {
			if (!mixed) throw new ChartError('series types are only for bar and line charts');
			if (item.type !== 'bar' && item.type !== 'line') throw new ChartError(`${label}: "type" must be bar or line`);
		}
		if (!absent(item.axis)) {
			if (!mixed) throw new ChartError('"axis" is only for bar and line charts');
			if (item.axis !== 'left' && item.axis !== 'right') throw new ChartError(`${label}: "axis" must be left or right`);
		}
		if (type === 'scatter') {
			if (data.length > MAX_POINTS) throw new ChartError(`${label}: at most ${MAX_POINTS} points`);
			data.forEach((point, i) => {
				if (!Array.isArray(point) || point.length !== 2 || !point.every(isNumber)) {
					throw new ChartError(`${label}: point ${i + 1} must be [x, y] numbers`);
				}
			});
		} else if (type === 'heatmap') {
			if (data.length !== y.length || !data.every((row) => Array.isArray(row) && row.length === x.length)) {
				throw new ChartError(`${label} must have ${plural(y.length, 'row')} of ${plural(x.length, 'value')}`);
			}
			(data as unknown[][]).forEach((row, r) =>
				row.forEach((cell, c) => {
					if (cell !== null && !isNumber(cell)) throw new ChartError(`${label}: row ${r + 1}, value ${c + 1} is not a number`);
				})
			);
		} else {
			if (data.length !== x.length) throw new ChartError(`${label} has ${plural(data.length, 'value')} for ${plural(x.length, 'label')}`);
			const pie = type === 'pie' || type === 'donut';
			data.forEach((cell, i) => {
				if (!isNumber(cell) && !(cell === null && !pie)) throw new ChartError(`${label}: value ${i + 1} is not a number`);
				if (pie && cell < 0) throw new ChartError(`${label}: value ${i + 1} is negative`);
			});
		}
		return {
			name,
			type: (item.type as ChartSeries['type'] | undefined) ?? null,
			axis: item.axis === 'right' ? 'right' : 'left',
			data: data as ChartSeries['data'],
		};
	});
	return { type, title, x, y, series, stacked, horizontal, unit, unitRight, xTitle, yTitle };
}

/** The chart of a ```chart block, or what is wrong with it. */
export function parseChart(source: string): { chart: Chart } | { error: string } {
	try {
		return { chart: check(source) };
	} catch (error) {
		if (error instanceof ChartError) return { error: error.message };
		throw error;
	}
}

/** Colors of a chart in a theme. */
export interface ChartColors {
	/** Categorical series colors in their fixed order. */
	series: string[];
	text: string;
	muted: string;
	grid: string;
	surface: string;
	/** Heatmap ramp, from near zero to the largest value. */
	ramp: [string, string];
}

// A categorical palette validated for color-vision deficiencies, stepped for light and dark surfaces.
const SERIES: Record<ThemeMode, string[]> = {
	light: ['#2a78d6', '#eb6834', '#1baf7a', '#eda100', '#e87ba4', '#008300', '#6250d6', '#e34948'],
	dark: ['#3987e5', '#d95926', '#199e70', '#c98500', '#d55181', '#008300', '#9085e9', '#e66767'],
};
const RAMP: Record<ThemeMode, [string, string]> = { light: ['#cde2fb', '#1c5cab'], dark: ['#104281', '#9ec5f4'] };

export function chartColors(themeId: string): ChartColors {
	const theme = themeById(themeId) ?? themeById(DEFAULT_THEME.light)!;
	const colors = theme.colors;
	return {
		series: SERIES[theme.mode],
		text: colors['--app-text'],
		muted: colors['--app-muted'],
		grid: colors['--app-border'],
		surface: colors['--app-panel'],
		ramp: RAMP[theme.mode],
	};
}

const withUnit = (unit: string) => (value: unknown) => (unit && value !== null && value !== undefined && value !== '-' ? `${value} ${unit}` : String(value ?? '–'));

/** ECharts options for a checked chart. */
export function chartOption(chart: Chart, colors: ChartColors): EChartsOption {
	const muted = { color: colors.muted };
	const axisLine = { lineStyle: { color: colors.grid } };
	const splitLine = { lineStyle: { color: colors.grid, opacity: 0.6 } };
	// Axis titles sit along the middle of their axis, clear of the plot's edge.
	const titled = (name: string, vertical: boolean) =>
		name ? { name, nameLocation: 'middle', nameGap: vertical ? 48 : 28, nameTextStyle: muted } : { name };
	const category = (data: string[], name: string, vertical = false) => ({
		type: 'category',
		data,
		...titled(name, vertical),
		axisLabel: muted,
		axisLine,
		axisTick: { show: false },
	});
	const value = (name: string, unit: string, vertical = true) => ({
		type: 'value',
		...titled(name, vertical),
		scale: chart.type === 'scatter',
		// Scatter points keep clear of the plot's edges.
		...(chart.type === 'scatter' ? { boundaryGap: ['5%', '5%'] } : {}),
		axisLabel: { ...muted, formatter: withUnit(unit) },
		axisLine,
		splitLine,
	});
	const many = chart.type === 'pie' || chart.type === 'donut' || chart.series.length > 1;
	const base: Record<string, unknown> = {
		color: colors.series,
		backgroundColor: 'transparent',
		textStyle: { color: colors.text, fontFamily: 'inherit' },
		title: { text: chart.title, left: 'center', textStyle: { color: colors.text, fontSize: 14, fontWeight: 600 } },
		legend: { show: many, bottom: 0, type: 'scroll', textStyle: { color: colors.text }, pageTextStyle: muted },
		tooltip: {
			trigger: chart.type === 'bar' || chart.type === 'line' || chart.type === 'area' ? 'axis' : 'item',
			valueFormatter: withUnit(chart.unit),
			backgroundColor: colors.surface,
			borderColor: colors.grid,
			textStyle: { color: colors.text },
		},
		// The plot shrinks to keep axis labels and titles inside the picture.
		grid: { left: 8, right: 8, top: chart.title ? 40 : 16, bottom: many ? 36 : 8, outerBoundsMode: 'same', outerBoundsContain: 'all' },
	};

	switch (chart.type) {
		case 'pie':
		case 'donut':
			return {
				...base,
				series: [
					{
						type: 'pie',
						radius: chart.type === 'donut' ? ['45%', '70%'] : '70%',
						center: ['50%', '50%'],
						itemStyle: { borderColor: colors.surface, borderWidth: 2 },
						label: { color: colors.text },
						data: chart.x.map((name, i) => ({ name, value: (chart.series[0].data as number[])[i] })),
					},
				],
			} as EChartsOption;
		case 'radar':
			return {
				...base,
				radar: { radius: '58%', center: ['50%', '52%'], indicator: chart.x.map((name) => ({ name })), axisName: muted, axisLine, splitLine, splitArea: { show: false } },
				series: [
					{
						type: 'radar',
						lineStyle: { width: 2 },
						symbolSize: 6,
						data: chart.series.map((s, i) => ({ name: s.name || `${i + 1}`, value: s.data })),
					},
				],
			} as EChartsOption;
		case 'heatmap': {
			const cells: [number, number, number][] = [];
			(chart.series[0].data as (number | null)[][]).forEach((row, r) =>
				row.forEach((cell, c) => {
					if (cell !== null) cells.push([c, r, cell]);
				})
			);
			const values = cells.map((cell) => cell[2]);
			return {
				...base,
				tooltip: { ...(base.tooltip as object), trigger: 'item' },
				grid: { ...(base.grid as object), bottom: 64 },
				xAxis: [category(chart.x, chart.xTitle)],
				// Rows read top to bottom in the order given.
				yAxis: [{ ...category(chart.y, chart.yTitle, true), inverse: true }],
				visualMap: {
					min: values.length ? Math.min(...values) : 0,
					max: values.length ? Math.max(...values) : 1,
					calculable: true,
					orient: 'horizontal',
					left: 'center',
					bottom: 0,
					itemHeight: 120,
					textStyle: muted,
					inRange: { color: colors.ramp },
				},
				series: [{ type: 'heatmap', name: chart.series[0].name, data: cells, itemStyle: { borderColor: colors.surface, borderWidth: 2 } }],
			} as EChartsOption;
		}
		case 'scatter':
			return {
				...base,
				xAxis: [value(chart.xTitle, '', false)],
				yAxis: [value(chart.yTitle, chart.unit)],
				series: chart.series.map((s) => ({ type: 'scatter', name: s.name, data: s.data, symbolSize: 8 })),
			} as EChartsOption;
	}

	// Bars, lines and areas share a category axis.
	const horizontal = chart.type === 'bar' && chart.horizontal;
	const right = chart.series.some((s) => s.axis === 'right');
	const values = [value(chart.yTitle, chart.unit, !horizontal), ...(right ? [value('', chart.unitRight, !horizontal)] : [])];
	const labels = [category(chart.x, chart.xTitle, horizontal)];
	return {
		...base,
		xAxis: horizontal ? values : labels,
		yAxis: horizontal ? labels : values,
		series: chart.series.map((s) => {
			const kind = s.type ?? (chart.type === 'bar' ? 'bar' : 'line');
			const common = { name: s.name, data: s.data, yAxisIndex: s.axis === 'right' ? 1 : 0, stack: chart.stacked ? 'total' : undefined };
			if (kind === 'bar') {
				return { ...common, type: 'bar', barMaxWidth: 48, itemStyle: { borderRadius: horizontal ? [0, 4, 4, 0] : [4, 4, 0, 0] } };
			}
			return {
				...common,
				type: 'line',
				connectNulls: false,
				lineStyle: { width: 2 },
				symbolSize: 6,
				...(chart.type === 'area' ? { areaStyle: { opacity: 0.3 } } : {}),
			};
		}),
	} as EChartsOption;
}
