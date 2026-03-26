<script lang="ts">
	let query = $state('');
	let isRunning = $state(false);

	function handleSubmit(e: Event) {
		e.preventDefault();
		if (!query.trim()) return;
		// TODO: invoke run_query command
		isRunning = true;
	}
</script>

<div class="query-page">
	<h1>New Research Query</h1>
	<p class="subtitle">Describe what you want to research. Query2Table will search, extract, and organize results into a structured table.</p>

	<form class="query-form" onsubmit={handleSubmit}>
		<textarea
			class="query-input"
			bind:value={query}
			placeholder="e.g. Find all YC-backed AI startups from 2024 with their funding amount, CEO name, and website..."
			rows={4}
			disabled={isRunning}
		></textarea>
		<div class="query-actions">
			<button type="submit" class="btn-primary" disabled={!query.trim() || isRunning}>
				{isRunning ? 'Running...' : 'Start Research'}
			</button>
		</div>
	</form>

	{#if isRunning}
		<div class="status-panel">
			<p>Pipeline running... Results will appear below.</p>
		</div>
	{/if}
</div>

<style>
	.query-page {
		max-width: 800px;
		margin: 0 auto;
	}

	h1 {
		font-size: 1.8rem;
		font-weight: 700;
		margin-bottom: 8px;
	}

	.subtitle {
		color: var(--color-surface-500);
		margin-bottom: 24px;
	}

	.query-form {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.query-input {
		width: 100%;
		padding: 12px 16px;
		border: 2px solid var(--color-surface-300);
		border-radius: 8px;
		background: var(--color-surface-50);
		color: inherit;
		font-size: 1rem;
		font-family: inherit;
		resize: vertical;
		transition: border-color 0.15s;
	}

	.query-input:focus {
		outline: none;
		border-color: var(--color-primary-500);
	}

	.query-actions {
		display: flex;
		justify-content: flex-end;
	}

	.btn-primary {
		padding: 10px 24px;
		background: var(--color-primary-500);
		color: white;
		border: none;
		border-radius: 8px;
		font-size: 1rem;
		font-weight: 600;
		cursor: pointer;
		transition: background 0.15s;
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--color-primary-600);
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.status-panel {
		margin-top: 24px;
		padding: 16px;
		border: 1px solid var(--color-surface-300);
		border-radius: 8px;
		background: var(--color-surface-100);
	}
</style>
