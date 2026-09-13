import { afterEach, describe, expect, it } from 'vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte';
import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
import LlmIssues from '$lib/components/run/LlmIssues.svelte';
import { llmIssue } from './fixtures/llm-issue';

afterEach(cleanup);

describe('model issue and error display', () => {
	it('shows output cap, exact reported usage, stage, and a settings action after completion', () => {
		render(LlmIssues, { issues: [llmIssue], runStatus: 'completed' });
		const section = screen.getByRole('region', { name: 'Model request issues' });
		expect(section).toHaveTextContent('The run completed');
		expect(section).toHaveTextContent('deepseek-test · Expanding search queries');
		expect(section).toHaveTextContent(/Requested output cap\s*4,096 tokens per request/);
		expect(section).toHaveTextContent(/Reported input\s*600 tokens/);
		expect(section).toHaveTextContent(/Reported output\s*4,096 tokens/);
		expect(section).not.toHaveTextContent('Reported thinking');
		expect(within(section).getByRole('link', { name: 'Open Settings' })).toHaveAttribute('href', '/settings#llm_max_tokens');
		expect(section).toHaveTextContent('Run cost and time limits are separate');
	});

	it('preserves unknown token usage and describes retries as recorded events', () => {
		render(LlmIssues, { issues: [{ ...llmIssue, code: 'rate_limit', prompt_tokens: null, completion_tokens: null,
			will_retry: true, retry_after_ms: 1500, attempt: 2 }], runStatus: 'completed' });
		expect(screen.getByText('Token usage was not reported by the provider.')).toBeInTheDocument();
		expect(screen.getByText(/Attempt 2 of 3/)).toHaveTextContent('An automatic retry was scheduled after 1.5 s');
		expect(screen.queryByText('Reported input')).not.toBeInTheDocument();
	});

	it('keeps raw errors collapsed behind technical details and updates the explanation', async () => {
		const page = render(ErrorNotice, { props: { error: 'Strange opaque condition ZX-17', context: 'catalog' } });
		expect(screen.getByRole('alert')).toHaveTextContent('An unexpected error occurred');
		const detail = screen.getByText('Strange opaque condition ZX-17');
		expect(detail).not.toBeVisible();
		await fireEvent.click(screen.getByText('Technical details'));
		expect(detail).toBeVisible();
		await page.rerender({ error: 'HTTP 401: unauthorized' });
		expect(screen.getByRole('alert')).toHaveTextContent('The provider could not authenticate');
		expect(screen.queryByText('Strange opaque condition ZX-17')).not.toBeInTheDocument();
	});
});
