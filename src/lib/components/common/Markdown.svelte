<script lang="ts">
	import { t } from '$lib/i18n';
	import { Renderer } from 'marked';
	import DOMPurify from 'dompurify';
	import { openExternal } from '$lib/utils/links';
	import { toast } from '$lib/stores/toasts';
	import { showContextMenu } from '$lib/stores/contextMenu';
	import { linkMenuItems } from '$lib/utils/linkMenu';
	import { webUrl } from '$lib/utils/values';
	import { isAttachmentUrl, plainInline, uniqueSlugger } from '$lib/utils/research';
	import { showAttachment } from '$lib/stores/attachmentPreview';
	import { markdown } from '$lib/utils/math';
	import { displayHost } from '$lib/utils/hosts';
	import { diagramLabel, diagramPlaceholder, diagramSource } from '$lib/utils/diagrams';
	import { chartColors, parseChart } from '$lib/utils/chart';
	import { appliedTheme } from '$lib/stores/ui';
	import { themeById } from '$lib/themes';
	import { copyWithToast } from '$lib/utils/linkMenu';
	import { menuPointFor } from './ContextMenu.svelte';
	import { CodeIcon, FileImageIcon, ImageIcon, Maximize2Icon } from '@lucide/svelte';
	import { tooltip } from '$lib/actions/tooltip';
	import DiagramViewer from './DiagramViewer.svelte';
	import type { ResearchImage } from '$lib/types';
	let {
		content,
		idPrefix,
		images = null,
		loadImage,
		onimage,
	}: {
		content: string;
		/** Prefix of heading ids, so a contents list can link to them: `${idPrefix}-${slug}`. */
		idPrefix?: string;
		/** Pictures kept for the answer; null while not known yet. Others are never loaded from the web. */
		images?: ResearchImage[] | null;
		/** The kept copy of a picture as a data URL. */
		loadImage?: (url: string) => Promise<string | null>;
		/** A picture was clicked. */
		onimage?: (url: string) => void;
	} = $props();
	const ownPrefix = $props.id();
	const prefix = $derived(idPrefix ?? ownPrefix);
	const html = $derived.by(() => {
		const renderer = new Renderer();
		const slug = uniqueSlugger();
		renderer.table = function (token) {
			return `<div class="markdown-table">${Renderer.prototype.table.call(this, token)}</div>`;
		};
		// Pictures become placeholders; the kept copy is put in below, never the web address.
		renderer.image = function ({ href, text }) {
			const caption = escapeHtml(text.trim());
			if (!/^https?:\/\//i.test(href)) return caption;
			return `<span class="answer-image" data-image="${escapeHtml(href)}" data-caption="${caption}"><span class="answer-image-caption">${caption}</span></span>`;
		};
		// Mermaid and chart blocks become placeholders showing their source; drawn below.
		renderer.code = function (token) {
			return diagramPlaceholder(token.lang, token.text) ?? Renderer.prototype.code.call(this, token);
		};
		renderer.heading = function ({ tokens, depth }) {
			const id = `${prefix}-${slug(plainInline(tokens).trim())}`;
			return `<h${depth} id="${id}">${this.parser.parseInline(tokens)}</h${depth}>`;
		};
		// Answers cite attached files as attachment:// links; keep those besides web links.
		return DOMPurify.sanitize(markdown.parse(content, { async: false, renderer }) as string, {
			ALLOWED_URI_REGEXP: /^(?:(?:https?|mailto|tel|attachment):|[^a-z]|[a-z+.-]+(?:[^a-z+.\-:]|$))/i,
		});
	});
	function escapeHtml(text: string): string {
		return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
	}

	let root: HTMLDivElement;
	// Answer pictures: the kept copy with its caption and source, or a link when none was kept.
	$effect(() => {
		void html;
		const known = images;
		const load = loadImage;
		let current = true;
		for (const element of root.querySelectorAll<HTMLElement>('.answer-image[data-image]')) {
			const url = element.dataset.image ?? '';
			const caption = element.dataset.caption ?? '';
			const image = known?.find((item) => item.url === url);
			if (!known) continue;
			if (!image || !load) {
				element.replaceChildren(captionLine(caption || t('research.image'), url, null));
				element.classList.add('missing');
				continue;
			}
			load(url).then(
				(src) => {
					if (!current) return;
					if (!src) {
						element.replaceChildren(captionLine(caption || t('research.image'), url, null));
						element.classList.add('missing');
						return;
					}
					const open = document.createElement('button');
					open.type = 'button';
					open.className = 'answer-image-open';
					open.dataset.open = url;
					open.setAttribute('aria-label', t('research.openImage', { name: caption || image.title || t('research.image') }));
					const picture = document.createElement('img');
					picture.src = src;
					picture.alt = caption;
					picture.width = image.width;
					picture.height = image.height;
					open.append(picture);
					element.replaceChildren(open, captionLine(caption, null, image.source_url));
					element.classList.remove('missing');
				},
				() => {}
			);
		}
		return () => {
			current = false;
		};
	});

	/** "Caption · site" with the site linking to the page; a picture without a copy links to itself. */
	function captionLine(caption: string, link: string | null, page: string | null): HTMLElement {
		const line = document.createElement('span');
		line.className = 'answer-image-caption';
		if (link) {
			const anchor = document.createElement('a');
			anchor.href = link;
			anchor.textContent = caption;
			line.append(anchor);
			return line;
		}
		line.append(caption);
		let host = '';
		try {
			host = page ? displayHost(new URL(page).hostname) : '';
		} catch {
			host = '';
		}
		if (page && host) {
			if (caption) line.append(' · ');
			const anchor = document.createElement('a');
			anchor.href = page;
			anchor.textContent = host;
			line.append(anchor);
		}
		return line;
	}

	// Formulas arrive as placeholders showing their source; MathJax (loaded on first use) draws them.
	$effect(() => {
		void html;
		const pending = [...root.querySelectorAll<HTMLElement>('.math[data-tex]:not([data-drawn])')];
		if (!pending.length) return;
		let current = true;
		import('$lib/utils/mathRender').then(({ renderMath }) => {
			for (const element of pending) {
				const tex = element.dataset.tex ?? '';
				renderMath(tex, 'display' in element.dataset).then(
					(svg) => {
						if (!current) return;
						element.innerHTML = svg;
						element.dataset.drawn = '';
						element.setAttribute('role', 'img');
						element.setAttribute('aria-label', tex);
					},
					() => current && element.classList.add('math-error')
				);
			}
		});
		return () => {
			current = false;
		};
	});

	// Diagrams and charts: drawn once their library loads (on first use), again when the theme
	// changes; a block that cannot be drawn keeps its source with a note.
	interface Drawn {
		title: string;
		source: string;
		svg: () => string;
	}
	const drawnDiagrams = new WeakMap<HTMLElement, Drawn>();
	// Lucide's "ellipsis" icon for the actions button made outside Svelte.
	const ELLIPSIS =
		'<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg>';
	let viewing = $state<{ title: string; svg: string } | null>(null);

	$effect(() => {
		void html;
		const themeId = $appliedTheme;
		const figures = [...root.querySelectorAll<HTMLElement>('figure.diagram[data-diagram]')];
		if (!figures.length) return;
		let current = true;
		const disposers: (() => void)[] = [];
		for (const figure of figures) {
			const source = diagramSource(figure);
			const fail = (message: string) => {
				if (!current) return;
				figure.querySelector('.diagram-view')?.remove();
				figure.classList.add('diagram-error');
				figure.removeAttribute('role');
				figure.removeAttribute('aria-label');
				let note = figure.querySelector<HTMLElement>('.diagram-note');
				if (!note) {
					note = document.createElement('figcaption');
					note.className = 'diagram-note';
					figure.prepend(note);
				}
				note.textContent = message;
			};
			const view = (label: string) => {
				figure.querySelector('.diagram-view')?.remove();
				const element = document.createElement('div');
				element.className = 'diagram-view';
				figure.prepend(element);
				figure.classList.remove('diagram-error');
				figure.querySelector('.diagram-note')?.remove();
				figure.setAttribute('role', 'img');
				figure.setAttribute('aria-label', label);
				if (!figure.querySelector('.diagram-actions')) {
					const button = document.createElement('button');
					button.type = 'button';
					button.className = 'diagram-actions';
					button.setAttribute('aria-label', t('research.diagramActions'));
					button.innerHTML = ELLIPSIS;
					tooltip(button, { text: t('research.diagramActions') });
					figure.append(button);
				}
				return element;
			};
			if (figure.dataset.diagram === 'chart') {
				const parsed = parseChart(source);
				if ('error' in parsed) {
					fail(`${t('research.diagramFailed')}: ${parsed.error}`);
					continue;
				}
				import('$lib/utils/chartRender').then(({ drawChart }) => {
					if (!current) return;
					const chart = view(diagramLabel(source, parsed.chart));
					const drawn = drawChart(chart, parsed.chart, chartColors(themeId));
					disposers.push(drawn.dispose);
					drawnDiagrams.set(figure, { title: parsed.chart.title, source, svg: drawn.svg });
				}, () => fail(t('research.diagramFailed')));
			} else {
				import('$lib/utils/mermaidRender')
					.then(({ drawMermaid }) => drawMermaid(source, themeId))
					.then(
						(svg) => {
							if (!current) return;
							view(diagramLabel(source)).innerHTML = svg;
							drawnDiagrams.set(figure, { title: '', source, svg: () => svg });
						},
						() => fail(t('research.diagramFailed'))
					);
			}
		}
		return () => {
			current = false;
			for (const dispose of disposers) dispose();
		};
	});

	function diagramMenu(figure: HTMLElement, point: { x: number; y: number }) {
		const drawn = drawnDiagrams.get(figure);
		if (!drawn) return;
		const title = drawn.title || t('research.diagram');
		const background = (themeById($appliedTheme) ?? themeById('default-light')!).colors['--app-panel'];
		const save = (format: 'svg' | 'png') => import('$lib/utils/diagramFiles').then(({ saveDiagram }) => saveDiagram(drawn.svg(), drawn.title, format, background));
		showContextMenu(point, t('research.diagramActions'), [
			{ label: t('research.diagramFullScreen'), icon: Maximize2Icon, action: () => void (viewing = { title, svg: drawn.svg() }) },
			{ label: t('research.diagramSaveSvg'), icon: FileImageIcon, separator: true, action: () => save('svg') },
			{ label: t('research.diagramSavePng'), icon: ImageIcon, action: () => save('png') },
			{ label: t('research.diagramCopySource'), icon: CodeIcon, separator: true, action: () => copyWithToast(drawn.source, t('research.diagramSourceCopied')) },
		]);
	}

	async function open(event: MouseEvent) {
		const actions = (event.target as Element)?.closest<HTMLElement>('.diagram-actions');
		const figure = actions?.closest<HTMLElement>('figure.diagram');
		if (actions && figure) {
			diagramMenu(figure, menuPointFor(actions));
			return;
		}
		const picture = (event.target as Element)?.closest<HTMLElement>('.answer-image-open');
		if (picture?.dataset.open) {
			onimage?.(picture.dataset.open);
			return;
		}
		const anchor = (event.target as Element)?.closest('a');
		if (!anchor) return;
		const href = anchor.getAttribute('href');
		if (!href) return;
		event.preventDefault();
		if (href.startsWith('#')) {
			document.getElementById(`${prefix}-${href.slice(1)}`)?.scrollIntoView({ block: 'start' });
			return;
		}
		if (isAttachmentUrl(href)) {
			showAttachment(href);
			return;
		}
		try {
			await openExternal(href);
		} catch {
			toast(t('common.openLinkFailed'), 'error');
		}
	}
</script>

<div
	bind:this={root}
	class="markdown"
	onclick={open}
	oncontextmenu={(event) => {
		const figure = (event.target as Element)?.closest<HTMLElement>('figure.diagram');
		if (figure && drawnDiagrams.has(figure)) {
			event.preventDefault();
			diagramMenu(figure, { x: event.clientX, y: event.clientY });
			return;
		}
		const anchor = (event.target as Element)?.closest('a');
		const url = webUrl(anchor?.getAttribute('href') ?? '');
		if (!anchor || !url) return;
		event.preventDefault();
		showContextMenu({ x: event.clientX, y: event.clientY }, t('common.actionsFor', { name: url }), linkMenuItems(url, anchor.textContent ?? ''));
	}}
	role="presentation"
>
	{@html html}
</div>
{#if viewing}
	<DiagramViewer title={viewing.title} svg={viewing.svg} onclose={() => (viewing = null)} />
{/if}

<style>
	.markdown {
		line-height: 1.7;
		overflow-wrap: anywhere;
		min-width: 0;
	}
	.markdown :global(h1),
	.markdown :global(h2),
	.markdown :global(h3),
	.markdown :global(h4) {
		font-weight: 650;
		line-height: 1.35;
		margin: 1.25em 0 0.5em;
		scroll-margin-top: 12px;
	}
	.markdown :global(h1) {
		font-size: var(--app-text-3xl);
	}
	.markdown :global(h2) {
		font-size: var(--app-text-2xl);
	}
	.markdown :global(h3) {
		font-size: var(--app-text-xl);
	}
	.markdown :global(> :first-child) {
		margin-top: 0;
	}
	.markdown :global(p) {
		margin: 0.75em 0;
	}
	.markdown :global(ul) {
		list-style: disc;
	}
	.markdown :global(ol) {
		list-style: decimal;
	}
	.markdown :global(ul),
	.markdown :global(ol) {
		padding-left: 1.7em;
		margin: 0.75em 0;
	}
	.markdown :global(li) {
		margin: 0.25em 0;
	}
	.markdown :global(a) {
		color: var(--app-accent);
		text-decoration: underline;
		text-underline-offset: 3px;
	}
	.markdown :global(pre) {
		overflow: auto;
		max-width: 100%;
		background: var(--app-bg);
		border: 1px solid var(--app-border);
		padding: 12px;
		border-radius: var(--app-radius);
	}
	.markdown :global(code) {
		font: 0.9em/1.6 var(--app-font-mono);
		padding: 2px 4px;
		background: var(--app-subtle);
		border-radius: var(--app-radius-sm);
	}
	.markdown :global(pre code) {
		padding: 0;
		background: transparent;
		white-space: pre;
	}
	.markdown :global(img) {
		max-width: 100%;
		height: auto;
		border-radius: var(--app-radius);
	}
	.markdown :global(.markdown-table) {
		max-width: 100%;
		overflow-x: auto;
		margin: 16px 0;
	}
	.markdown :global(table) {
		border-collapse: collapse;
		min-width: 100%;
	}
	.markdown :global(th),
	.markdown :global(td) {
		border: 1px solid var(--app-border);
		padding: 8px 12px;
		text-align: left;
		min-width: 100px;
		/* Cells keep words whole; a wide table scrolls inside .markdown-table instead. */
		overflow-wrap: normal;
		word-break: normal;
	}
	/* Comfortable line length for running text; tables and code may use the full width. */
	.markdown > :global(:is(p, ul, ol, blockquote, h1, h2, h3, h4, h5, h6, hr)) {
		max-width: 75ch;
	}
	.markdown :global(th) {
		background: var(--app-subtle);
	}
	.markdown :global(blockquote) {
		padding-left: 16px;
		border-left: 3px solid var(--app-border);
		margin: 16px 0;
		color: var(--app-muted);
	}
	.markdown :global(.answer-image) {
		display: block;
		margin: 16px 0;
		max-width: min(100%, 640px);
	}
	.markdown :global(.answer-image-open) {
		display: block;
		padding: 0;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		background: var(--app-subtle);
		overflow: hidden;
		cursor: zoom-in;
	}
	.markdown :global(.answer-image-open img) {
		display: block;
		width: auto;
		max-width: 100%;
		max-height: 420px;
		height: auto;
		border-radius: 0;
	}
	.markdown :global(.answer-image-caption) {
		display: block;
		margin-top: 6px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		line-height: 1.5;
	}
	.markdown :global(.answer-image.missing .answer-image-caption) {
		margin-top: 0;
	}
	.markdown :global(.math-display) {
		display: block;
		margin: 0.75em 0;
		text-align: center;
		/* A long formula scrolls by itself instead of widening the answer. */
		overflow-x: auto;
		overflow-y: hidden;
		overflow-wrap: normal;
	}
	.markdown :global(.math svg) {
		display: inline;
		max-width: none;
	}
	.markdown :global(.math-error) {
		font-family: var(--app-font-mono);
		font-size: 0.9em;
		text-decoration: wavy underline var(--app-danger);
		text-underline-offset: 3px;
	}
	.markdown :global(.diagram) {
		position: relative;
		margin: 16px 0;
		max-width: 100%;
	}
	.markdown :global(.diagram[role='img']) {
		padding: 12px 40px 12px 12px;
		background: var(--app-panel);
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
	}
	/* The source shows until the picture is drawn, and stays when it cannot be. */
	.markdown :global(.diagram[role='img'] .diagram-source) {
		display: none;
	}
	.markdown :global(.diagram-view) {
		width: 100%;
		overflow-x: auto;
		overflow-wrap: normal;
	}
	.markdown :global(.diagram-view > svg) {
		display: block;
		max-width: 100%;
		height: auto;
		margin: 0 auto;
	}
	.markdown :global(.diagram-actions) {
		position: absolute;
		top: 8px;
		right: 8px;
		display: grid;
		place-items: center;
		width: 28px;
		height: 28px;
		padding: 0;
		color: var(--app-muted);
		background: transparent;
		border: 1px solid transparent;
		border-radius: var(--app-radius-sm);
		cursor: pointer;
	}
	.markdown :global(.diagram-actions:hover),
	.markdown :global(.diagram-actions:focus-visible) {
		color: var(--app-text);
		background: var(--app-subtle);
		border-color: var(--app-border);
	}
	.markdown :global(.diagram-error .diagram-actions) {
		display: none;
	}
	.markdown :global(.diagram-note) {
		margin-bottom: 6px;
		color: var(--app-danger);
		font-size: var(--app-text-sm);
	}
	.markdown :global(hr) {
		border: 0;
		border-top: 1px solid var(--app-border);
		margin: 20px 0;
	}
</style>
