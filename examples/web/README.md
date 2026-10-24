# Running ggez examples in a browser

`build.mjs` compiles the examples to `wasm32-unknown-unknown`, runs `wasm-bindgen`, and the
Vite dev server serves them in a gallery.

ggez only supports WebGPU (not WebGL2). On Linux you may need to enable WebGPU in your browser
(e.g. `chrome://flags/#enable-unsafe-webgpu` for Chrome).

## Setup

```sh
rustup target add wasm32-unknown-unknown
cd examples/web
npm install
```

`build.mjs` checks that your `wasm-bindgen-cli` matches `Cargo.lock` and prints the install
command if it doesn't.

## Scripts

```sh
npm run dev                   # build every example, then serve
npm run build                 # just rebuild
npm run build -- 04_snake     # only the 04_snake example
npm run build -- --release    # release-mode wasm (smoother in the browser)
npm run serve                 # serve without rebuilding
npm run build:static          # release wasm + `vite build` into dist/
```

The gallery at <http://localhost:5173> runs the selected example in an iframe pointed at
`run.html?example=NAME`.

## Layout

- `build.mjs` builds the examples into `public/examples/<name>/` and zips `../../resources` into
  `public/resources.zip`.
- `index.html` / `src/gallery.js`: the gallery.
- `run.html` / `src/runner.js`: runs one example. It sets `window.__GGEZ_RESOURCES_ZIP__` to the
  resources zip, which ggez mounts as its resources directory.

For a single page with no bundler or Node, see [`../web-minimal`](../web-minimal).

If you'd rather not load resources from JS, embed them in the binary instead:

```rust
ggez::ContextBuilder::new("my_game", "me")
    .add_zipfile_bytes(include_bytes!("../resources.zip").to_vec())
    .run::<MyGame>()
```
