import { defineConfig } from 'vite';

export default defineConfig({
  // Project root contains index.html and run.html.
  root: '.',
  // public/ holds artifacts emitted by build.mjs (wasm + JS shims + resources).
  publicDir: 'public',
  server: {
    open: '/',
    fs: {
      // Allow serving resources/wasm from elsewhere if a future build script
      // decides to symlink rather than copy.
      strict: false,
    },
  },
  build: {
    rollupOptions: {
      input: {
        index: 'index.html',
        run: 'run.html',
      },
    },
  },
});
