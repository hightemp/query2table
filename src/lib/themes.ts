// Color themes. src/app.css defines the same palettes as `:root[data-theme='<id>']` blocks
// (the defaults as `:root` and `:root.dark`); a test keeps the two in sync.

export type ThemeMode = 'light' | 'dark';

export const COLOR_TOKENS = [
	'--app-bg',
	'--app-panel',
	'--app-subtle',
	'--app-border',
	'--app-text',
	'--app-muted',
	'--app-accent',
	'--app-accent-solid',
	'--app-accent-solid-hover',
	'--app-on-accent',
	'--app-success',
	'--app-warning',
	'--app-danger',
] as const;

export type ColorToken = (typeof COLOR_TOKENS)[number];

export interface Theme {
	id: string;
	/** Proper name, shown as is in every language. */
	name: string;
	mode: ThemeMode;
	colors: Record<ColorToken, string>;
	/** Colors shown on the preview card: background, panel, accent, text. */
	swatches: [string, string, string, string];
}

function theme(id: string, name: string, mode: ThemeMode, colors: Record<ColorToken, string>): Theme {
	return {
		id,
		name,
		mode,
		colors,
		swatches: [colors['--app-bg'], colors['--app-panel'], colors['--app-accent-solid'], colors['--app-text']],
	};
}

function palette(
	bg: string,
	panel: string,
	subtle: string,
	border: string,
	text: string,
	muted: string,
	accent: string,
	solid: string,
	solidHover: string,
	onAccent: string,
	success: string,
	warning: string,
	danger: string
): Record<ColorToken, string> {
	return {
		'--app-bg': bg,
		'--app-panel': panel,
		'--app-subtle': subtle,
		'--app-border': border,
		'--app-text': text,
		'--app-muted': muted,
		'--app-accent': accent,
		'--app-accent-solid': solid,
		'--app-accent-solid-hover': solidHover,
		'--app-on-accent': onAccent,
		'--app-success': success,
		'--app-warning': warning,
		'--app-danger': danger,
	};
}

// prettier-ignore
export const THEMES: Theme[] = [
	theme('default-light', 'Default', 'light', palette('#f5f7fa', '#ffffff', '#eef1f6', '#dce2eb', '#182338', '#56657b', '#1764d8', '#1764d8', '#1255bb', '#ffffff', '#12804a', '#925400', '#c4231b')),
	theme('solarized-light', 'Solarized Light', 'light', palette('#eee8d5', '#fdf6e3', '#f5efdc', '#d6cfb9', '#073642', '#4f6168', '#1d6a9e', '#1d6a9e', '#175a87', '#ffffff', '#5c6a00', '#8a5a00', '#b3261e')),
	theme('nord-light', 'Nord Light', 'light', palette('#e5e9f0', '#f6f8fb', '#eceff4', '#d0d7e2', '#2e3440', '#4c566a', '#3d6290', '#4a6f9c', '#3d6290', '#ffffff', '#3f6e2e', '#875a00', '#a83a45')),
	theme('gruvbox-light', 'Gruvbox Light', 'light', palette('#f2e5bc', '#fbf1c7', '#ebdbb2', '#d5c4a1', '#3c3836', '#5f5650', '#076678', '#076678', '#05505e', '#ffffff', '#5f5c0b', '#8a4f02', '#9d0006')),
	theme('rose-pine-dawn', 'Rosé Pine Dawn', 'light', palette('#faf4ed', '#fffaf3', '#f2e9e1', '#dfdad9', '#575279', '#68647f', '#286983', '#286983', '#1f566c', '#ffffff', '#2f6b4f', '#8f5a12', '#a14f67')),
	theme('default-dark', 'Default', 'dark', palette('#11151c', '#191f29', '#222b38', '#343e4e', '#e6ebf3', '#a0aec2', '#79afff', '#2f6fd6', '#3d7ce3', '#ffffff', '#4ac38a', '#e3b341', '#ff7b72')),
	theme('solarized-dark', 'Solarized Dark', 'dark', palette('#002b36', '#073642', '#0c4250', '#1c5563', '#eee8d5', '#a3b1b1', '#5eaee8', '#1f6fa8', '#2a7cb8', '#ffffff', '#a8bd2a', '#d6a520', '#ff7a6e')),
	theme('nord', 'Nord', 'dark', palette('#2e3440', '#3b4252', '#434c5e', '#4c566a', '#eceff4', '#c0c8d6', '#88c0d0', '#88c0d0', '#9dcbd9', '#2e3440', '#a3be8c', '#ebcb8b', '#f0959d')),
	theme('dracula', 'Dracula', 'dark', palette('#21222c', '#282a36', '#343746', '#44475a', '#f8f8f2', '#b4bad8', '#bd93f9', '#bd93f9', '#cba8fa', '#21222c', '#50fa7b', '#f1fa8c', '#ff6e6e')),
	theme('gruvbox-dark', 'Gruvbox Dark', 'dark', palette('#1d2021', '#282828', '#32302f', '#504945', '#ebdbb2', '#bdae93', '#83a598', '#83a598', '#97b4a8', '#1d2021', '#b8bb26', '#fabd2f', '#fe6a52')),
	theme('rose-pine', 'Rosé Pine', 'dark', palette('#191724', '#1f1d2e', '#26233a', '#403d52', '#e0def4', '#a5a1c0', '#c4a7e7', '#c4a7e7', '#d2bcef', '#191724', '#9ccfd8', '#f6c177', '#eb6f92')),
];

export const DEFAULT_THEME: Record<ThemeMode, string> = { light: 'default-light', dark: 'default-dark' };

export function themeById(id: string): Theme | undefined {
	return THEMES.find((theme) => theme.id === id);
}

/** A saved theme id if it is one of `mode`, otherwise that mode's default. */
export function themeFor(mode: ThemeMode, id: string | null | undefined): string {
	return id && themeById(id)?.mode === mode ? id : DEFAULT_THEME[mode];
}

/** The theme shown for a light/dark/system preference and the chosen pair. */
export function resolveTheme(
	preference: 'light' | 'dark' | 'system',
	light: string,
	dark: string,
	systemDark: boolean
): string {
	const mode = preference === 'system' ? (systemDark ? 'dark' : 'light') : preference;
	return mode === 'dark' ? themeFor('dark', dark) : themeFor('light', light);
}

export type UiScale = 'compact' | 'normal' | 'large';
export const UI_SCALES: UiScale[] = ['compact', 'normal', 'large'];
/** Multiplier applied to the type scale. */
export const UI_SCALE_FACTOR: Record<UiScale, number> = { compact: 0.9, normal: 1, large: 1.15 };

export function isUiScale(value: unknown): value is UiScale {
	return value === 'compact' || value === 'normal' || value === 'large';
}
