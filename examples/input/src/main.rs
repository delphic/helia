mod game;

fn main() {
    pollster::block_on(game::run());
}
