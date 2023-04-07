# Helia

A Radiant Game Engine.

Focusing on simplicity, flexbility and programmer productivity.

## Build

Standalone: `cargo build --example name` / `cargo run --example name`

### Web
Ensure wasm bindgen cli is installed and matches your cargo lock file.

`cargo install -f wasm-bindgen-cli`

Create `build/` folder and copy over `templates/index.html` and adjust its js module import to match example name.

`cargo build --example <name> --target wasm32-unknown-unknown --release`

`wasm-bindgen --target web ./target/wasm32-unknown-unknown/release/examples/<name>.wasm --out-dir ./build/pkg --no-typescript`

`http-server ./build`

## System Dependencies on Linux

Install cmake, fontconfig*, libfontconfig1-dev.

For target x86_64-pc-windows-gnu: mingw-w64