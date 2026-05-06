import { defineConfig } from 'vite';

export default defineConfig({
  root: '.',
  // contains artifacts emitted by build.mjs (wasm + JS shims + resources)
  publicDir: 'public',
  server: {
    open: '/',
    fs: {
      // Allow serving wasm from elsewhere (e.g. if we symlink in build.mjs instead of copy)
      strict: false,
    },
    allowedHosts: true
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
