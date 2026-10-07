<script lang="ts">
	import { t } from '$lib/i18n';
	import NumberInput from '$lib/components/common/NumberInput.svelte';
	import Checkbox from '$lib/components/common/Checkbox.svelte';
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
	<h3>{t('pricing.title')}</h3>
	<p>{t('pricing.applies')}</p>
	<p>
		{t('pricing.priority')}
	</p>
	<span class="pricing-toggle"
		><Checkbox checked={!!rate} disabled={!model.trim()} onchange={toggle}>{t('pricing.custom')}</Checkbox></span
	>

	{#if rate}
		<p class="scope">{model} · {provider}</p>
		<div class="rate-fields">
			<label
				>{t('pricing.input')}<NumberInput
					required
					min="0"
					step="any"
					value={rate.input_per_million ?? ''}
					oninput={(event) => edit('input_per_million', event.currentTarget.value)}
				/></label
			>
			<label
				>{t('pricing.output')}<NumberInput
					required
					min="0"
					step="any"
					value={rate.output_per_million ?? ''}
					oninput={(event) => edit('output_per_million', event.currentTarget.value)}
				/></label
			>
			<label
				>{t('pricing.cached')}<NumberInput
					min="0"
					step="any"
					value={rate.cached_input_per_million ?? ''}
					oninput={(event) => edit('cached_input_per_million', event.currentTarget.value)}
				/></label
			>
		</div>
		<p>
			{t('pricing.help')}
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
	.rate-fields :global(.number-input) {
		margin-top: 5px;
	}
	.scope {
		overflow-wrap: anywhere;
	}
</style>
