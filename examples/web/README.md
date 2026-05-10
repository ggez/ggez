# Running ggez examples in a browser

Use build.mjs to compile examples to `wasm32-unknown-unknown`, run `wasm-bindgen`, and
serve them in a gallery.

## One-time setup

```sh
rustup target add wasm32-unknown-unknown
cd examples/web
npm install
```

`build.mjs` checks that your `wasm-bindgen-cli` matches `Cargo.lock` and prints the
install command if it doesn't.

## Scripts

```sh
npm run dev                   # build every example, then `vite`
npm run build                 # just rebuild
npm run build -- 04_snake     # only the 04_snake example
npm run build -- --release    # release-mode wasm (smoother in the browser)
npm run serve                 # serve without rebuilding
npm run build:static          # release wasm + `vite build` → dist/
```

Vite opens <http://localhost:5173>; the index page lists every built example and
runs the selected one in an iframe pointed at `run.html?example=NAME`.

## Layout

- `build.mjs` — invokes `cargo build --target wasm32-unknown-unknown --example NAME` for each
  example (grouped by features), then `wasm-bindgen --target web`, and copies `../../resources`
  to `public/resources`.
- `index.html` / `src/gallery.js` — the gallery (sidebar + iframe).
- `run.html` / `src/runner.js` — example host, a canvas is appended to `#ggez-canvas-host`.
- `public/` gitignored output: `examples/<name>/<name>.js`, `<name>_bg.wasm`, `resources/`.

## Embedding ggez in your own page

ggez appends its `<canvas>` to `<body>` by default. To target an existing element:

```rust
let cb = ggez::ContextBuilder::new("my_game", "me")
    .window_setup(
        ggez::conf::WindowSetup::default().web_canvas_parent_id("my-host-element"),
    );
```

That's all `run.html` does; you can copy it as a starting point.
