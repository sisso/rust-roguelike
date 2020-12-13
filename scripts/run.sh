#!/bin/bash

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/rust-rogelike.wasm --out-dir wasm --no-modules --no-typescript
cd wasm
python3 -m http.server