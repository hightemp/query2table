/** Decodes one punycode label (RFC 3492), e.g. "80aswg" → "рф". */
function decodePunycode(input: string): string {
	const base = 36;
	const output: number[] = [];
	const delimiter = input.lastIndexOf('-');
	for (let i = 0; i < Math.max(0, delimiter); i++) output.push(input.charCodeAt(i));
	let n = 128;
	let bias = 72;
	let i = 0;
	for (let index = delimiter + 1; index < input.length; ) {
		const oldi = i;
		for (let w = 1, k = base; ; k += base) {
			if (index >= input.length) throw new Error('Invalid punycode');
			const code = input.charCodeAt(index++);
			const digit =
				code >= 48 && code <= 57
					? code - 22
					: code >= 65 && code <= 90
						? code - 65
						: code >= 97 && code <= 122
							? code - 97
							: base;
			if (digit >= base) throw new Error('Invalid punycode');
			i += digit * w;
			const t = k <= bias ? 1 : k >= bias + 26 ? 26 : k - bias;
			if (digit < t) break;
			w *= base - t;
		}
		const length = output.length + 1;
		let delta = oldi === 0 ? Math.floor((i - oldi) / 700) : (i - oldi) >> 1;
		delta += Math.floor(delta / length);
		let k = 0;
		for (; delta > 455; k += base) delta = Math.floor(delta / 35);
		bias = Math.floor(k + (36 * delta) / (delta + 38));
		n += Math.floor(i / length);
		i %= length;
		output.splice(i++, 0, n);
	}
	return String.fromCodePoint(...output);
}

/** Readable host name: without "www." and with internationalized domains decoded. */
export function displayHost(host: string): string {
	return host
		.replace(/^www\./, '')
		.split('.')
		.map((label) => {
			if (!label.startsWith('xn--')) return label;
			try {
				return decodePunycode(label.slice(4));
			} catch {
				return label;
			}
		})
		.join('.');
}

/** Readable host of a URL, or null when it is not a URL. */
export function hostname(url: string | null): string | null {
	if (!url) return null;
	try {
		return displayHost(new URL(url).hostname);
	} catch {
		return null;
	}
}
