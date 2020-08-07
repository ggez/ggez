#!/bin/sh
# To get the wasm-bindgen command you have to do:
# cargo install wasm-bindgen-cli
# the wasm-bindgen LIB does not heckin document this well.
cargo build --target wasm32-unknown-unknown --example particles
#wasm-bindgen target/wasm32-unknown-unknown/debug/ggraphics.wasm --out-dir generated --no-modules
wasm-bindgen ../target/wasm32-unknown-unknown/debug/examples/particles.wasm --out-dir generated --no-modules
cp -f index.html generated/
rsync -av generated/ icefox@roc.alopex.li:htdocs/temp/g12
