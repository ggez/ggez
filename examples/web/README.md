# Running ggez examples in a browser

Use build.mjs to compile every example to `wasm32-unknown-unknown`, run `wasm-bindgen`,
and serve them in a simple gallery.

## One-time setup

```sh
rustup target add wasm32-unknown-unknown
cd examples/web
npm install
```

`build.mjs` will install a matching `wasm-bindgen-cli` via `cargo install` on the
first run if your version doesn't match `Cargo.lock`. Pass `--no-bindgen-install`
to opt out.

## Day-to-day

```sh
npm run dev                   # build every example, then `vite`
npm run build                 # just rebuild
npm run build -- 04_snake     # only the 04_snake example
npm run build -- --release    # release-mode wasm (much smoother in the browser)
npm run serve                 # serve a previously built tree without rebuilding
```

Vite opens <http://localhost:5173>; the index page lists every example that's
been built and runs the selected one in an iframe pointed at
`run.html?example=NAME`.

## Layout

- `build.mjs` — invokes `cargo build --target wasm32-unknown-unknown --example NAME`
  for each example (grouped by required features), then `wasm-bindgen --target web`,
  and copies `../../resources` into `public/resources`.
- `index.html` / `src/gallery.js` — the gallery (sidebar + iframe).
- `run.html` / `src/runner.js` — the single-example host. The canvas is appended
  to `#ggez-canvas-host` (configured via `WindowSetup::web_canvas_parent_id`).
- `public/` (gitignored) — output: `examples/<name>/<name>.js`, `<name>_bg.wasm`,
  plus `resources/`.

## Embedding ggez in your own page

ggez appends its `<canvas>` to `<body>` by default. To target an existing element:

```rust
let cb = ggez::ContextBuilder::new("my_game", "me")
    .window_setup(
        ggez::conf::WindowSetup::default().web_canvas_parent_id("my-host-element"),
    );
```

That's all `run.html` does; you can copy it as a starting point.
