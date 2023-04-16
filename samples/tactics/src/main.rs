mod character;
mod grid;
mod player;

use character::*;
use grid::*;
use player::*;

use glam::*;
use helia::{
    camera::{Camera, OrthographicSize},
    entity::*,
    material::{Material, MaterialId},
    mesh::{Mesh, MeshId},
    texture::Texture,
    *,
};

const QUAD_POSITIONS: &[Vec3] = &[
    Vec3::new(-0.5, -0.5, 0.0),
    Vec3::new(0.5, -0.5, 0.0),
    Vec3::new(0.5, 0.5, 0.0),
    Vec3::new(-0.5, 0.5, 0.0),
];
const QUAD_UVS: &[Vec2] = &[
    Vec2::new(0.0, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, 0.0),
];
const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

fn sized_quad_positions(width: f32, height: f32, offset: Vec2) -> Vec<Vec3> {
    QUAD_POSITIONS
        .iter()
        .map(|v| Vec3::new(width * v.x + offset.x, height * v.y + offset.y, v.z))
        .collect::<Vec<Vec3>>()
}
// TODO: Should we perhaps just have a single global quad mesh in Helia and use scale instead?

pub struct GameState {
    players: Vec<Player>,
    dummys: Vec<Character>,
    grid: Grid,
}
// ^^ TODO: Since enum of current state, which has the information for each state
// Initial state is loading

impl GameState {
    fn new() -> Self {
        Self {
            players: Vec::new(),
            dummys: Vec::new(),
            grid: Grid::new(),
        }
    }
}

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        fn build_sprite_resources(
            label: &str,
            width: f32,
            height: f32,
            offset: Vec2,
            sprite_bytes: &[u8],
            state: &mut State,
        ) -> (MeshId, MaterialId) {
            let texture =
                Texture::from_bytes(&state.device, &state.queue, sprite_bytes, label).unwrap();
            let material = Material::new(state.shaders.sprite, texture, &state);
            let material_id = state.resources.materials.insert(material);

            let quad_mesh = Mesh::from_arrays(
                &sized_quad_positions(width, height, offset).as_slice(),
                QUAD_UVS,
                QUAD_INDICES,
                &state.device,
            );
            let mesh_id = state.resources.meshes.insert(quad_mesh);
            (mesh_id, material_id)
        }

        let helia_sprite_ids = build_sprite_resources(
            "helia",
            96.0,
            96.0,
            Vec2::new(0.0, 48.0),
            include_bytes!("../assets/helia.png"),
            state,
        );
        let bg_sprite_ids = build_sprite_resources(
            "bg",
            960.0,
            480.0,
            Vec2::ZERO,
            include_bytes!("../assets/placeholder-bg.png"),
            state,
        );
        let highlight_ids = build_sprite_resources(
            "sq",
            96.0,
            32.0,
            Vec2::new(0.0, 16.0),
            include_bytes!("../assets/grid_sq.png"),
            state,
        );
        let dummy_ids = build_sprite_resources(
            "dummy",
            64.0,
            64.0,
            Vec2::new(0.0, 32.0),
            include_bytes!("../assets/dummy.png"),
            state,
        );

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

        state.scene.camera = camera;

        let helia_character = Character::create_on_grid(
            IVec2::new(8, 1),
            helia_sprite_ids.0,
            helia_sprite_ids.1,
            &mut self.grid,
            state,
        );
        self.players.push(Player {
            character: helia_character,
            facing: IVec2::new(-1, 0),
        });

        state.scene.add_entity(
            bg_sprite_ids.0,
            bg_sprite_ids.1,
            InstanceProperties::builder()
                .with_translation(Vec3::new(0.0, 0.0, -100.0))
                .build(),
        );

        for i in 0..3 {
            let dummy_character = Character::create_on_grid(
                IVec2::new(4 + i % 2, i),
                dummy_ids.0,
                dummy_ids.1,
                &mut self.grid,
                state,
            );
            self.dummys.push(dummy_character);
        }

        let highlight_prefab = state.scene.create_prefab(highlight_ids.0, highlight_ids.1);

        self.grid.init(highlight_prefab, state);

        // start player movement turn
        let player = &mut self.players[0]; // todo: active player
        player.character.update_distance_map(&self.grid);
        self.grid.update_hightlights(&player.character, state);
    }

    fn update(&mut self, state: &mut State, _elapsed: f32) {
        let player = &mut self.players[0]; // todo: active player
        if let Some(character_move) = player.update(&self.grid, state, _elapsed) {
            self.grid.occupancy.remove(&character_move.0);
            self.grid.occupancy.insert(character_move.1);
            player.character.distance_map = None;

            for dummy in &mut self.dummys {
                let delta = IVec2::new(1, 0);
                if dummy.is_move_valid(&self.grid, delta) {
                    dummy.perform_move(delta, &self.grid, state);
                    self.grid.occupancy.remove(&dummy.last_position);
                    self.grid.occupancy.insert(dummy.position);
                    dummy.last_position = dummy.position;

                    // flip, for fun
                    dummy.flip_visual(state);
                }
            }

            // back to the players turn
            player.character.update_distance_map(&self.grid);
            self.grid.update_hightlights(&player.character, state);
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
