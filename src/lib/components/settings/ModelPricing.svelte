<script lang="ts">
	import { pricingScope } from '$lib/utils/pricing';
	let {
		provider,
		endpoint,
		model,
		value,
		onchange,
	}: {
		provider: string;
		endpoint: string;
		model: string;
		value: string;
		onchange: (value: string) => void;
	} = $props();
	let scope = $derived(pricingScope(provider, endpoint, model));
	let overrides = $derived.by<Record<string, any>>(() => {
		try {
			const parsed = JSON.parse(value || '{}');
			return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? parsed : {};
		} catch {
			return {};
		}
	});
	let rate = $derived(overrides[scope]);
	function toggle(enabled: boolean) {
		const next = { ...overrides };
		if (enabled)
			next[scope] = {
				input_per_million: null,
				output_per_million: null,
				cached_input_per_million: null,
				per_request: 0,
				source: 'Manual model rates',
				credit_based:
					provider === 'ollama_cloud' || model.endsWith(':cloud') || model.endsWith('-cloud'),
			};
		else delete next[scope];
		onchange(JSON.stringify(next));
	}
	function edit(field: string, raw: string) {
		const number = raw.trim() === '' ? null : Number.isFinite(Number(raw)) ? Number(raw) : raw;
		onchange(JSON.stringify({ ...overrides, [scope]: { ...rate, [field]: number } }));
	}
</script>

<div class="model-pricing" id="model-pricing">
	<h3>Usage and model pricing</h3>
	<p>Changes apply to new runs after Save. Saved run costs retain their original rates.</p>
	<p>
		Provider-reported cost takes priority. Otherwise, OpenRouter and Ollama Cloud use published
		model rates when available. Calculated amounts are estimates; unavailable prices stay unknown.
	</p>
	<label class="pricing-toggle"
		><input
			type="checkbox"
			checked={!!rate}
			disabled={!model.trim()}
			onchange={(event) => toggle(event.currentTarget.checked)}
		/>Use custom rates for this model and endpoint</label
	>
	{#if rate}
		<p class="scope">{model} · {provider}</p>
		<div class="rate-fields">
			<label
				>Input USD / 1M tokens<input
					class="input"
					required
					type="number"
					min="0"
					step="any"
					value={rate.input_per_million ?? ''}
					oninput={(event) => edit('input_per_million', event.currentTarget.value)}
				/></label
			>
			<label
				>Output USD / 1M tokens<input
					class="input"
					required
					type="number"
					min="0"
					step="any"
					value={rate.output_per_million ?? ''}
					oninput={(event) => edit('output_per_million', event.currentTarget.value)}
				/></label
			>
			<label
				>Cached input USD / 1M tokens (optional)<input
					class="input"
					type="number"
					min="0"
					step="any"
					value={rate.cached_input_per_million ?? ''}
					oninput={(event) => edit('cached_input_per_million', event.currentTarget.value)}
				/></label
			>
		</div>
		<p>
			Enter both input and output rates. An explicit 0 means free; an empty optional cache rate uses
			the input rate for estimates. Rates for other models are kept separately.
		</p>
	{/if}
</div>

<style>
	.model-pricing {
		border-top: 1px solid var(--app-border);
		padding-top: 16px;
		margin-top: 20px;
	}
	h3 {
		font-size: var(--app-text-base);
		font-weight: 600;
	}
	p {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		margin: 6px 0 10px;
	}
	.pricing-toggle {
		display: flex;
		gap: 8px;
		align-items: center;
	}
	.rate-fields {
		display: flex;
		gap: 12px;
		flex-wrap: wrap;
	}
	.rate-fields label {
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		flex: 1 1 180px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.rate-fields input {
		margin-top: 5px;
	}
	.scope {
		overflow-wrap: anywhere;
	}
</style>
