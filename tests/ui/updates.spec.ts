import { test, expect } from './fixtures';
import type { Page } from '@playwright/test';

const update = {
	version: '1.0.0',
	current_version: '0.9.0',
	notes: '### Added\n\n- App updates with signature checks',
	date: 1_792_490_400,
	installer: 'appimage',
};

/** The startup check finds `found`; set before the app loads. */
async function withUpdate(page: Page, found: Record<string, unknown> | null, result: unknown = 'restart') {
	await page.addInitScript(
		([found, result]) => {
			const fixture = (window as any).__uiFixture;
			fixture.update = found;
			fixture.updateResult = result;
		},
		[found, result] as const
	);
}

test('A found update is announced, shows its changes and installs with progress', async ({ page }) => {
	await withUpdate(page, update);
	await page.goto('/');
	const notice = page.getByRole('button', { name: 'Version 1.0.0 available' });
	await expect(notice).toBeVisible();
	await notice.click();

	const dialog = page.getByRole('dialog', { name: 'Update to 1.0.0' });
	await expect(dialog).toContainText('Installed 0.9.0 → new 1.0.0');
	await expect(dialog.getByRole('region', { name: 'What’s new' }).getByRole('heading', { name: 'Added' })).toBeVisible();
	await expect(dialog).toContainText('App updates with signature checks');
	await expect(dialog).not.toContainText('administrator password');

	await dialog.getByRole('button', { name: 'Update' }).click();
	await expect(dialog.getByRole('progressbar', { name: 'Downloading the update…' })).toHaveAttribute('aria-valuenow', '50');
	await expect(dialog).toContainText('4.0 of 8.0 MB downloaded');
	await expect(dialog.getByRole('button', { name: 'Later' })).toBeDisabled();
	await expect(dialog).toContainText('The update is installed. Restart the app to use it.');
	await dialog.getByRole('button', { name: 'Restart now' }).click();
	await expect.poll(() => page.evaluate(() => (window as any).__uiFixture.restarted)).toBe(true);
});

test('A skipped version is not announced again', async ({ page }) => {
	await withUpdate(page, update);
	await page.goto('/');
	await page.getByRole('button', { name: 'Version 1.0.0 available' }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Skip this version' }).click();
	await expect(page.getByRole('dialog')).toBeHidden();
	await expect(page.getByRole('button', { name: 'Version 1.0.0 available' })).toBeHidden();

	// Checking by hand still offers it.
	await page.goto('/settings');
	await page.getByRole('button', { name: 'Check for updates' }).click();
	await expect(page.getByRole('dialog', { name: 'Update to 1.0.0' })).toBeVisible();
});

test('A failed signature check is reported and nothing is installed', async ({ page }) => {
	await withUpdate(page, { ...update, installer: 'deb' }, { kind: 'signature', message: 'The signature does not match' });
	await page.goto('/');
	await page.getByRole('button', { name: 'Version 1.0.0 available' }).click();
	const dialog = page.getByRole('dialog', { name: 'Update to 1.0.0' });
	await expect(dialog).toContainText('Installing the package asks for the administrator password.');
	await dialog.getByRole('button', { name: 'Update' }).click();
	const alert = dialog.getByRole('alert');
	await expect(alert).toContainText('The update’s signature did not check out');
	await expect(alert).toContainText('Nothing was installed.');
	await alert.getByText('Technical details').click();
	await expect(alert).toContainText('The signature does not match');
	await expect(dialog.getByRole('button', { name: 'Try again' })).toBeEnabled();
	await expect(dialog.getByRole('button', { name: 'Restart now' })).toHaveCount(0);
});

test('Settings show the version and check for updates by hand', async ({ page }) => {
	await page.goto('/settings');
	await expect(page.getByText('Query2Table 0.9.0')).toBeVisible();
	await expect(page.getByRole('switch', { name: 'Check for updates at startup' })).toBeChecked();
	await page.getByRole('button', { name: 'Check for updates' }).click();
	await expect(page.getByText('You have the latest version.')).toBeVisible();

	await page.evaluate(() => ((window as any).__uiFixture.updateCheckError = { kind: 'check', message: 'HTTP 503' }));
	await page.getByRole('button', { name: 'Check for updates' }).click();
	await expect(page.getByText('Could not check for updates: HTTP 503')).toBeVisible();
	await expect(page.getByRole('dialog')).toHaveCount(0);
});

test.describe('on Android', () => {
	test.use({ userAgent: 'Mozilla/5.0 (Linux; Android 15; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Mobile Safari/537.36' });

	test('the APK installs after the phone allows it', async ({ page }) => {
		await page.setViewportSize({ width: 390, height: 800 });
		await withUpdate(page, { ...update, installer: 'apk' }, 'permission');
		await page.goto('/');
		// The phone layout marks Settings instead of showing the sidebar notice.
		await expect(page.locator('.update-dot')).toHaveCount(1);
		await page.getByRole('link', { name: 'Settings' }).click();
		await page.getByRole('button', { name: 'Version 1.0.0 available' }).click();
		const dialog = page.getByRole('dialog', { name: 'Update to 1.0.0' });
		await dialog.getByRole('button', { name: 'Update' }).click();
		await expect(dialog).toContainText('Allow Query2Table to install apps');
		await page.evaluate(() => ((window as any).__uiFixture.updateResult = 'installer'));
		await dialog.getByRole('button', { name: 'Try again' }).click();
		await expect(dialog).toContainText('The Android installer is open: confirm the update there.');
		expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(390);
	});
});
