#!/usr/bin/env sh
# Build an example for the browser with just cargo, wasm-bindgen and zip.
# Usage: ./build.sh [example]   (default: 05_astroblasto)
set -eu
cd "$(dirname "$0")"
EXAMPLE=${1:-05_astroblasto}
ROOT=../..

cargo build --manifest-path "$ROOT/Cargo.toml" --target wasm32-unknown-unknown --release \
  --features 3d --example "$EXAMPLE"
wasm-bindgen --target web --no-typescript --out-dir pkg --out-name game \
  "$ROOT/target/wasm32-unknown-unknown/release/examples/$EXAMPLE.wasm"

rm -f resources.zip
(cd "$ROOT/resources" && zip -qr "$OLDPWD/resources.zip" .)

echo "Built $EXAMPLE. Serve this directory, e.g.: python3 -m http.server 8000"
