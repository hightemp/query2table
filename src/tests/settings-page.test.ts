import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { get } from 'svelte/store';
import SettingsPage from '../routes/settings/+page.svelte';
import { settings } from '$lib/stores/settings';

describe('LLM provider settings', () => {
	beforeEach(async () => {
		await settings.load();
	});
	afterEach(() => { cleanup(); vi.restoreAllMocks(); });

	it('keeps reasoning changes local until Save and restores the saved choice', async () => {
		const page = render(SettingsPage);
		const effort = screen.getByRole('combobox', { name: /^Thinking \/ reasoning effort / });
		expect(effort).toHaveValue('auto');
		await fireEvent.change(effort, { target: { value: 'high' } });
		expect(get(settings).get('llm_reasoning_effort')).toBeUndefined();
		expect(screen.getByText(/Auto disables thinking/)).toHaveTextContent('Provider default leaves thinking unchanged');
		await fireEvent.click(screen.getByRole('button', { name: /^Save \(/ }));
		await waitFor(() => expect(get(settings).get('llm_reasoning_effort')).toBe('high'));
		page.unmount();
		render(SettingsPage);
		expect(screen.getByRole('combobox', { name: /^Thinking \/ reasoning effort / })).toHaveValue('high');
	});

	it('explains unsupported GPT-OSS effort choices only for Ollama providers', async () => {
		render(SettingsPage);
		await fireEvent.change(screen.getByRole('combobox', { name: /^Provider / }), { target: { value: 'ollama' } });
		await fireEvent.input(screen.getByLabelText(/^Ollama Model /), { target: { value: 'gpt-oss:120b' } });
		const effort = screen.getByRole('combobox', { name: /^Thinking \/ reasoning effort / });
		expect(within(effort).getByRole('option', { name: 'Off' })).toBeDisabled();
		expect(within(effort).getByRole('option', { name: 'Max' })).toBeDisabled();
		expect(screen.getByText(/GPT-OSS cannot use Off or Max/)).toBeInTheDocument();
		await fireEvent.change(effort, { target: { value: 'low' } });
		await fireEvent.change(screen.getByRole('combobox', { name: /^Provider / }), { target: { value: 'openrouter' } });
		expect(within(effort).getByRole('option', { name: 'Off' })).toBeEnabled();
		expect(within(effort).getByRole('option', { name: 'Max' })).toBeEnabled();
	});

	it('shows save failures and keeps unsaved edits available for retry', async () => {
		render(SettingsPage);
		await fireEvent.change(screen.getByRole('combobox', { name: /^Thinking \/ reasoning effort / }), { target: { value: 'medium' } });
		vi.spyOn(settings, 'save').mockRejectedValueOnce(new Error('sqlite: database is locked'));
		await fireEvent.click(screen.getByRole('button', { name: /^Save \(/ }));
		expect(await screen.findByRole('alert')).toHaveTextContent('Local data could not be accessed');
		expect(screen.getByRole('combobox', { name: /^Thinking \/ reasoning effort / })).toHaveValue('medium');
		expect(get(settings).get('llm_reasoning_effort')).toBeUndefined();
		await fireEvent.click(screen.getByRole('button', { name: /^Save \(/ }));
		await waitFor(() => expect(get(settings).get('llm_reasoning_effort')).toBe('medium'));
		expect(screen.queryByRole('alert')).not.toBeInTheDocument();
	});

	it('selects an OpenRouter model from the filtered catalog and saves its ID', async () => {
		const page = render(SettingsPage);
		const input = screen.getByRole('combobox', { name: /^OpenRouter Model / });
		await fireEvent.focus(input);
		await screen.findByRole('option', { name: 'anthropic/claude-test' });
		await fireEvent.input(input, { target: { value: 'GPT-TEST' } });
		expect(within(screen.getByRole('listbox', { name: 'OpenRouter models' })).getAllByRole('option')).toHaveLength(1);
		expect(get(settings).get('openrouter_model')).toBe('openai/gpt-4.1-mini');
		await fireEvent.click(screen.getByRole('option', { name: 'openai/gpt-test' }));
		await fireEvent.click(screen.getByRole('button', { name: /^Save \(/ }));
		await waitFor(() => expect(screen.queryByRole('button', { name: /^Sav/ })).not.toBeInTheDocument());
		expect(get(settings).get('openrouter_model')).toBe('openai/gpt-test');
		page.unmount();
		render(SettingsPage);
		expect(screen.getByRole('combobox', { name: /^OpenRouter Model / })).toHaveValue('openai/gpt-test');
	});

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
		await fireEvent.click(await screen.findByRole('option', { name: 'cloud-model' }));

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
