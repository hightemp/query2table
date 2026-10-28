<script lang="ts">
	import { t } from '$lib/i18n';
</script>

<details class="cookies-help">
	<summary>{t('components.cookiesHelp.title')}</summary>
	<ol>
		<li>{t('components.cookiesHelp.signIn')}</li>
		<li>{t('components.cookiesHelp.browser')}</li>
		<li>{t('components.cookiesHelp.file')}</li>
		<li>{t('components.cookiesHelp.locked')}</li>
		<li>{t('components.cookiesHelp.account')}</li>
	</ol>
</details>

<style>
	.cookies-help {
		margin: 4px 0 12px;
		font-size: var(--app-text-sm);
	}
	.cookies-help summary {
		cursor: pointer;
		color: var(--app-accent);
	}
	.cookies-help ol {
		margin: 8px 0 0;
		padding-left: 1.4em;
		display: grid;
		gap: 4px;
		color: var(--app-muted);
		max-width: 75ch;
	}
</style>
