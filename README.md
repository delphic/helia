# Helia

A WIP Rust Game Engine!

## Build

Standalone: `cargo build` / `cargo run`

### Web
Ensure wasm bindgen cli is installed and matches your cargo lock file.

`cargo install -f wasm-bindgen-cli`

Install a simple http-server of your choice, e.g.
`npm install -g http-server`

Update index.html in example crate so JS file import matches crate name.

Then per build

`cd ./examples/<example>`

`cargo build --target wasm32-unknown-unknown --release`

`wasm-bindgen --target web ../../target/wasm32-unknown-unknown/release/<example-crate-name>.wasm --out-dir ./pkg`

`http-server`

## System Dependencies on Linux

Install cmake, fontconfig*, libfontconfig1-dev.

For target x86_64-pc-windows-gnu: mingw-w64