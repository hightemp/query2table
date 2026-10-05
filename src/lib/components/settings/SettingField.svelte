<script lang="ts">
	import type { Snippet } from 'svelte';
	import { EyeIcon, EyeOffIcon, RotateCcwIcon } from '@lucide/svelte';
	import { fieldDescription, fieldLabel, isDefault, optionLabel, unitLabel, type FieldDef } from '$lib/settings/schema';
	import { t } from '$lib/i18n';

	let {
		field,
		value,
		error = null,
		changed = false,
		disabledOptions = [],
		onchange,
		control,
		help,
	}: {
		field: FieldDef;
		value: string;
		error?: string | null;
		/** Edited but not saved. */
		changed?: boolean;
		disabledOptions?: string[];
		onchange: (value: string) => void;
		/** The control of a model field (a model picker). */
		control?: Snippet;
		/** Extra explanation under the control. */
		help?: Snippet;
	} = $props();

	let reveal = $state(false);
	let label = $derived(fieldLabel(field));
	let describedBy = $derived([`${field.key}-description`, error ? `${field.key}-error` : ''].filter(Boolean).join(' '));
	let canReset = $derived(!field.keepOnReset && field.kind !== 'password' && !isDefault(field, value));
	let defaultLabel = $derived(
		field.kind === 'switch'
			? t(field.default === 'true' ? 'field.defaultOn' : 'field.defaultOff')
			: field.options?.includes(field.default)
				? optionLabel(field, field.default)
				: field.default
	);
</script>

<div class="setting-field" class:changed class:invalid={!!error} data-key={field.key}>
	<div class="label-column">
		<label for={field.key} id={`${field.key}-label`}>{label}{#if changed}<span class="changed-dot" title={t('field.notSaved')} aria-hidden="true"></span>{/if}</label>
		<p class="description" id={`${field.key}-description`}>{fieldDescription(field)}</p>
	</div>
	<div class="control-column">
		<div class="control-row">
			{#if control && field.kind === 'model'}
				{@render control()}
			{:else if field.kind === 'select'}
				<select id={field.key} class="input" {value} aria-describedby={describedBy} onchange={(e) => onchange(e.currentTarget.value)}>
					{#each field.options ?? [] as option (option)}
						<option value={option} disabled={disabledOptions.includes(option)}>{optionLabel(field, option)}</option>
					{/each}
				</select>
			{:else if field.kind === 'switch'}
				<button
					id={field.key}
					type="button"
					role="switch"
					class="switch"
					aria-labelledby={`${field.key}-label`}
					aria-checked={value === 'true'}
					aria-describedby={describedBy}
					onclick={() => onchange(value === 'true' ? 'false' : 'true')}><span class="thumb"></span></button
				>
				<span class="switch-state" aria-hidden="true">{value === 'true' ? t('field.on') : t('field.off')}</span>
			{:else if field.kind === 'password'}
				<div class="password">
					<input
						id={field.key}
						class="input"
						type={reveal ? 'text' : 'password'}
						autocomplete="off"
						spellcheck="false"
						{value}
						placeholder={field.placeholder}
						aria-describedby={describedBy}
						oninput={(e) => onchange(e.currentTarget.value)}
					/>
					<button
						type="button"
						class="icon-button ghost"
						aria-label={t(reveal ? 'field.hide' : 'field.show', { name: label })}
						aria-pressed={reveal}
						onclick={() => (reveal = !reveal)}
						>{#if reveal}<EyeOffIcon size={16} />{:else}<EyeIcon size={16} />{/if}</button
					>
				</div>
			{:else if field.kind === 'range'}
				<input
					type="range"
					class="slider"
					aria-label={label}
					min={field.min}
					max={field.max}
					step={field.step}
					value={Number.isFinite(Number(value)) ? value : field.default}
					oninput={(e) => onchange(e.currentTarget.value)}
				/>
				<input
					id={field.key}
					class="input number"
					type="number"
					min={field.min}
					max={field.max}
					step={field.step}
					{value}
					aria-invalid={!!error}
					aria-describedby={describedBy}
					oninput={(e) => onchange(e.currentTarget.value)}
				/>
				{#if field.unit}<span class="unit">{unitLabel(field.unit)}</span>{/if}
			{:else if field.kind === 'number'}
				<input
					id={field.key}
					class="input number wide"
					type="number"
					min={field.min}
					max={field.max}
					step={field.step === 0.01 ? 'any' : field.step}
					{value}
					aria-invalid={!!error}
					aria-describedby={describedBy}
					oninput={(e) => onchange(e.currentTarget.value)}
				/>
				{#if field.unit}<span class="unit">{unitLabel(field.unit)}</span>{/if}
			{:else}
				<input
					id={field.key}
					class="input"
					type={field.kind === 'url' ? 'url' : 'text'}
					spellcheck="false"
					{value}
					placeholder={field.placeholder}
					aria-invalid={!!error}
					aria-describedby={describedBy}
					oninput={(e) => onchange(e.currentTarget.value)}
				/>
			{/if}
			{#if canReset}
				<button
					type="button"
					class="icon-button ghost reset"
					aria-label={t('field.reset', { name: label, value: defaultLabel })}
					title={t('field.resetHint', { value: defaultLabel })}
					onclick={() => onchange(field.default)}><RotateCcwIcon size={14} /></button
				>
			{/if}
		</div>
		{#if error}<p class="error" id={`${field.key}-error`}>{error}</p>{/if}
		{#if help}{@render help()}{/if}
	</div>
</div>

<style>
	.setting-field {
		display: grid;
		grid-template-columns: minmax(0, 2fr) minmax(0, 3fr);
		gap: 6px 24px;
		padding: 12px 0;
		border-top: 1px solid var(--app-border);
	}
	@container settings (max-width: 640px) {
		.setting-field {
			grid-template-columns: minmax(0, 1fr);
		}
	}
	label {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-weight: 600;
	}
	.changed-dot {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--app-accent);
	}
	.description {
		margin-top: 2px;
		color: var(--app-muted);
		font-size: var(--app-text-sm);
		line-height: 1.45;
	}
	.control-column {
		min-width: 0;
	}
	.control-row {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}
	.control-row > :global(select),
	.control-row > :global(input:not(.slider):not(.number)),
	.control-row > :global(.password),
	.control-row > :global(.model-picker) {
		flex: 1;
		min-width: 0;
	}
	.password {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.password input {
		flex: 1;
		min-width: 0;
	}
	.slider {
		flex: 1;
		min-width: 80px;
		accent-color: var(--app-accent);
	}
	.number {
		width: 96px;
	}
	.number.wide {
		width: 140px;
	}
	.unit {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.switch {
		position: relative;
		flex-shrink: 0;
		width: 38px;
		height: 22px;
		border-radius: var(--app-radius-pill);
		background: var(--app-border);
		transition: background 0.15s;
	}
	.switch[aria-checked='true'] {
		background: var(--app-accent-solid, var(--app-accent));
	}
	.thumb {
		position: absolute;
		top: 3px;
		left: 3px;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: #fff;
		transition: transform 0.15s;
	}
	.switch[aria-checked='true'] .thumb {
		transform: translateX(16px);
	}
	.switch-state {
		color: var(--app-muted);
		font-size: var(--app-text-sm);
	}
	.reset {
		flex-shrink: 0;
		color: var(--app-muted);
	}
	.error {
		margin-top: 4px;
		color: var(--app-danger);
		font-size: var(--app-text-sm);
	}
	.invalid :global(.input) {
		border-color: var(--app-danger);
	}
</style>
