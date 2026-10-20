import { defineConfig } from 'vite';

export default defineConfig({
  root: '.',
  publicDir: 'public',
  server: {
    open: '/',
    fs: { strict: false },
    allowedHosts: true,
  },
  build: {
    target: 'esnext',
    rollupOptions: {
      input: {
        index: 'index.html',
        run: 'run.html',
      },
    },
  },
});
