import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/svelte';
import { get } from 'svelte/store';
import SettingsPage from '../routes/settings/+page.svelte';
import { settings } from '$lib/stores/settings';

const thinking = () => screen.getByRole('combobox', { name: 'Thinking' });
const provider = () => screen.getByRole('combobox', { name: 'Provider' });
const save = () => screen.getByRole('button', { name: 'Save' });
async function openAdvanced(section: string) {
	const element = document.getElementById(`settings-${section}`)!;
	const toggle = within(element).getByRole('button', { name: /Advanced/ });
	if (toggle.getAttribute('aria-expanded') !== 'true') await fireEvent.click(toggle);
}

describe('LLM provider settings', () => {
	beforeEach(async () => {
		await settings.load();
	});
	afterEach(() => {
		cleanup();
		vi.restoreAllMocks();
	});

	it('keeps reasoning changes local until Save and restores the saved choice', async () => {
		const page = render(SettingsPage);
		expect(thinking()).toHaveValue('auto');
		await fireEvent.change(thinking(), { target: { value: 'high' } });
		expect(get(settings).get('llm_reasoning_effort')).toBeUndefined();
		expect(screen.getByText(/Auto turns thinking off/)).toHaveTextContent('Provider default leaves thinking unchanged');
		await fireEvent.click(save());
		await waitFor(() => expect(get(settings).get('llm_reasoning_effort')).toBe('high'));
		page.unmount();
		render(SettingsPage);
		expect(thinking()).toHaveValue('high');
	});

	it('saves and restores the search fallback choice without clearing provider keys', async () => {
		await settings.saveMany({ brave_api_key: 'fixture-brave', serper_api_key: 'fixture-serper' });
		const page = render(SettingsPage);
		const fallback = screen.getByRole('switch', { name: 'Use the other provider as backup' });
		expect(fallback).toHaveAttribute('aria-checked', 'true');
		await fireEvent.click(fallback);
		expect(get(settings).get('search_fallback_enabled')).toBeUndefined();
		await fireEvent.click(save());
		await waitFor(() => expect(get(settings).get('search_fallback_enabled')).toBe('false'));
		page.unmount();
		render(SettingsPage);
		expect(screen.getByRole('switch', { name: 'Use the other provider as backup' })).toHaveAttribute('aria-checked', 'false');
		expect(screen.getByLabelText('Brave Search API key')).toHaveValue('fixture-brave');
		expect(screen.getByLabelText('Serper API key')).toHaveValue('fixture-serper');
	});

	it('explains unsupported GPT-OSS effort choices only for Ollama providers', async () => {
		render(SettingsPage);
		await fireEvent.change(provider(), { target: { value: 'ollama' } });
		await fireEvent.input(screen.getByRole('combobox', { name: 'Model' }), { target: { value: 'gpt-oss:120b' } });
		expect(within(thinking()).getByRole('option', { name: 'Off' })).toBeDisabled();
		expect(within(thinking()).getByRole('option', { name: 'Max' })).toBeDisabled();
		expect(screen.getByText(/GPT-OSS cannot use Off or Max/)).toBeInTheDocument();
		await fireEvent.change(thinking(), { target: { value: 'low' } });
		await fireEvent.change(provider(), { target: { value: 'openrouter' } });
		expect(within(thinking()).getByRole('option', { name: 'Off' })).toBeEnabled();
		expect(within(thinking()).getByRole('option', { name: 'Max' })).toBeEnabled();
	});

	it('shows save failures and keeps unsaved edits available for retry', async () => {
		render(SettingsPage);
		await fireEvent.change(thinking(), { target: { value: 'medium' } });
		vi.spyOn(settings, 'saveMany').mockRejectedValueOnce(new Error('sqlite: database is locked'));
		await fireEvent.click(save());
		expect(await screen.findByRole('alert')).toHaveTextContent('Local data could not be accessed');
		expect(thinking()).toHaveValue('medium');
		expect(get(settings).get('llm_reasoning_effort')).toBeUndefined();
		await fireEvent.click(save());
		await waitFor(() => expect(get(settings).get('llm_reasoning_effort')).toBe('medium'));
		expect(screen.queryByRole('alert')).not.toBeInTheDocument();
	});

	it('selects an OpenRouter model from the filtered catalog and saves its ID', async () => {
		const page = render(SettingsPage);
		const input = screen.getByRole('combobox', { name: 'Model' });
		await fireEvent.focus(input);
		await screen.findByRole('option', { name: 'anthropic/claude-test' });
		await fireEvent.input(input, { target: { value: 'GPT-TEST' } });
		expect(within(screen.getByRole('listbox', { name: 'OpenRouter models' })).getAllByRole('option')).toHaveLength(1);
		expect(get(settings).get('openrouter_model')).toBe('openai/gpt-4.1-mini');
		await fireEvent.click(screen.getByRole('option', { name: 'openai/gpt-test' }));
		await fireEvent.click(save());
		await waitFor(() => expect(save()).toBeDisabled());
		expect(get(settings).get('openrouter_model')).toBe('openai/gpt-test');
		page.unmount();
		render(SettingsPage);
		expect(screen.getByRole('combobox', { name: 'Model' })).toHaveValue('openai/gpt-test');
	});

	it('switches provider fields and preserves edits when saving and reopening', async () => {
		const page = render(SettingsPage);
		expect(screen.getByLabelText('API key')).toBeInTheDocument();
		expect(screen.queryByLabelText('Server URL')).not.toBeInTheDocument();

		await fireEvent.change(provider(), { target: { value: 'ollama' } });
		await fireEvent.input(screen.getByRole('combobox', { name: 'Model' }), { target: { value: 'local-model' } });
		expect(screen.getByLabelText('Server URL')).toHaveValue('http://localhost:11434');

		await fireEvent.change(provider(), { target: { value: 'ollama_cloud' } });
		const cloudKey = screen.getByLabelText('API key');
		expect(cloudKey).toHaveAttribute('type', 'password');
		await fireEvent.input(cloudKey, { target: { value: 'test-cloud-key' } });
		await fireEvent.input(screen.getByRole('combobox', { name: 'Model' }), { target: { value: 'cloud-model' } });
		await fireEvent.click(await screen.findByRole('option', { name: 'cloud-model' }));

		await fireEvent.change(provider(), { target: { value: 'openai_compatible' } });
		await fireEvent.input(screen.getByLabelText('API base URL'), { target: { value: 'http://localhost:8080/v1' } });
		await fireEvent.input(screen.getByRole('combobox', { name: 'Model' }), { target: { value: 'llama-model' } });
		await openAdvanced('llm');
		await fireEvent.click(screen.getByRole('switch', { name: 'JSON mode' }));
		expect(screen.getByLabelText('API key')).toHaveAttribute('type', 'password');

		await fireEvent.change(provider(), { target: { value: 'ollama' } });
		expect(screen.getByRole('combobox', { name: 'Model' })).toHaveValue('local-model');
		await fireEvent.change(provider(), { target: { value: 'openai_compatible' } });
		await fireEvent.click(save());
		await waitFor(() => expect(save()).toBeDisabled());

		const saved = get(settings);
		expect(saved.get('llm_provider')).toBe('openai_compatible');
		expect(saved.get('ollama_model')).toBe('local-model');
		expect(saved.get('ollama_cloud_api_key')).toBe('test-cloud-key');
		expect(saved.get('ollama_cloud_model')).toBe('cloud-model');
		expect(saved.get('openai_json_mode')).toBe('false');
		page.unmount();
		render(SettingsPage);
		expect(screen.getByLabelText('API base URL')).toHaveValue('http://localhost:8080/v1');
		expect(screen.getByRole('combobox', { name: 'Model' })).toHaveValue('llama-model');
	});
});

