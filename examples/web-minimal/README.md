# ggez on the web without a bundler

The smallest setup that runs a ggez game in a browser: one HTML file, no Node.

```sh
rustup target add wasm32-unknown-unknown
cargo install --locked wasm-bindgen-cli   # version must match `wasm-bindgen` in Cargo.lock
./build.sh                                # or ./build.sh 04_snake
python3 -m http.server 8000               # any static file server works
```

Then open <http://localhost:8000> in a browser with WebGPU (on Linux, Chrome with
`chrome://flags/#enable-unsafe-webgpu`).

`build.sh` runs `cargo build --target wasm32-unknown-unknown`, `wasm-bindgen --target web` into
`pkg/`, and zips `../../resources` into `resources.zip`. `index.html` does the two things a page
hosting ggez needs: preload the resources zip and call `init()`. Copy the file into your own project.

For a gallery of every example with a dev server, see [`../web`](../web).
