import { afterEach, describe, expect, it } from 'vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte';
import ErrorNotice from '$lib/components/common/ErrorNotice.svelte';
import RunNotices from '$lib/components/run/RunNotices.svelte';
import { llmIssue } from './fixtures/llm-issue';

afterEach(cleanup);

describe('model issue and error display', () => {
	const unread = { ...llmIssue, code: 'invalid_response' as const, stage: 'research', attempt: 1, max_attempts: 1,
		message: 'The model response did not contain a valid research action' };

	it('stays one line until expanded and then explains each notice', async () => {
		render(RunNotices, { issues: [llmIssue], accounting: null, runStatus: 'failed' });
		const strip = screen.getByRole('region', { name: 'Run notices' });
		const toggle = within(strip).getByRole('button', { name: /1 notice/ });
		expect(toggle).toHaveAttribute('aria-expanded', 'false');
		expect(toggle).toHaveTextContent('The model reached its output limit');
		expect(screen.queryByText('The run stopped at this step.')).not.toBeInTheDocument();

		await fireEvent.click(toggle);
		const card = within(strip).getByRole('article', { name: 'The model reached its output limit' });
		expect(card).toHaveTextContent('The run stopped at this step.');
		expect(within(card).getByRole('link', { name: 'Open Settings' })).toHaveAttribute('href', '/settings#llm_max_tokens');
		// Provider, tokens and attempts are technical details, collapsed.
		const technical = within(card).getByText(/deepseek-test · Expanding search queries/);
		expect(technical).not.toBeVisible();
		await fireEvent.click(within(card).getByText('Technical details'));
		expect(technical).toBeVisible();
		expect(card).toHaveTextContent(/Requested output cap\s*4,096 tokens per request/);
		expect(card).toHaveTextContent(/Reported input\s*600 tokens/);
		expect(card).toHaveTextContent('Attempt 1 of 3');
		expect(card).toHaveTextContent('Response ended with done_reason=length');
	});

	it('counts repeated failures, names the question and keeps settled ones calm', async () => {
		render(RunNotices, {
			issues: [
				{ ...unread, turn_index: 1, outcome: 'continued' },
				...['a', 'b'].map((id) => ({ ...llmIssue, code: 'timeout' as const, stage: 'extractor', call_id: id, outcome: 'skipped' as const })),
			],
			accounting: null,
			runStatus: 'completed',
			turnCount: 2,
		});
		const strip = screen.getByRole('region', { name: 'Run notices' });
		expect(strip).toHaveAttribute('data-level', 'warning');
		await fireEvent.click(within(strip).getByRole('button', { name: /2 notices/ }));
		const pages = within(strip).getByRole('article', { name: 'The request timed out' });
		expect(pages).toHaveTextContent('×2');
		expect(pages).toHaveTextContent('Data from 2 pages was skipped.');
		const agent = within(strip).getByRole('article', { name: 'The response could not be read' });
		expect(agent).toHaveTextContent('Question 2');
		expect(agent).toHaveTextContent('The answer is not affected.');
		expect(agent).toHaveAttribute('data-level', 'info');
	});

	it('can be dismissed until something new happens', async () => {
		const dismissals: Record<string, number>[] = [];
		const view = render(RunNotices, {
			issues: [unread],
			accounting: null,
			runStatus: 'completed',
			ondismiss: (value: Record<string, number>) => dismissals.push(value),
		});
		await fireEvent.click(screen.getByRole('button', { name: 'Dismiss notices' }));
		expect(dismissals).toHaveLength(1);
		await view.rerender({ dismissed: dismissals[0] });
		const strip = screen.getByRole('region', { name: 'Run notices' });
		expect(strip).toHaveAttribute('data-dismissed', 'true');
		expect(screen.queryByRole('button', { name: 'Dismiss notices' })).not.toBeInTheDocument();
		await view.rerender({ issues: [unread, { ...llmIssue, call_id: 'new' }] });
		expect(strip).not.toHaveAttribute('data-dismissed');
	});

	it('shows nothing without notices', () => {
		render(RunNotices, { issues: [], accounting: null, runStatus: 'completed' });
		expect(screen.queryByRole('region', { name: 'Run notices' })).not.toBeInTheDocument();
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
