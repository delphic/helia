use glam::*;
use helia::{
    camera::{Camera, OrthographicSize},
    // entity::*,
    material::Material,
    mesh::Mesh,
    orbit_camera::*,
    texture::Texture,
    *,
};

const CUBE_POSITIONS: &[Vec3] = &[
    // Front face
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    // Back face
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    // Top face
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(1.0, 1.0, -1.0),
    // Bottom face
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    // Right face
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    // Left face
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(-1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, -1.0),
];
const CUBE_UVS: &[Vec2] = &[
    Vec2::new(0.0, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, 0.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, 0.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(1.0, 1.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(0.0, 0.0),
];
const CUBE_INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, // Front face
    4, 5, 6, 4, 6, 7, // Back face
    8, 9, 10, 8, 10, 11, // Top face
    12, 13, 14, 12, 14, 15, // Bottom face
    16, 17, 18, 16, 18, 19, // Right face
    20, 21, 22, 20, 22, 23, // Left face
];

pub struct GameState {
    orbit_camera: Option<OrbitCamera>,
    cube: Option<(MeshId, MaterialId, Transform)>,
}

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        let device = &state.device;
        let queue = &state.queue;

        let camera = Camera {
            eye: (0.0, 2.0, 4.0).into(),
            target: (0.0, 0.0, 0.0).into(),
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

        state.camera = camera;

        // Makin' Textures
        let texture_bytes = include_bytes!("../assets/crate.png");
        let texture = Texture::from_bytes(&device, &queue, texture_bytes).unwrap();
        let texture_id = state.resources.textures.insert(texture);
        let material = Material::new(state.shaders.unlit_textured, texture_id, state);
        let material_id = state.resources.materials.insert(material);

        let mesh = Mesh::from_arrays(CUBE_POSITIONS, CUBE_UVS, CUBE_INDICES, &device);
        let mesh_id = state.resources.meshes.insert(mesh);

        // let props = InstanceProperties::default();
        // self.cube = Some(state.scene.add_entity(mesh_id, material_id, props));
        self.cube = Some((mesh_id, material_id, Transform::default()));
    }

    fn update(&mut self, state: &mut State, elapsed: f32) {
        if let Some(camera_controller) = &self.orbit_camera {
            camera_controller.update_camera(&mut state.camera, &state.input, elapsed);
        }

        if let Some((_, _, transform)) = &mut self.cube {
            transform.position = Vec3::new(3.0 * state.time.total_elapsed.sin(), 0.0, 0.0);
            transform.rotation =
                Quat::from_euler(EulerRot::XYZ, 0.5 * elapsed, 0.4 * elapsed, 0.2 * elapsed)
                    * transform.rotation;
        }
    }

    fn render(&mut self, commands: &mut Vec<DrawCommand>, _state: &mut State) {
        if let Some((mesh_id, material_id, transform)) = self.cube {
            commands.push(DrawCommand::Draw(mesh_id, material_id, transform));
        }
    }

    fn resize(&mut self, state: &mut State) {
        state.camera.aspect_ratio = state.size.width as f32 / state.size.height as f32;
    }
}

pub async fn run() {
    let game_state = GameState {
        orbit_camera: Some(OrbitCamera::new(1.5)),
        cube: None,
    };
    Helia::new().run(Box::new(game_state)).await;
}

use material::MaterialId;
use mesh::MeshId;
use transform::Transform;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() {
    run().await;
}

fn main() {
    pollster::block_on(run());
}
