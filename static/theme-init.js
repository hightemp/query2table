// Applies the saved theme and interface size before the app renders, so a light theme
// never flashes dark. Keep the storage keys in sync with src/lib/stores/ui.ts.
try {
	var root = document.documentElement;
	var theme = localStorage.getItem('q2t-theme');
	var dark =
		theme === 'dark' ||
		(theme !== 'light' && window.matchMedia('(prefers-color-scheme: dark)').matches);
	root.classList.toggle('dark', dark);
	// An id of the other appearance has no effect: its colors are scoped to that id only.
	var chosen = localStorage.getItem(dark ? 'q2t-theme-dark' : 'q2t-theme-light');
	root.dataset.theme = chosen || (dark ? 'default-dark' : 'default-light');
	var scale = localStorage.getItem('q2t-ui-scale');
	if (scale) root.dataset.scale = scale;
} catch (error) {
	// Without storage the app applies the theme after loading settings.
}
