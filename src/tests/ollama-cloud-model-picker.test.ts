import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import OllamaCloudModelPicker from '$lib/components/settings/OllamaCloudModelPicker.svelte';
import { listOllamaCloudModels } from '$lib/api/tauri';

vi.mock('$lib/api/tauri', () => ({ listOllamaCloudModels: vi.fn() }));
const listModels = vi.mocked(listOllamaCloudModels);
const props = {
	id: 'cloud-model', value: 'saved-model', baseUrl: 'https://ollama.com', apiKey: '',
	onchange: vi.fn()
};

describe('Ollama Cloud model picker', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		listModels.mockReset().mockResolvedValue(['deepseek-v4', 'gpt-oss:120b', 'gpt-oss:20b']);
	});
	afterEach(cleanup);

	it('loads the server catalog and filters without committing the search text', async () => {
		render(OllamaCloudModelPicker, props);
		expect(screen.getByRole('status')).toHaveTextContent('Loading models');
		const input = screen.getByRole('combobox');
		expect(input).toHaveValue('saved-model');
		await fireEvent.focus(input);
		expect(await screen.findByRole('option', { name: 'deepseek-v4' })).toBeInTheDocument();
		expect(listModels).toHaveBeenCalledWith('https://ollama.com', '');
		await fireEvent.input(input, { target: { value: 'GPT-OSS' } });
		expect(screen.getAllByRole('option')).toHaveLength(2);
		expect(props.onchange).not.toHaveBeenCalled();
		await fireEvent.keyDown(input, { key: 'ArrowDown' });
		expect(input).toHaveAttribute('aria-activedescendant', 'cloud-model-options-0');
		await fireEvent.keyDown(input, { key: 'ArrowDown' });
		await fireEvent.keyDown(input, { key: 'Enter' });
		expect(props.onchange).toHaveBeenCalledExactlyOnceWith('gpt-oss:20b');
		expect(input).toHaveAttribute('aria-expanded', 'false');
	});

	it('supports mouse selection and restores the saved value on Escape or blur', async () => {
		render(OllamaCloudModelPicker, props);
		const input = screen.getByRole('combobox');
		await fireEvent.focus(input);
		await screen.findByRole('option', { name: 'deepseek-v4' });
		await fireEvent.input(input, { target: { value: 'missing-model' } });
		expect(screen.getByText('No matching models')).toBeInTheDocument();
		await fireEvent.keyDown(input, { key: 'Enter' });
		expect(props.onchange).not.toHaveBeenCalled();
		await fireEvent.keyDown(input, { key: 'Escape' });
		expect(input).toHaveValue('saved-model');
		await fireEvent.click(input);
		await fireEvent.input(input, { target: { value: 'draft' } });
		await fireEvent.blur(input);
		expect(input).toHaveValue('saved-model');
		await fireEvent.focus(input);
		await fireEvent.click(screen.getByRole('option', { name: 'deepseek-v4' }));
		expect(props.onchange).toHaveBeenCalledExactlyOnceWith('deepseek-v4');
	});

	it('shows failures and empty catalogs and lets the user retry', async () => {
		listModels.mockRejectedValueOnce('HTTP 503').mockResolvedValueOnce([]);
		render(OllamaCloudModelPicker, props);
		await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('HTTP 503'));
		expect(screen.getByRole('combobox')).toHaveValue('saved-model');
		await fireEvent.click(screen.getByRole('button', { name: 'Refresh' }));
		await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('The server returned no models'));
		await fireEvent.click(screen.getByRole('button', { name: 'Refresh' }));
		await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('3 models available'));
		expect(props.onchange).not.toHaveBeenCalled();
	});

	it('uses edited URL and key and ignores an older response', async () => {
		let finishOld!: (models: string[]) => void;
		listModels.mockImplementationOnce(() => new Promise((resolve) => { finishOld = resolve; }));
		const page = render(OllamaCloudModelPicker, props);
		await waitFor(() => expect(listModels).toHaveBeenCalledTimes(1));
		await page.rerender({ baseUrl: 'https://new-server.example/api/', apiKey: 'new-key' });
		await waitFor(() => expect(listModels).toHaveBeenCalledWith('https://new-server.example/api/', 'new-key'));
		await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('3 models available'));
		finishOld(['outdated-model']);
		await fireEvent.focus(screen.getByRole('combobox'));
		expect(screen.queryByRole('option', { name: 'outdated-model' })).not.toBeInTheDocument();
		expect(screen.getAllByRole('option')).toHaveLength(3);
	});
});
