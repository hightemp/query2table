<script lang="ts">
	import { marked, Renderer } from 'marked';
	import DOMPurify from 'dompurify';
	import { openExternal } from '$lib/utils/links';
	let { content }: { content: string } = $props();
	const prefix = $props.id();
	let linkError = $state(false);
	const html = $derived.by(() => {
		const renderer = new Renderer();
		renderer.table = function (token) {
			return `<div class="markdown-table">${Renderer.prototype.table.call(this, token)}</div>`;
		};
		renderer.heading = function ({ tokens, depth, text }) {
			const slug = text
				.toLowerCase()
				.replace(/[^\p{L}\p{N}]+/gu, '-')
				.replace(/^-|-$/g, '');
			return `<h${depth} id="${prefix}-${slug}">${this.parser.parseInline(tokens)}</h${depth}>`;
		};
		return DOMPurify.sanitize(marked.parse(content, { async: false, renderer }) as string);
	});
	async function open(event: MouseEvent) {
		const anchor = (event.target as Element)?.closest('a');
		if (!anchor) return;
		const href = anchor.getAttribute('href');
		if (!href) return;
		event.preventDefault();
		linkError = false;
		if (href.startsWith('#')) {
			document.getElementById(`${prefix}-${href.slice(1)}`)?.scrollIntoView({ block: 'start' });
			return;
		}
		try {
			await openExternal(href);
		} catch {
			linkError = true;
		}
	}
</script>

<div class="markdown" onclick={open} role="presentation">{@html html}</div>
{#if linkError}<p role="alert">
		Could not open the link. Copy its address and open it in your browser.
	</p>{/if}

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
		font-size: 24px;
	}
	.markdown :global(h2) {
		font-size: 20px;
	}
	.markdown :global(h3) {
		font-size: 17px;
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
		border-radius: 8px;
	}
	.markdown :global(code) {
		font:
			0.9em/1.6 ui-monospace,
			monospace;
		padding: 2px 4px;
		background: var(--app-subtle);
		border-radius: 4px;
	}
	.markdown :global(pre code) {
		padding: 0;
		background: transparent;
		white-space: pre;
	}
	.markdown :global(img) {
		max-width: 100%;
		height: auto;
		border-radius: 8px;
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
	.markdown :global(hr) {
		border: 0;
		border-top: 1px solid var(--app-border);
		margin: 20px 0;
	}
</style>
