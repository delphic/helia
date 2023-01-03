use helia::*;

fn main() {
    pollster::block_on(run());
}

// Q: how does macroquad manage to make main async?
// consider use of https://docs.rs/tokio or https://docs.rs/async-std over pollster