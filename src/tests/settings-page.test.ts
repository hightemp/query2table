import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { get } from 'svelte/store';
import SettingsPage from '../routes/settings/+page.svelte';
import { settings } from '$lib/stores/settings';

describe('LLM provider settings', () => {
	beforeEach(async () => {
		await settings.load();
	});
	afterEach(cleanup);

	it('switches provider fields and preserves edits when saving and reopening', async () => {
		const page = render(SettingsPage);
		const provider = screen.getByRole('combobox', { name: /^Provider / });
		expect(screen.getByLabelText(/^OpenRouter API Key /)).toBeInTheDocument();
		expect(screen.queryByLabelText(/^Ollama URL /)).not.toBeInTheDocument();

		await fireEvent.change(provider, { target: { value: 'ollama' } });
		await fireEvent.input(screen.getByLabelText(/^Ollama Model /), { target: { value: 'local-model' } });
		expect(screen.queryByLabelText(/^OpenRouter API Key /)).not.toBeInTheDocument();

		await fireEvent.change(provider, { target: { value: 'ollama_cloud' } });
		const cloudKey = screen.getByLabelText(/^Ollama Cloud API Key /);
		expect(cloudKey).toHaveAttribute('type', 'password');
		await fireEvent.input(cloudKey, { target: { value: 'test-cloud-key' } });
		await fireEvent.input(screen.getByLabelText(/^Ollama Cloud Model /), { target: { value: 'cloud-model' } });

		await fireEvent.change(provider, { target: { value: 'openai_compatible' } });
		await fireEvent.input(screen.getByLabelText(/^API Base URL /), { target: { value: 'http://localhost:8080/v1' } });
		await fireEvent.input(screen.getByLabelText(/^Model Required /), { target: { value: 'llama-model' } });
		await fireEvent.change(screen.getByLabelText(/^JSON Mode /), { target: { value: 'false' } });
		expect(screen.getByLabelText(/^API Key \(optional\)/)).toHaveAttribute('type', 'password');

		await fireEvent.change(provider, { target: { value: 'ollama' } });
		expect(screen.getByLabelText(/^Ollama Model /)).toHaveValue('local-model');
		await fireEvent.change(provider, { target: { value: 'openai_compatible' } });
		await fireEvent.click(screen.getByRole('button', { name: /^Save \(/ }));
		await waitFor(() => expect(screen.queryByRole('button', { name: /^Sav/ })).not.toBeInTheDocument());

		const saved = get(settings);
		expect(saved.get('llm_provider')).toBe('openai_compatible');
		expect(saved.get('ollama_model')).toBe('local-model');
		expect(saved.get('ollama_cloud_api_key')).toBe('test-cloud-key');
		expect(saved.get('ollama_cloud_model')).toBe('cloud-model');
		expect(saved.get('openai_json_mode')).toBe('false');
		page.unmount();
		render(SettingsPage);
		expect(screen.getByLabelText(/^API Base URL /)).toHaveValue('http://localhost:8080/v1');
		expect(screen.getByLabelText(/^Model Required /)).toHaveValue('llama-model');
	});
});
