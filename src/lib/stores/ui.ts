import { writable } from 'svelte/store';

export const sidebarCollapsed = writable(false);
export const currentTheme = writable<'cerberus' | 'catppuccin'>('cerberus');
