mod battle_stage;
mod character;
mod grid;
mod player;
mod utils;

use std::collections::HashMap;

use battle_stage::*;

use glam::*;
use helia::{camera::*, material::MaterialId, mesh::MeshId, *};

type GameResources = HashMap<String, (MeshId, MaterialId)>;

enum Stage {
    Init,
    Battle { state: BattleState },
}

pub struct GameState {
    stage: Stage,
    resources: GameResources,
}

impl GameState {
    fn new() -> Self {
        Self {
            stage: Stage::Init,
            resources: GameResources::new(),
        }
    }

    fn load_resources(&mut self, state: &mut State) {
        self.resources.insert(
            "helia".to_string(),
            utils::build_sprite_resources(
                "helia",
                96.0,
                96.0,
                Vec2::new(0.0, 48.0),
                include_bytes!("../assets/helia.png"),
                state,
            ),
        );
        self.resources.insert(
            "bg".to_string(),
            utils::build_sprite_resources(
                "bg",
                960.0,
                480.0,
                Vec2::ZERO,
                include_bytes!("../assets/placeholder-bg.png"),
                state,
            ),
        );
        self.resources.insert(
            "highlight".to_string(),
            utils::build_sprite_resources(
                "sq",
                96.0,
                32.0,
                Vec2::new(0.0, 16.0),
                include_bytes!("../assets/grid_sq.png"),
                state,
            ),
        );
        self.resources.insert(
            "dummy".to_string(),
            utils::build_sprite_resources(
                "dummy",
                64.0,
                64.0,
                Vec2::new(0.0, 32.0),
                include_bytes!("../assets/dummy.png"),
                state,
            ),
        );
    }
}

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        let camera = Camera {
            eye: (0.0, 0.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vec3::Y,
            aspect_ratio: state.size.width as f32 / state.size.height as f32,
            fov: 60.0 * std::f32::consts::PI / 180.0,
            near: 0.01,
            far: 1000.0,
            clear_color: Color::BLACK,
            projection: camera::Projection::Orthographic,
            size: OrthographicSize::from_size(state.size),
        };

        self.load_resources(state);

        state.scene.camera = camera;

        let mut battle_state = BattleState::new(&self.resources, state);

        battle_state.enter(state);
        self.stage = Stage::Battle {
            state: battle_state,
        };
    }

    fn update(&mut self, state: &mut State, elapsed: f32) {
        match &mut self.stage {
            Stage::Init => {}
            Stage::Battle {
                state: battle_state,
            } => battle_state.update(state, elapsed),
        }
    }

    fn resize(&mut self, state: &mut State) {
        state.scene.camera.size = OrthographicSize::from_size(state.size);
    }
}

pub async fn run() {
    Helia::new()
        .with_title("Helia Tactics")
        .with_size(960, 640)
        .with_resizable(false)
        .run(Box::new(GameState::new()))
        .await;
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
