#!/bin/sh
cargo build --target wasm32-unknown-unknown --example particles
#wasm-bindgen target/wasm32-unknown-unknown/debug/ggraphics.wasm --out-dir generated --no-modules
wasm-bindgen ../target/wasm32-unknown-unknown/debug/examples/particles.wasm --out-dir generated --no-modules
cp -f index.html generated/
rsync -av generated/ icefox@roc.alopex.li:htdocs/temp/g12
