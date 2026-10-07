<script lang="ts">
	import { t } from '$lib/i18n';
	import { THEMES, type ThemeMode } from '$lib/themes';

	let {
		mode,
		selected,
		onselect,
	}: {
		mode: ThemeMode;
		selected: string;
		onselect: (id: string) => void;
	} = $props();

	let themes = $derived(THEMES.filter((theme) => theme.mode === mode));
	const labelId = `theme-picker-${Math.random().toString(36).slice(2)}`;
</script>

<div class="picker">
	<span class="picker-label" id={labelId}>{mode === 'light' ? t('theme.lightThemes') : t('theme.darkThemes')}</span>
	<div class="cards" role="radiogroup" aria-labelledby={labelId}>
		{#each themes as theme (theme.id)}
			{@const [bg, panel, accent, text] = theme.swatches}
			{@const name = theme.id.startsWith('default') ? t('theme.default') : theme.name}
			<label class="card" class:checked={selected === theme.id}>
				<input
					type="radio"
					name={`theme-${mode}`}
					value={theme.id}
					checked={selected === theme.id}
					onchange={() => onselect(theme.id)}
				/>
				<span class="preview" style:background={bg} aria-hidden="true">
					<span class="panel" style:background={panel}>
						<span class="line" style:background={text}></span>
						<span class="line short" style:background={text}></span>
						<span class="button" style:background={accent}></span>
					</span>
				</span>
				<span class="name">{name}</span>
			</label>
		{/each}
	</div>
</div>

<style>
	.picker {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px 0;
		border-top: 1px solid var(--app-border);
	}
	.picker-label {
		font-weight: 600;
	}
	.cards {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(124px, 1fr));
		gap: 10px;
	}
	.card {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 6px;
		border: 1px solid var(--app-border);
		border-radius: var(--app-radius);
		cursor: pointer;
	}
	.card:hover {
		border-color: var(--app-muted);
	}
	.card.checked {
		border-color: var(--app-accent);
		box-shadow: 0 0 0 1px var(--app-accent);
	}
	.card:has(input:focus-visible) {
		outline: 2px solid var(--app-accent);
		outline-offset: 2px;
	}
	/* The radio covers its card invisibly, so the whole card is the control. */
	.card input {
		position: absolute;
		inset: 0;
		margin: 0;
		opacity: 0;
		cursor: pointer;
	}
	.preview {
		display: flex;
		align-items: flex-end;
		height: 56px;
		padding: 8px 0 0 10px;
		border-radius: var(--app-radius-sm);
		overflow: hidden;
		box-shadow: inset 0 0 0 1px rgb(128 128 128 / 25%);
	}
	.panel {
		display: flex;
		flex-direction: column;
		gap: 5px;
		width: 100%;
		height: 100%;
		padding: 8px;
		border-top-left-radius: 4px;
	}
	.line {
		height: 4px;
		width: 70%;
		border-radius: 2px;
		opacity: 0.85;
	}
	.line.short {
		width: 45%;
		opacity: 0.5;
	}
	.button {
		width: 34px;
		height: 8px;
		margin-top: auto;
		border-radius: 3px;
	}
	.name {
		font-size: var(--app-text-sm);
		text-align: center;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.checked .name {
		color: var(--app-accent);
		font-weight: 600;
	}
</style>
