use glam::*;
use helia::{camera::{Camera, OrthographicSize}, entity::*, mesh::{Mesh, MeshId}, *, material::{MaterialId, Material}, texture::Texture};

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

fn sized_quad_positions(width: f32, height: f32) -> Vec<Vec3> {
    QUAD_POSITIONS.iter().map(|v| Vec3::new(width * v.x, height * v.y, v.z)).collect::<Vec<Vec3>>()
}

pub struct GameState {
    sprite: Option<EntityId>,
}

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        fn build_sprite_resources(label: &str, width: f32, height: f32, sprite_bytes: &[u8], state: &mut State) -> (MeshId, MaterialId) {
            let texture = Texture::from_bytes(
                &state.device,
                &state.queue,
                sprite_bytes,
                label,
            )
            .unwrap();
            let material = Material::new(state.shaders.sprite, texture, &state);
            let material_id = state.resources.materials.insert(material);
    
            let quad_mesh = Mesh::from_arrays(&sized_quad_positions(width, height).as_slice(), QUAD_UVS, QUAD_INDICES, &state.device);
            let mesh_id = state.resources.meshes.insert(quad_mesh);
            (mesh_id, material_id)
        }

        let helia_sprite = build_sprite_resources("helia", 96.0, 96.0, include_bytes!("../assets/helia.png"), state);
        let bg_sprite = build_sprite_resources("bg", 960.0, 480.0, include_bytes!("../assets/placeholder-bg.png"), state);

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

        self.sprite = Some(
            state.scene.add_entity(
                helia_sprite.0,
                helia_sprite.1,
                InstanceProperties::builder()
                    .with_translation(Vec3::new(64.0, 0.0, 0.0)) // right/left is x: +/- 64, down / up is x : +/-32.0, y: +/-32.0
                    .build(),
            ),
        );

        state.scene.add_entity(
            bg_sprite.0,
            bg_sprite.1,
            InstanceProperties::builder()
                .with_translation(Vec3::new(0.0, 0.0, -100.0))
                .build()
        );
    }

    fn update(&mut self, _state: &mut State, _elapsed: f32) {

    }

    fn resize(&mut self, state: &mut State) {
        state.scene.camera.size = OrthographicSize::from_size(state.size);
    }
}

pub async fn run() {
    let game_state = GameState {
        sprite: None,
    };
    Helia::new()
        .with_title("Helia Tactics")
        .with_size(960, 640)

        .with_resizable(false)
        .run(Box::new(game_state)).await;
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