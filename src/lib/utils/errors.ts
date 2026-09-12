import type { LlmIssueCode } from '$lib/types';

export type ErrorContext = 'run' | 'control' | 'settings' | 'settings_load' | 'catalog' | 'history' | 'export' | 'search' | 'fetch';

export interface ErrorPresentation {
	title: string;
	cause: string;
	action: string;
	settingsHref?: string;
	details: string;
}

export function errorText(error: unknown): string {
	if (error instanceof Error) return error.message;
	if (typeof error === 'string') return error;
	try { return JSON.stringify(error) ?? String(error); }
	catch { return String(error); }
}

/** Prefer backend issue codes. String matching is only a fallback for legacy errors. */
export function presentError(error: unknown, context: ErrorContext = 'run', code?: LlmIssueCode): ErrorPresentation {
	const details = errorText(error);
	const text = details.toLowerCase();
	const result = (title: string, cause: string, action: string, settingsHref?: string): ErrorPresentation =>
		({ title, cause, action, settingsHref, details });
	const matches = (value: LlmIssueCode, pattern: RegExp) => code ? code === value : pattern.test(text);

	if (matches('model_not_found', /model.*(?:not found|not available|does not exist|unavailable)|(?:unknown|missing) model/)) {
		return result('The selected model is unavailable', 'The service could not find or provide the requested model.',
			'Choose an available model from the provider list. For a local server, install or load the model and check its exact name.', '/settings#llm_provider');
	}
	if (matches('not_configured', /not configured|no .*(?:api key|provider|model)|(?:api key|model|base url).*(?:required|missing|empty)/)) {
		return result('Provider setup is incomplete', 'A required provider, model, or API key is missing.',
			'Choose a provider and model, check the required API keys, and save Settings before trying again.', '/settings');
	}
	if (matches('unsupported_setting', /unsupported.*(?:reasoning|thinking|effort|setting)|(?:reasoning|thinking|effort).*(?:unsupported|not support|invalid)/)) {
		return result('The model does not support this setting', 'The provider rejected a requested model option.',
			'Use Auto or Provider default for Thinking / reasoning effort, or choose a supported level. GPT-OSS on Ollama requires Low, Medium, or High.', '/settings#llm_reasoning_effort');
	}
	if (matches('context_limit', /context[_ ](?:length|limit|window)|maximum context|prompt (?:is )?too long|input.*token.*(?:exceed|limit)/)) {
		return result('The input exceeds the model context limit', 'The prompt and source text do not fit in this model’s available context.',
			'Reduce source text limits in Content Processing or choose a model with a larger context window.', '/settings#max_extraction_text_chars');
	}
	if (matches('output_limit', /output[_ ]limit|(?:output|completion|generation).*token.*limit|(?:max_tokens|max tokens|token limit).*(?:reach|exceed|exhaust)|finish_reason.*length|done_reason.*length|truncated.*(?:response|json)/)) {
		return result('The model reached its output limit', 'The response stopped before a complete answer was available. Thinking may use part of the same token allowance.',
			'Increase Max output tokens or lower Thinking / reasoning effort, then try again. Run cost and time limits are separate.', '/settings#llm_max_tokens');
	}
	if (matches('quota', /insufficient[_ ]quota|quota|credits|insufficient.*(?:balance|funds)|billing|\b402\b/)) {
		return result('The provider allowance is exhausted', 'The provider reported a credit, quota, or account usage limit.',
			'Check your provider account allowance or choose another provider or model. Increasing the run budget does not change provider quotas.', '/settings#llm_provider');
	}
	if (matches('auth', /\b401\b|unauthori[sz]ed|invalid.*(?:api.?key|token)|authentication/)) {
		return result('The provider could not authenticate', 'The API key is missing, expired, or was rejected by the service.',
			'Check the API key for the selected service, save Settings, and retry.', '/settings');
	}
	if (matches('access_denied', /\b403\b|forbidden|access denied/)) {
		return result('Access was denied', 'The service refused access to the requested resource or model.',
			'Check model access, account permissions, and the configured endpoint or proxy. A different available model may work.', '/settings');
	}
	if (matches('rate_limit', /\b429\b|rate[_ -]?limit|too many requests/)) {
		return result('The service limited requests', 'The service is currently limiting how many requests it accepts.',
			'Wait for the reported retry interval or try again later. Check provider limits if this continues.', '/settings');
	}
	if (matches('timeout', /timed? out|timeout|deadline exceeded/)) {
		return result('The request timed out', 'The operation did not finish within the allowed time.',
			'Try again, check the connection, or use a faster model. For page downloads, review Fetch Timeout in Settings.', '/settings#fetch_timeout_seconds');
	}
	if (matches('connection', /connect(?:ion|ing)?.*(?:refused|reset|failed|error)|error.*connect|network|dns|certificate|tls|ssl|unreachable|failed to fetch|sending request/)) {
		return result('Could not connect to the service', 'The connection failed before a usable response arrived.',
			'Check your connection, server URL, and proxy. For local models, make sure the model server is running.', '/settings');
	}
	if (matches('empty_response', /empty[_ ]response|response.*empty|no (?:answer|content).*returned/)) {
		return result('The model returned no answer', 'The provider returned no usable answer content.',
			'Try again or choose another model. Review Thinking / reasoning effort and Max output tokens if this repeats.', '/settings#llm_reasoning_effort');
	}
	if (matches('invalid_response', /invalid[_ ]response|parse error|failed to parse|invalid json|json.*(?:invalid|parse|eof)|eof.*parsing/)) {
		return result('The response could not be read', 'The returned data did not match the format needed for this step.',
			context === 'catalog' ? 'Use Refresh to reload the model list. Check the provider URL if it keeps failing.'
				: ['history', 'settings', 'settings_load', 'export'].includes(context) ? 'Retry the operation. Keep the technical details if stored data cannot be read.'
				: 'Try again. For repeated model failures, choose another model or review its JSON and reasoning settings.',
			['history', 'settings', 'settings_load', 'export'].includes(context) ? undefined : '/settings');
	}
	if (matches('provider_error', /\b5\d\d\b|service unavailable|bad gateway|provider error/)) {
		return result('The service could not complete the request', 'The provider reported an error while processing the request.',
			'Try again later or select another available provider or model.', '/settings#llm_provider');
	}
	if (/run(?: [a-z0-9-]+)? is (?:not|no longer) active|control channel.*closed/.test(text)) {
		return result('This run can no longer receive commands', 'The backend no longer has an active worker for this run.',
			'Check Run History and the logs for the final outcome. Reopen the app if its displayed status has not updated.');
	}
	if (/no space left|disk.*full/.test(text)) {
		return result('There is not enough disk space', 'The app could not write data because the storage device is full.',
			'Free disk space and retry. For exports, you can choose another destination.');
	}
	if (/permission denied|read-only file|readonly database|not permitted/.test(text)) {
		return result('The app cannot write to this location', 'The operating system refused permission to save the data.',
			context === 'export' ? 'Choose a writable export folder and retry.' : 'Check write permissions for the app’s data folder and retry.');
	}
	if (/database|sqlite|sqlx|storage/.test(text)) {
		return result('Local data could not be accessed', 'The app encountered a problem while reading or saving local data.',
			'Check disk space and access to the app’s data folder, then retry. Keep the technical details if the problem continues.');
	}
	const actions: Record<ErrorContext, string> = {
		run: 'Try a new run. If it fails again, use the technical details and logs to investigate.',
		control: 'Check the run status and retry the command. Keep the technical details if the problem continues.',
		settings: 'Your unsaved edits are still available. Retry saving and keep the technical details if it fails again.',
		settings_load: 'Retry loading Settings. Keep the technical details if local settings remain unavailable.',
		catalog: 'Use Refresh to try loading the model list again. Check the technical details if it keeps failing.',
		history: 'Retry opening Run History. Keep the technical details if local data remains unavailable.',
		export: 'Try exporting again to a writable folder. Keep the technical details if it fails again.',
		search: 'Retry the search and check the technical details if it fails again.',
		fetch: 'Try another source or retry the download. Check the technical details if it keeps failing.',
	};
	return result('An unexpected error occurred', 'The available error information does not identify a specific cause.', actions[context]);
}
