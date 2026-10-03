// Applies the saved theme before the app renders, so a light theme never flashes dark.
// Keep the storage key in sync with src/lib/stores/ui.ts.
try {
	var theme = localStorage.getItem('q2t-theme');
	var dark =
		theme === 'dark' ||
		(theme !== 'light' && window.matchMedia('(prefers-color-scheme: dark)').matches);
	document.documentElement.classList.toggle('dark', dark);
} catch (error) {
	// Without storage the app applies the theme after loading settings.
}
