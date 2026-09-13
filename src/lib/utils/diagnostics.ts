/** UI diagnostics deliberately exclude settings values, URLs, content and credentials. */
export function debugUi(event: string, metadata: Record<string, number | boolean | null> = {}) {
	if (import.meta.env.DEV && import.meta.env.VITE_LOG_LEVEL !== 'off') {
		console.debug(`[ui] ${event}`, metadata);
	}
}
