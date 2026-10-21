/** Saving drawn diagrams and charts as SVG or PNG files. */
import { save } from '@tauri-apps/plugin-dialog';
import { t } from '$lib/i18n';
import { saveFileData } from '$lib/api/tauri';
import { toast } from '$lib/stores/toasts';
import { errorText } from './errors';
import { readablePath } from './platform';
import { diagramFileName } from './diagrams';

function base64(bytes: Uint8Array): string {
	let binary = '';
	for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
	return btoa(binary);
}

/** The picture's size: its width and height, or its view box when it scales with the page. */
function sizeOf(svg: SVGSVGElement): { width: number; height: number } {
	const box = svg.viewBox.baseVal;
	const width = Number.parseFloat(svg.getAttribute('width') ?? '');
	const height = Number.parseFloat(svg.getAttribute('height') ?? '');
	if (width > 0 && height > 0 && !svg.getAttribute('width')?.includes('%')) return { width, height };
	return { width: box?.width || 800, height: box?.height || 600 };
}

/** A standalone SVG document with a fixed size, as files and images need. */
export function standaloneSvg(markup: string): { svg: string; width: number; height: number } {
	const document = new DOMParser().parseFromString(markup, 'image/svg+xml');
	const svg = document.documentElement as unknown as SVGSVGElement;
	const { width, height } = sizeOf(svg);
	svg.setAttribute('xmlns', 'http://www.w3.org/2000/svg');
	svg.setAttribute('width', String(width));
	svg.setAttribute('height', String(height));
	svg.style.removeProperty('max-width');
	return { svg: new XMLSerializer().serializeToString(svg), width, height };
}

/** PNG at twice the size, on the given background. */
async function png(markup: string, background: string): Promise<Uint8Array> {
	const { svg, width, height } = standaloneSvg(markup);
	const image = new Image();
	image.src = `data:image/svg+xml;base64,${base64(new TextEncoder().encode(svg))}`;
	await image.decode();
	const scale = 2;
	const canvas = document.createElement('canvas');
	canvas.width = Math.ceil(width * scale);
	canvas.height = Math.ceil(height * scale);
	const context = canvas.getContext('2d')!;
	context.fillStyle = background;
	context.fillRect(0, 0, canvas.width, canvas.height);
	context.drawImage(image, 0, 0, canvas.width, canvas.height);
	const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, 'image/png'));
	if (!blob) throw new Error('PNG encoding failed');
	return new Uint8Array(await blob.arrayBuffer());
}

/** Asks where to save the picture and writes it there. */
export async function saveDiagram(markup: string, title: string, format: 'svg' | 'png', background: string) {
	try {
		const path = await save({
			defaultPath: diagramFileName(title, format),
			filters: [{ name: t('research.diagramFileType', { type: format.toUpperCase() }), extensions: [format] }],
		});
		if (!path) return;
		const bytes = format === 'svg' ? new TextEncoder().encode(standaloneSvg(markup).svg) : await png(markup, background);
		const written = await saveFileData(path, base64(bytes));
		toast(t('research.diagramSaved', { path: readablePath(written) ?? t('files.chosenFile') }), 'success');
	} catch (error) {
		toast(t('research.diagramSaveFailed', { error: errorText(error) }), 'error');
	}
}
