use helia::*;

fn main() {
    pollster::block_on(run());
}

// following - https://sotrh.github.io/learn-wgpu/
// Needed to install cmake
// Need fontconfig & libfontconfig1-dev

// Q: how does macroquad manage to make main async?
// consider use of https://docs.rs/tokio or https://docs.rs/async-std over pollster
