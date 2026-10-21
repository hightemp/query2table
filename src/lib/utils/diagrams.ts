/**
 * Mermaid diagrams and charts in research answers: ```mermaid and ```chart code blocks become
 * placeholders showing their source, which `Markdown.svelte` draws with `mermaidRender.ts` and
 * `chartRender.ts`.
 */
import { t } from '$lib/i18n';
import type { Chart } from './chart';

export type DiagramKind = 'mermaid' | 'chart';

function escape(text: string): string {
	return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/** The source of a placeholder made by `diagramPlaceholder`. */
export function diagramSource(figure: Element): string {
	return figure.querySelector('.diagram-source code')?.textContent ?? '';
}

/** The placeholder of a code block in `language`, or null for ordinary code. */
export function diagramPlaceholder(language: string | undefined, source: string): string | null {
	const kind = language?.trim().split(/\s+/)[0]?.toLowerCase();
	if (kind !== 'mermaid' && kind !== 'chart') return null;
	// The source stays element text: DOMPurify drops attributes holding "-->", common in diagrams.
	return `<figure class="diagram" data-diagram="${kind}"><pre class="diagram-source"><code>${escape(source)}</code></pre></figure>`;
}

// Node texts: A["…"], B[…], C(…), D{…}, E((…)).
const NODE_TEXT = /[[({]+\s*"?([^\])}"]+?)"?\s*[\])}]+/g;

/** Text for screen readers: the chart's title and series, or the diagram's kind and labels. */
export function diagramLabel(source: string, chart?: Chart): string {
	if (chart) {
		const kind = t(`research.chartType.${chart.type}`);
		const names = chart.series.map((s) => s.name).filter(Boolean);
		const body = names.length ? `${kind}: ${names.join(', ')}` : kind;
		return chart.title ? `${chart.title}. ${body}` : body;
	}
	const [first = '', ...rest] = source.trim().split('\n');
	const kind = first.trim().split(/\s+/)[0];
	const labels = [...rest.join('\n').matchAll(NODE_TEXT)].map((m) => m[1].trim()).filter(Boolean);
	const name = kind ? `${t('research.diagram')} (${kind})` : t('research.diagram');
	return labels.length ? `${name}: ${[...new Set(labels)].join(', ')}` : name;
}

/** A file name from the title, without characters file systems refuse. */
export function diagramFileName(title: string, extension: string): string {
	const name = title.replace(/[\\/:*?"<>|]+/g, ' ').replace(/\s+/g, ' ').trim().slice(0, 80);
	return `${name || t('research.diagram')}.${extension}`;
}
