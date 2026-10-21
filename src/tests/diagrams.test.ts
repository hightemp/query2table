import { describe, expect, it } from 'vitest';
import DOMPurify from 'dompurify';
import { diagramFileName, diagramLabel, diagramPlaceholder, diagramSource } from '$lib/utils/diagrams';
import { parseChart } from '$lib/utils/chart';
import { readableAnswer } from '$lib/utils/research';

describe('diagram blocks', () => {
	it('become placeholders that show their source until drawn', () => {
		const html = diagramPlaceholder('mermaid', 'graph LR\n  A["<b>x</b>"] --> B');
		expect(html).toContain('class="diagram"');
		expect(html).toContain('data-diagram="mermaid"');
		expect(html).toContain('<code>graph LR\n  A["&lt;b&gt;x&lt;/b&gt;"] --&gt; B</code>');
		// The source survives sanitizing ("-->" is dropped from attributes) and reads back as written.
		const figure = document.createElement('div');
		figure.innerHTML = DOMPurify.sanitize(html!);
		expect(diagramSource(figure.firstElementChild!)).toBe('graph LR\n  A["<b>x</b>"] --> B');
		expect(diagramPlaceholder('Chart', '{}')).toContain('data-diagram="chart"');
		expect(diagramPlaceholder('python', 'print(1)')).toBeNull();
		expect(diagramPlaceholder(undefined, 'x')).toBeNull();
	});

	it('are named by their labels for screen readers', () => {
		expect(diagramLabel('flowchart LR\n  A["Ask a question"] --> B[Read pages] --> C(Answer)')).toBe(
			'Diagram (flowchart): Ask a question, Read pages, Answer'
		);
		expect(diagramLabel('sequenceDiagram\n  Alice->>Bob: Hi')).toBe('Diagram (sequenceDiagram)');
		const parsed = parseChart('{"type":"bar","title":"Sales","x":["a"],"series":[{"name":"A","data":[1]},{"name":"B","data":[2]}]}');
		if ('error' in parsed) throw new Error(parsed.error);
		expect(diagramLabel('', parsed.chart)).toBe('Sales. Bar chart: A, B');
		expect(diagramLabel('', { ...parsed.chart, title: '', series: [{ ...parsed.chart.series[0], name: '' }] })).toBe('Bar chart');
	});

	it('are saved under their title', () => {
		expect(diagramFileName('Sales: 2024/25', 'svg')).toBe('Sales 2024 25.svg');
		expect(diagramFileName('', 'png')).toBe('Diagram.png');
	});

	it('keep their source in copied text', () => {
		const text = readableAnswer('Intro\n\n```chart\n{"type":"pie"}\n```\n\n```mermaid\ngraph LR\n```', []);
		expect(text).toContain('{"type":"pie"}');
		expect(text).toContain('graph LR');
	});
});
