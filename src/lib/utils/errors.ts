import type { LlmIssueCode } from '$lib/types';
import { t, type MessageKey } from '$lib/i18n';

export type ErrorContext = 'run' | 'control' | 'settings' | 'settings_load' | 'catalog' | 'history' | 'export' | 'search' | 'fetch' | 'app_files';

export interface ErrorPresentation {
	title: string;
	cause: string;
	action: string;
	settingsHref?: string;
	details: string;
	/** No specific explanation was found. */
	generic?: boolean;
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
	const result = (name: string, settingsHref?: string, action?: MessageKey): ErrorPresentation => ({
		title: t(`errors.${name}.title` as MessageKey),
		cause: t(`errors.${name}.cause` as MessageKey),
		action: t(action ?? (`errors.${name}.action` as MessageKey)),
		settingsHref,
		details,
	});
	const matches = (value: LlmIssueCode, pattern: RegExp) => code ? code === value : pattern.test(text);

	if (context === 'app_files' && /clipboard|could not copy/.test(text)) {
		return result('copyPath');
	}
	if (context === 'app_files' && /could not open.*folder|file manager/.test(text)) {
		return result('openFolder');
	}

	if (/pricing settings are invalid/.test(text)) {
		return result('pricing');
	}
	if (matches('budget_limit', /run spending limit reached/)) {
		return result('budget');
	}
	if (matches('model_not_found', /model.*(?:not found|not available|does not exist|unavailable)|(?:unknown|missing) model/)) {
		return result('modelNotFound', '/settings#llm_provider');
	}
	if (matches('not_configured', /not configured|no .*(?:api key|provider|model)|(?:api key|model|base url).*(?:required|missing|empty)/)) {
		return result('notConfigured', '/settings');
	}
	if (matches('unsupported_setting', /unsupported.*(?:reasoning|thinking|effort|setting)|(?:reasoning|thinking|effort).*(?:unsupported|not support|invalid)/)) {
		return result('unsupportedSetting', '/settings#llm_reasoning_effort');
	}
	if (matches('context_limit', /context[_ ](?:length|limit|window)|maximum context|prompt (?:is )?too long|input.*token.*(?:exceed|limit)/)) {
		return result('contextLimit', '/settings#max_extraction_text_chars');
	}
	if (matches('output_limit', /output[_ ]limit|(?:output|completion|generation).*token.*limit|(?:max_tokens|max tokens|token limit).*(?:reach|exceed|exhaust)|finish_reason.*length|done_reason.*length|truncated.*(?:response|json)/)) {
		return result('outputLimit', '/settings#llm_max_tokens');
	}
	if (matches('quota', /insufficient[_ ]quota|quota|credits|insufficient.*(?:balance|funds)|billing|\b402\b/)) {
		return result('quota', '/settings#llm_provider');
	}
	if (matches('auth', /\b401\b|unauthori[sz]ed|invalid.*(?:api.?key|token)|authentication/)) {
		return result('auth', '/settings');
	}
	if (matches('access_denied', /\b403\b|forbidden|access denied/)) {
		return result('accessDenied', '/settings');
	}
	if (matches('rate_limit', /\b429\b|rate[_ -]?limit|too many requests/)) {
		return result('rateLimit', '/settings');
	}
	if (matches('timeout', /timed? out|timeout|deadline exceeded/)) {
		return result('timeout', '/settings#fetch_timeout_seconds');
	}
	if (matches('connection', /connect(?:ion|ing)?.*(?:refused|reset|failed|error)|error.*connect|network|dns|certificate|tls|ssl|unreachable|failed to fetch|sending request/)) {
		return result('connection', '/settings');
	}
	if (matches('empty_response', /empty[_ ]response|response.*empty|no (?:answer|content).*returned/)) {
		return result('emptyResponse', '/settings#llm_reasoning_effort');
	}
	if (matches('invalid_response', /invalid[_ ]response|parse error|failed to parse|invalid json|json.*(?:invalid|parse|eof)|eof.*parsing/)) {
		const stored = ['history', 'settings', 'settings_load', 'export'].includes(context);
		return result(
			'invalidResponse',
			stored ? undefined : '/settings',
			context === 'catalog'
				? 'errors.invalidResponse.actionCatalog'
				: stored
					? 'errors.invalidResponse.actionStored'
					: 'errors.invalidResponse.actionRun'
		);
	}
	if (matches('provider_error', /\b5\d\d\b|service unavailable|bad gateway|provider error/)) {
		return result('providerError', '/settings#llm_provider');
	}
	if (/run(?: [a-z0-9-]+)? is (?:not|no longer) active|control channel.*closed/.test(text)) {
		return result('runInactive');
	}
	if (/no space left|disk.*full/.test(text)) {
		return result('diskFull');
	}
	if (/permission denied|read-only file|readonly database|not permitted/.test(text)) {
		return result('permission', undefined, context === 'export' ? 'errors.permission.actionExport' : undefined);
	}
	if (/database|sqlite|sqlx|storage/.test(text)) {
		return result('storage');
	}
	return { ...result('unknown', undefined, `errors.unknown.action.${context}` as MessageKey), generic: true };
}
