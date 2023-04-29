mod battle_state;
mod character;
mod grid;
mod player;
mod slice_sprite;
mod text_mesh;
mod utils;

use std::collections::HashMap;

use battle_state::*;
use slice_sprite::*;

use glam::*;
use helia::{camera::*, entity::InstanceProperties, material::MaterialId, mesh::MeshId, *};

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
        // just keeping all resources in memory for now
        // will probably want a way to clear and reset
        // resources in a larger game though
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
        self.resources.insert(
            "micro_font".to_string(),
            utils::build_sprite_resources(
                "micro_font",
                22.0 * 4.0, // characters are 4 pixels wide, 22 characters per row
                4.0 * 6.0,  // characters are 6 pixels high, 4 rows in the atlas
                Vec2::new(0.0, 0.0),
                include_bytes!("../assets/micro-font.png"),
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

        // 9 slice test
        let mut slice_mesh = SliceSpriteMesh::new(
            Vec2::new(16.0, 16.0),
            SliceConfig {
                width: 8.0,
                height: 8.0,
                top: 2.0,
                right: 2.0,
                bottom: 2.0,
                left: 2.0,
            },
            state,
        );
        let texture = helia::texture::Texture::from_bytes(
            &state.device,
            &state.queue,
            include_bytes!("../assets/slice.png"),
            "slice",
        )
        .unwrap();
        let material = helia::material::Material::new(state.shaders.sprite, texture, &state);
        let material_id = state.resources.materials.insert(material);
        state.scene.add_entity(
            slice_mesh.mesh,
            material_id,
            InstanceProperties::builder()
                .with_translation(Vec3::new(0.0, 64.0, 0.0))
                .with_scale(4.0 * Vec3::ONE)
                .build(),
        );

        slice_mesh.resize(Vec2::new(32.0, 16.0), state);

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
