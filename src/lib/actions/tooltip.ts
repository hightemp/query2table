// Themed tooltips that replace the browser's `title` tooltips.
// Usage: <span use:tooltip={'Text'}> or use:tooltip={{ text, whenTruncated: true }}.

export type TooltipOptions = string | null | undefined | { text: string | null | undefined; whenTruncated?: boolean };

const SHOW_DELAY = 500;
/** Moving from one tooltip to the next shows it at once. */
const WARM_FOR = 300;
const GAP = 6;
const ID = 'app-tooltip';

let element: HTMLDivElement | null = null;
let owner: HTMLElement | null = null;
let lastHidden = 0;

function tooltipElement(host: HTMLElement): HTMLDivElement {
	if (!element) {
		element = document.createElement('div');
		element.id = ID;
		element.className = 'app-tooltip';
		element.setAttribute('role', 'tooltip');
		element.hidden = true;
	}
	// A modal dialog sits in the top layer, so the tooltip must live inside it to be seen.
	const container = host.closest('dialog[open]') ?? document.body;
	if (element.parentElement !== container) container.append(element);
	return element;
}

function normalize(options: TooltipOptions) {
	if (options == null || typeof options === 'string') return { text: options ?? '', whenTruncated: false };
	return { text: options.text ?? '', whenTruncated: !!options.whenTruncated };
}

function isTruncated(node: HTMLElement) {
	return node.scrollWidth > node.clientWidth + 1 || node.scrollHeight > node.clientHeight + 1;
}

function place(node: HTMLElement, tip: HTMLElement) {
	const rect = node.getBoundingClientRect();
	tip.style.left = '0px';
	tip.style.top = '0px';
	const { width, height } = tip.getBoundingClientRect();
	const above = rect.top - height - GAP;
	const top = above >= 8 ? above : Math.min(rect.bottom + GAP, window.innerHeight - height - 8);
	const left = Math.max(8, Math.min(rect.left + rect.width / 2 - width / 2, window.innerWidth - width - 8));
	tip.style.left = `${Math.round(left)}px`;
	tip.style.top = `${Math.round(top)}px`;
	tip.dataset.side = above >= 8 ? 'top' : 'bottom';
}

function describe(node: HTMLElement, on: boolean) {
	const ids = (node.getAttribute('aria-describedby') ?? '').split(/\s+/).filter((id) => id && id !== ID);
	if (on) ids.push(ID);
	if (ids.length) node.setAttribute('aria-describedby', ids.join(' '));
	else node.removeAttribute('aria-describedby');
}

function hideCurrent() {
	if (!element || element.hidden) return;
	element.hidden = true;
	if (owner) describe(owner, false);
	owner = null;
	lastHidden = Date.now();
}

function onKey(event: KeyboardEvent) {
	if (event.key === 'Escape') hideCurrent();
}

let listening = false;
function listenGlobally() {
	if (listening) return;
	listening = true;
	document.addEventListener('keydown', onKey, true);
	window.addEventListener('scroll', hideCurrent, true);
	window.addEventListener('blur', hideCurrent);
}

export function tooltip(node: HTMLElement, options: TooltipOptions) {
	let current = normalize(options);
	let timer: ReturnType<typeof setTimeout> | undefined;

	function show() {
		timer = undefined;
		const text = current.text.trim();
		if (!text || !node.isConnected) return;
		if (current.whenTruncated && !isTruncated(node)) return;
		const tip = tooltipElement(node);
		if (owner && owner !== node) describe(owner, false);
		owner = node;
		tip.textContent = text;
		tip.hidden = false;
		place(node, tip);
		// Text that repeats the element's own name would be read twice.
		const own = (node.getAttribute('aria-label') ?? node.textContent ?? '').trim();
		describe(node, own !== text);
		listenGlobally();
	}
	function schedule() {
		clearTimeout(timer);
		timer = setTimeout(show, Date.now() - lastHidden < WARM_FOR ? 0 : SHOW_DELAY);
	}
	function cancel() {
		clearTimeout(timer);
		timer = undefined;
		if (owner === node) hideCurrent();
	}
	function focusIn() {
		// Keyboard focus only: a click focuses too, and the hover already handles the mouse.
		if (node.matches(':focus-visible') || node.querySelector(':focus-visible')) schedule();
	}

	node.addEventListener('pointerenter', schedule);
	node.addEventListener('pointerleave', cancel);
	node.addEventListener('pointerdown', cancel);
	node.addEventListener('focusin', focusIn);
	node.addEventListener('focusout', cancel);

	return {
		update(next: TooltipOptions) {
			current = normalize(next);
			if (owner === node && element && !element.hidden) {
				if (!current.text.trim()) hideCurrent();
				else {
					element.textContent = current.text.trim();
					place(node, element);
				}
			}
		},
		destroy() {
			cancel();
			node.removeEventListener('pointerenter', schedule);
			node.removeEventListener('pointerleave', cancel);
			node.removeEventListener('pointerdown', cancel);
			node.removeEventListener('focusin', focusIn);
			node.removeEventListener('focusout', cancel);
		},
	};
}
