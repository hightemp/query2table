import '@testing-library/jest-dom/vitest';

// Mock Tauri APIs for testing
const mockInvoke = async (cmd: string, _args?: Record<string, unknown>) => {
	switch (cmd) {
		case 'get_settings':
			return [
				{ key: 'llm_provider', value: 'openrouter' },
				{ key: 'openrouter_model', value: 'openai/gpt-4.1-mini' },
			];
		case 'get_setting':
			return null;
		case 'update_setting':
			return undefined;
		default:
			return undefined;
	}
};

const mockListen = async (_event: string, _handler: (event: unknown) => void) => {
	return () => {};
};

vi.mock('@tauri-apps/api/core', () => ({
	invoke: mockInvoke,
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: mockListen,
}));
