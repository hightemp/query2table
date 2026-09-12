import { describe, expect, it } from 'vitest';
import { presentError } from '$lib/utils/errors';
import type { LlmIssueCode } from '$lib/types';

describe('error explanations', () => {
	it.each<[LlmIssueCode, string, string]>([
		['output_limit', 'output limit', 'Max output tokens'],
		['context_limit', 'context limit', 'Content Processing'],
		['rate_limit', 'limited requests', 'retry interval'],
		['quota', 'allowance', 'run budget'],
		['auth', 'authenticate', 'API key'],
		['access_denied', 'denied', 'permissions'],
		['timeout', 'timed out', 'connection'],
		['connection', 'connect', 'server URL'],
		['invalid_response', 'could not be read', 'JSON'],
		['empty_response', 'no answer', 'Thinking'],
		['not_configured', 'incomplete', 'save Settings'],
		['unsupported_setting', 'does not support', 'Provider default'],
		['model_not_found', 'unavailable', 'install or load'],
		['provider_error', 'could not complete', 'another'],
	])('explains structured %s errors without relying on raw provider wording', (code, title, action) => {
		const result = presentError('Opaque provider detail', 'run', code);
		expect(result.title).toContain(title);
		expect(result.action).toContain(action);
		expect(result.settingsHref).toMatch(/^\/settings/);
		expect(result.details).toBe('Opaque provider detail');
	});

	it('distinguishes quota from rate limit and per-request output from run budget', () => {
		expect(presentError('HTTP 429: insufficient_quota').title).toContain('allowance');
		expect(presentError('Brave search HTTP 429: too many requests', 'search').title).toContain('limited requests');
		expect(presentError('finish_reason: length').action).toContain('Run cost and time limits are separate');
		expect(presentError('HTTP 429', 'run', 'quota').title).toContain('allowance');
	});

	it('gives local storage, export, network, and inactive-run recommendations', () => {
		expect(presentError('Permission denied', 'export').action).toContain('writable export folder');
		expect(presentError('sqlite: database is locked', 'history').title).toContain('Local data');
		expect(presentError('Connection refused', 'fetch').action).toContain('proxy');
		expect(presentError('Could not cancel: Run is not active', 'control').action).toContain('Run History');
		expect(presentError('Run aabb-1234 is not active', 'control').action).toContain('Run History');
		expect(presentError('Run aabb-1234 is no longer active', 'control').action).toContain('Run History');
	});

	it('does not invent an explanation for unknown failures', () => {
		const result = presentError(new Error('Strange opaque condition ZX-17'), 'export');
		expect(result.title).toBe('An unexpected error occurred');
		expect(result.cause).toContain('does not identify a specific cause');
		expect(result.action).toContain('exporting');
		expect(result.settingsHref).toBeUndefined();
		expect(result.details).toBe('Strange opaque condition ZX-17');
	});
});
