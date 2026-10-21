/**
 * Draws Mermaid diagrams as SVG in the app's theme. Loaded on demand (`import()`), since Mermaid
 * is large; its diagram kinds load in turn only when an answer uses them.
 */
import mermaid from 'mermaid';
import DOMPurify from 'dompurify';
import { DEFAULT_THEME, themeById } from '$lib/themes';

let configured = '';
let drawn = 0;

function configure(themeId: string) {
	if (configured === themeId) return;
	const theme = themeById(themeId) ?? themeById(DEFAULT_THEME.light)!;
	const c = theme.colors;
	mermaid.initialize({
		startOnLoad: false,
		// No scripts, links or HTML from the diagram; labels are SVG text.
		securityLevel: 'strict',
		htmlLabels: false,
		flowchart: { htmlLabels: false },
		theme: 'base',
		darkMode: theme.mode === 'dark',
		fontFamily: getComputedStyle(document.body).fontFamily,
		themeVariables: {
			darkMode: theme.mode === 'dark',
			background: c['--app-panel'],
			primaryColor: c['--app-subtle'],
			primaryTextColor: c['--app-text'],
			primaryBorderColor: c['--app-accent'],
			secondaryColor: c['--app-bg'],
			tertiaryColor: c['--app-panel'],
			mainBkg: c['--app-subtle'],
			nodeBorder: c['--app-accent'],
			lineColor: c['--app-muted'],
			textColor: c['--app-text'],
			titleColor: c['--app-text'],
			edgeLabelBackground: c['--app-panel'],
			clusterBkg: c['--app-bg'],
			clusterBorder: c['--app-border'],
			noteBkgColor: c['--app-subtle'],
			noteTextColor: c['--app-text'],
			noteBorderColor: c['--app-border'],
			actorBkg: c['--app-subtle'],
			actorBorder: c['--app-accent'],
			actorTextColor: c['--app-text'],
			signalColor: c['--app-text'],
			signalTextColor: c['--app-text'],
		},
	});
	configured = themeId;
}

/** SVG markup of a diagram; rejects when the source does not parse. */
export async function drawMermaid(source: string, themeId: string): Promise<string> {
	configure(themeId);
	// Parsing first keeps Mermaid from leaving its error picture in the page.
	await mermaid.parse(source);
	const id = `answer-diagram-${++drawn}`;
	try {
		const { svg } = await mermaid.render(id, source);
		return DOMPurify.sanitize(svg, { USE_PROFILES: { svg: true, svgFilters: true }, ADD_TAGS: ['style'] });
	} finally {
		document.getElementById(id)?.remove();
		document.getElementById(`d${id}`)?.remove();
	}
}
