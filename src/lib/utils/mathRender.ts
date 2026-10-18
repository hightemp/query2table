/**
 * Draws LaTeX with MathJax as SVG. Loaded on demand (`import()`), since MathJax is large;
 * the font's extra glyph files are bundled and load only when a formula needs them.
 */
import DOMPurify from 'dompurify';
import { mathjax } from '@mathjax/src/mjs/mathjax.js';
import { TeX } from '@mathjax/src/mjs/input/tex.js';
import { SVG } from '@mathjax/src/mjs/output/svg.js';
import { liteAdaptor } from '@mathjax/src/mjs/adaptors/liteAdaptor.js';
import { RegisterHTMLHandler } from '@mathjax/src/mjs/handlers/html.js';
import { MathJaxNewcmFont } from '@mathjax/mathjax-newcm-font/mjs/svg.js';
import '@mathjax/src/mjs/input/tex/base/BaseConfiguration.js';
import '@mathjax/src/mjs/input/tex/ams/AmsConfiguration.js';
import '@mathjax/src/mjs/input/tex/newcommand/NewcommandConfiguration.js';
import '@mathjax/src/mjs/input/tex/boldsymbol/BoldsymbolConfiguration.js';
import '@mathjax/src/mjs/input/tex/braket/BraketConfiguration.js';
import '@mathjax/src/mjs/input/tex/cancel/CancelConfiguration.js';
import '@mathjax/src/mjs/input/tex/cases/CasesConfiguration.js';
import '@mathjax/src/mjs/input/tex/color/ColorConfiguration.js';
import '@mathjax/src/mjs/input/tex/enclose/EncloseConfiguration.js';
import '@mathjax/src/mjs/input/tex/extpfeil/ExtpfeilConfiguration.js';
import '@mathjax/src/mjs/input/tex/gensymb/GensymbConfiguration.js';
import '@mathjax/src/mjs/input/tex/mathtools/MathtoolsConfiguration.js';
import '@mathjax/src/mjs/input/tex/mhchem/MhchemConfiguration.js';
import '@mathjax/src/mjs/input/tex/physics/PhysicsConfiguration.js';
import '@mathjax/src/mjs/input/tex/textmacros/TextMacrosConfiguration.js';

const PACKAGES = [
	'base',
	'ams',
	'newcommand',
	'boldsymbol',
	'braket',
	'cancel',
	'cases',
	'color',
	'enclose',
	'extpfeil',
	'gensymb',
	'mathtools',
	'mhchem',
	'physics',
	'textmacros',
];

const glyphFiles = import.meta.glob('/node_modules/@mathjax/mathjax-newcm-font/mjs/svg/dynamic/*.js');

// The font asks for '@mathjax/mathjax-newcm-font/js/svg/dynamic/<name>.js'.
mathjax.asyncLoad = (name: string) => {
	const file = name.split('/').pop();
	const load = glyphFiles[`/node_modules/@mathjax/mathjax-newcm-font/mjs/svg/dynamic/${file}`];
	return load ? load() : Promise.reject(new Error(`No MathJax file ${name}`));
};

const adaptor = liteAdaptor();
RegisterHTMLHandler(adaptor);
const document = mathjax.document('', {
	InputJax: new TeX({
		packages: PACKAGES,
		// Fail instead of drawing the error, so the answer shows the source.
		formatError: (_jax: unknown, error: Error) => {
			throw error;
		},
	}),
	// An inline formula stays one picture: its line-break points would need MathJax's own elements.
	OutputJax: new SVG({ fontData: MathJaxNewcmFont, fontCache: 'none', linebreaks: { inline: false } }),
});

const drawn = new Map<string, Promise<string>>();

/** SVG markup of a formula; rejects when the LaTeX does not parse. */
export function renderMath(tex: string, display: boolean): Promise<string> {
	const key = `${display ? 'D' : 'I'}${tex}`;
	let svg = drawn.get(key);
	if (!svg) {
		svg = document.convertPromise(tex, { display }).then((node) => {
			document.clear();
			return DOMPurify.sanitize(adaptor.innerHTML(node), { USE_PROFILES: { svg: true } });
		});
		svg.catch(() => drawn.delete(key));
		drawn.set(key, svg);
	}
	return svg;
}
