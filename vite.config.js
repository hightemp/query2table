import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import { svelteTesting } from "@testing-library/svelte/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

/**
 * vite-plugin-svelte only has a component's CSS after compiling the component. When the
 * dev server is asked for the CSS first, it serves the raw .svelte source as CSS and caches
 * it, so rules like `label {…}` apply globally. Compile the component before its CSS loads.
 * @returns {import('vite').Plugin}
 */
function compileSvelteBeforeCss() {
  return {
    name: "compile-svelte-before-css",
    apply: "serve",
    enforce: "pre",
    load: {
      filter: { id: /\.svelte\?svelte&type=style/ },
      async handler(id) {
        const file = id.slice(0, id.indexOf("?"));
        if (!this.getModuleInfo(file)?.meta?.svelte?.css)
          // @ts-expect-error transformRequest exists on the dev environment
          await this.environment.transformRequest?.(file);
        return null;
      },
    },
  };
}

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [compileSvelteBeforeCss(), sveltekit(), svelteTesting()],
  test: {
    include: ['src/**/*.test.{js,ts}'],
    environment: 'jsdom',
    globals: true,
    setupFiles: ['src/tests/setup.ts'],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
