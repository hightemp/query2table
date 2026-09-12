import type { LlmIssueEvent } from '$lib/types';

export const llmIssue: LlmIssueEvent = {
	run_id: 'run-1', code: 'output_limit', provider: 'ollama_cloud', model: 'deepseek-test',
	stage: 'query_expander', message: 'Response ended with done_reason=length', max_tokens: 4096,
	prompt_tokens: 600, completion_tokens: 4096, reasoning_tokens: null,
	retry_after_ms: null, attempt: 1, max_attempts: 3, will_retry: false,
};
