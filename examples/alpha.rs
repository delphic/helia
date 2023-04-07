use glam::*;
use helia::{
    camera::Camera,
    orbit_camera::*,
    entity::{EntityId, InstanceProperties},
    material::Material,
    mesh::Mesh,
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
    cubes: Vec<EntityId>,
    time: f32,
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
            size: 1.0,
            clear_color: Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        };

        state.scene.camera = camera;

        // Makin' Textures
        let texture_bytes = include_bytes!("../assets/lena.png");
        let texture = Texture::from_bytes(&device, &queue, texture_bytes, "lena").unwrap();
        let material = Material::new(state.shaders.sprite, texture, state);
        let material_id = state.resources.materials.insert(material);

        let mesh = Mesh::from_arrays(CUBE_POSITIONS, CUBE_UVS, CUBE_INDICES, &device);
        let mesh_id = state.resources.meshes.insert(mesh);

        for i in 0..3 {
            let props = InstanceProperties::builder()
                .with_transform(glam::Mat4::from_rotation_translation(
                    Quat::IDENTITY,
                    -2.0 * (i as f32) * Vec3::Z,
                ))
                .build();
            self.cubes
                .push(state.scene.add_entity(mesh_id, material_id, props));
        }
    }

    fn update(&mut self, state: &mut State, elapsed: f32) {
        self.time += elapsed; // todo: should be getting this from helia
        if let Some(camera_controller) = &self.orbit_camera {
            camera_controller.update_camera(&mut state.scene.camera, &state.input, elapsed);
        }

        for (i, cube) in self.cubes.iter().enumerate() {
            let entity = state.scene.get_entity_mut(*cube);
            let (scale, rotation, _) = entity.properties.transform.to_scale_rotation_translation();
            let translation = Vec3::new((i as f32 + self.time).sin(), 0.0, -2.0 * i as f32);
            let rotation =
                Quat::from_euler(EulerRot::XYZ, 0.5 * elapsed, 0.4 * elapsed, 0.2 * elapsed)
                    * rotation;
            entity.properties.transform =
                Mat4::from_scale_rotation_translation(scale, rotation, translation);
            // well that's horrible to work with, going to want some kind of Transform struct
            // exposing position / rotation / scale and build the matrix
        }
    }

    fn resize(&mut self, state: &mut State) {
        state.scene.camera.aspect_ratio = state.size.width as f32 / state.size.height as f32;
    }
}

pub async fn run() {
    let game_state = GameState {
        orbit_camera: Some(OrbitCamera::new(1.5)),
        cubes: Vec::new(),
        time: 0.0,
    };
    helia::run(Box::new(game_state)).await;
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