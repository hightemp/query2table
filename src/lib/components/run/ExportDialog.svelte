<script lang="ts">
	import { save } from '@tauri-apps/plugin-dialog';
	import { DownloadIcon, XIcon } from '@lucide/svelte';
	import { exportRun } from '$lib/api/tauri';

	interface Props {
		runId: string;
		onclose: () => void;
	}

	let { runId, onclose }: Props = $props();

	let format = $state<'csv' | 'json' | 'xlsx'>('csv');
	let exporting = $state(false);
	let error = $state('');

	const formatOptions = [
		{ value: 'csv' as const, label: 'CSV', ext: 'csv', description: 'Comma-separated values' },
		{ value: 'json' as const, label: 'JSON', ext: 'json', description: 'Structured JSON array' },
		{ value: 'xlsx' as const, label: 'XLSX', ext: 'xlsx', description: 'Excel spreadsheet' },
	];

	async function handleExport() {
		error = '';
		const opt = formatOptions.find((o) => o.value === format)!;
		const filePath = await save({
			defaultPath: `query2table-export.${opt.ext}`,
			filters: [{ name: opt.label, extensions: [opt.ext] }],
		});

		if (!filePath) return; // User cancelled

		exporting = true;
		try {
			await exportRun(runId, format, filePath);
			onclose();
		} catch (e) {
			error = String(e);
		} finally {
			exporting = false;
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onclose} onkeydown={(e) => e.key === 'Escape' && onclose()}>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="dialog" onclick={(e) => e.stopPropagation()}>
		<div class="dialog-header">
			<h3>Export Results</h3>
			<button class="close-btn" onclick={onclose} aria-label="Close">
				<XIcon size={18} />
			</button>
		</div>

		<div class="dialog-body">
			<label class="field-label">Format</label>
			<div class="format-options">
				{#each formatOptions as opt}
					<label class="format-option" class:selected={format === opt.value}>
						<input type="radio" name="format" value={opt.value} bind:group={format} />
						<div class="format-info">
							<span class="format-name">{opt.label}</span>
							<span class="format-desc">{opt.description}</span>
						</div>
					</label>
				{/each}
			</div>

			{#if error}
				<p class="error-msg">{error}</p>
			{/if}
		</div>

		<div class="dialog-footer">
			<button class="btn-secondary" onclick={onclose} disabled={exporting}>Cancel</button>
			<button class="btn-primary" onclick={handleExport} disabled={exporting}>
				<DownloadIcon size={16} />
				{exporting ? 'Exporting…' : 'Export'}
			</button>
		</div>
	</div>
</div>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}

	.dialog {
		background: var(--color-surface-50-950);
		border-radius: 12px;
		width: 420px;
		max-width: 90vw;
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
	}

	.dialog-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-surface-200-800);
	}

	.dialog-header h3 {
		margin: 0;
		font-size: 1.1rem;
	}

	.close-btn {
		background: none;
		border: none;
		cursor: pointer;
		color: var(--color-surface-600-400);
		padding: 4px;
		border-radius: 4px;
	}

	.close-btn:hover {
		background: var(--color-surface-200-800);
	}

	.dialog-body {
		padding: 20px;
	}

	.field-label {
		display: block;
		font-weight: 600;
		font-size: 0.85rem;
		margin-bottom: 8px;
		color: var(--color-surface-600-400);
	}

	.format-options {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.format-option {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px;
		border: 2px solid var(--color-surface-200-800);
		border-radius: 8px;
		cursor: pointer;
		transition: border-color 0.15s;
	}

	.format-option:hover {
		border-color: var(--color-surface-400-600);
	}

	.format-option.selected {
		border-color: var(--color-primary-500);
		background: var(--color-primary-100, rgba(59, 130, 246, 0.08));
	}

	.format-option input[type='radio'] {
		accent-color: var(--color-primary-500);
	}

	.format-info {
		display: flex;
		flex-direction: column;
	}

	.format-name {
		font-weight: 600;
		font-size: 0.95rem;
	}

	.format-desc {
		font-size: 0.8rem;
		color: var(--color-surface-600-400);
	}

	.error-msg {
		color: var(--color-error-500);
		font-size: 0.85rem;
		margin-top: 12px;
	}

	.dialog-footer {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		padding: 16px 20px;
		border-top: 1px solid var(--color-surface-200-800);
	}

	.btn-primary,
	.btn-secondary {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 8px 16px;
		border-radius: 6px;
		font-size: 0.9rem;
		font-weight: 500;
		cursor: pointer;
		border: none;
	}

	.btn-primary {
		background: var(--color-primary-500);
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--color-primary-600, #2563eb);
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-secondary {
		background: var(--color-surface-200-800);
		color: inherit;
	}

	.btn-secondary:hover:not(:disabled) {
		background: var(--color-surface-300-700);
	}
</style>