describe('settings save transactions', () => {
	beforeEach(async () => {
		await settings.load();
	});
	afterEach(() => {
		cleanup();
		vi.restoreAllMocks();
	});

	it('keeps an edit made during Save dirty and saves its latest value on retry', async () => {
		render(SettingsPage);
		await fireEvent.change(thinking(), { target: { value: 'low' } });
		let finish!: () => void;
		const original = settings.saveMany;
		vi.spyOn(settings, 'saveMany').mockImplementationOnce(async (values) => {
			await new Promise<void>((resolve) => {
				finish = resolve;
			});
			await original(values);
		});
		await fireEvent.click(save());
		await fireEvent.change(thinking(), { target: { value: 'high' } });
		finish();
		await waitFor(() => expect(save()).toBeEnabled());
		expect(thinking()).toHaveValue('high');
		expect(get(settings).get('llm_reasoning_effort')).toBe('low');
		await fireEvent.click(save());
		await waitFor(() => expect(get(settings).get('llm_reasoning_effort')).toBe('high'));
	});

	it('saves all changes or none and discards to the last saved state', async () => {
		render(SettingsPage);
		await fireEvent.change(thinking(), { target: { value: 'low' } });
		await openAdvanced('llm');
		await fireEvent.input(screen.getByRole('spinbutton', { name: 'Max output tokens' }), { target: { value: '8192' } });
		const saveMany = vi.spyOn(settings, 'saveMany').mockRejectedValueOnce(new Error('sqlite: database is locked'));
		await fireEvent.click(save());
		await screen.findByRole('alert');
		expect(saveMany).toHaveBeenCalledWith({ llm_reasoning_effort: 'low', llm_max_tokens: '8192' });
		expect(get(settings).get('llm_reasoning_effort')).toBeUndefined();
		expect(screen.getByText('2 unsaved changes')).toBeInTheDocument();
		await fireEvent.click(screen.getByRole('button', { name: 'Discard' }));
		expect(thinking()).toHaveValue('auto');
		expect(save()).toBeDisabled();
	});
});
