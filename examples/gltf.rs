use glam::*;
use helia::{
    camera::{Camera, OrthographicSize},
    *,
};

pub struct GameState {}

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        let camera = Camera {
            eye: (-0.5, 1.0, 2.0).into(),
            target: (-0.5, 0.0, 0.0).into(),
            up: Vec3::Y,
            aspect_ratio: state.size.width as f32 / state.size.height as f32,
            fov: 60.0 * std::f32::consts::PI / 180.0,
            near: 0.01,
            far: 1000.0,
            projection: camera::Projection::Perspective,
            size: OrthographicSize::default(),
            clear_color: Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            pixel_ratio: 1.0,
        };

        let model = gltf::Gltf::from_slice(include_bytes!("../assets/cube.gltf")).unwrap();
        log::info!("{:#?}", model);
        // bevy uses gltf so we can crib from them as for what utils we need
        // https://github.com/bevyengine/bevy/blob/1a43ce15edec5b730922d73dae2f2bfe379bf930/crates/bevy_gltf/src/loader.rs#L1115
        // looks like [base64](https://docs.rs/base64/latest/base64/) crate for decoding data uris as well as maybe
        // https://docs.rs/percent-encoding/latest/percent_encoding/fn.percent_decode.html

        state.camera = camera;
    }

    fn update(&mut self, _state: &mut State, _elapsed: f32) {}

    fn render(&mut self, _commands: &mut Vec<DrawCommand>) {
        // If we had something to render then... we'd render it here
    }

    fn resize(&mut self, state: &mut State) {
        state.camera.aspect_ratio = state.size.width as f32 / state.size.height as f32;
    }
}

pub async fn run() {
    let game_state = GameState {};
    Helia::new().run(Box::new(game_state)).await;
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() {
    run().await;
}

fn main() {
    pollster::block_on(run());
}
