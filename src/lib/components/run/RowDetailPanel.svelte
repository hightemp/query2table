<script lang="ts">
	import type { RunRow } from '$lib/stores/run';
	import { openUrl } from '@tauri-apps/plugin-opener';

	interface Props {
		row: RunRow;
		columns: string[];
		onclose: () => void;
	}

	let { row, columns, onclose }: Props = $props();

	function isUrl(value: string): boolean {
		try {
			const u = new URL(value);
			return u.protocol === 'http:' || u.protocol === 'https:';
		} catch {
			return false;
		}
	}

	function handleLinkClick(e: MouseEvent, url: string) {
		e.preventDefault();
		e.stopPropagation();
		openUrl(url);
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="detail-overlay" onclick={onclose} onkeydown={(e) => e.key === 'Escape' && onclose()}>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="detail-panel" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
		<div class="detail-header">
			<h3>Row Details</h3>
			<button class="close-btn" onclick={onclose}>&times;</button>
		</div>

		<div class="detail-body">
			<div class="confidence-bar">
				<span>Confidence</span>
				<div class="confidence-track">
					<div class="confidence-fill" style="width: {Math.round(row.confidence * 100)}%"></div>
				</div>
				<span class="confidence-val">{Math.round(row.confidence * 100)}%</span>
			</div>

			<table class="detail-table">
				<tbody>
					{#each columns as col}
						{@const value = row.data[col] != null ? String(row.data[col]) : ''}
						<tr>
							<td class="detail-key">{col}</td>
							<td class="detail-value">
								{#if value && isUrl(value)}
									<a href={value} class="detail-link" onclick={(e) => handleLinkClick(e, value)}>{value}</a>
								{:else}
									{value || '—'}
								{/if}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
</div>

<style>
	.detail-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.4);
		display: flex;
		justify-content: flex-end;
		z-index: 100;
	}

	.detail-panel {
		width: 420px;
		max-width: 90vw;
		height: 100%;
		background: var(--color-surface-50-950);
		box-shadow: -4px 0 24px rgba(0, 0, 0, 0.2);
		display: flex;
		flex-direction: column;
		overflow-y: auto;
	}

	.detail-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-surface-300-700);
	}

	h3 {
		font-size: 1.1rem;
		font-weight: 700;
		margin: 0;
	}

	.close-btn {
		background: none;
		border: none;
		font-size: 1.5rem;
		cursor: pointer;
		color: inherit;
		padding: 0 4px;
		line-height: 1;
	}

	.detail-body {
		padding: 20px;
		flex: 1;
	}

	.confidence-bar {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-bottom: 20px;
		font-size: 0.9rem;
	}

	.confidence-track {
		flex: 1;
		height: 6px;
		background: var(--color-surface-200-800);
		border-radius: 3px;
		overflow: hidden;
	}

	.confidence-fill {
		height: 100%;
		background: var(--color-primary-500);
		border-radius: 3px;
	}

	.confidence-val {
		font-weight: 600;
		min-width: 40px;
		text-align: right;
	}

	.detail-table {
		width: 100%;
		border-collapse: collapse;
	}

	.detail-table tr {
		border-bottom: 1px solid var(--color-surface-200-800);
	}

	.detail-key {
		font-weight: 600;
		padding: 10px 12px 10px 0;
		vertical-align: top;
		white-space: nowrap;
		font-size: 0.85rem;
		color: var(--color-surface-600-400);
		width: 120px;
	}

	.detail-value {
		padding: 10px 0;
		word-break: break-word;
		font-size: 0.9rem;
	}

	.detail-link {
		color: var(--color-primary-500);
		text-decoration: none;
		cursor: pointer;
	}
	.detail-link:hover {
		text-decoration: underline;
	}
</style>
