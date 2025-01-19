use glam::*;
use helia::{
    camera::{Camera, OrthographicSize},
    entity::*,
    material::Material,
    mesh::Mesh,
    orbit_camera::*,
    shader::Vertex,
    texture::Texture,
    transform::Transform,
    *,
};

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.00759614],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.43041354],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.949397],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.84732914],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.2652641],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: Vec3 = Vec3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    0.0,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub struct GameState {
    orbit_camera: Option<OrbitCamera>,
    scene: Scene,
}

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        let device = &state.device;
        let queue = &state.queue;

        let camera = Camera {
            eye: (-0.5, 1.0, 2.0).into(),
            target: (-0.5, 0.0, 0.0).into(),
            up: Vec3::Y,
            aspect_ratio: state.size.width as f32 / state.size.height as f32,
            fov: 60.0 * std::f32::consts::PI / 180.0,
            near: 0.01,
            far: 1000.0,
            clear_color: Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            projection: camera::Projection::Perspective,
            size: OrthographicSize::default(),
            pixel_ratio: 1.0,
        };

        state.camera = camera;

        // Makin' Textures
        let texture_bytes = include_bytes!("../assets/lena_on_black.png");
        let texture = Texture::from_bytes(&device, &queue, texture_bytes).unwrap();
        let texture_id = state.resources.textures.insert(texture);
        let black_material = Material::new(state.shaders.unlit_textured, texture_id, state);

        let texture_bytes = include_bytes!("../assets/lena_on_rink.png");
        let texture = Texture::from_bytes(&device, &queue, texture_bytes).unwrap();
        let texture_id = state.resources.textures.insert(texture);
        let rink_material = Material::new(state.shaders.unlit_textured, texture_id, state);

        let mesh = Mesh::new(VERTICES, INDICES, &device);
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = Vec3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - INSTANCE_DISPLACEMENT;

                    let rotation = if position == Vec3::ZERO {
                        Quat::from_axis_angle(Vec3::Z, 0.0)
                    } else {
                        Quat::from_axis_angle(
                            position.normalize(),
                            45.0 * std::f32::consts::PI / 180.0,
                        )
                    };

                    let transform = Transform::from_position_rotation(position, rotation);
                    (
                        transform, 
                        InstanceProperties::from_transform(transform)
                    )
                })
            })
            .collect::<Vec<_>>();

        let mesh_id = state.resources.meshes.insert(mesh);
        let black_material_id = state.resources.materials.insert(black_material);
        let rink_material_id = state.resources.materials.insert(rink_material);
        let lena_prefab_id = self.scene.create_prefab(mesh_id, black_material_id);
        let lena_alt_prefab_id = self.scene.create_prefab(mesh_id, rink_material_id);

        for (i, (transform, props)) in instances.iter().enumerate() {
            if i % 2 == 0 {
                self.scene.add_instance(lena_prefab_id, *transform, *props);
            } else {
                self.scene.add_instance(lena_alt_prefab_id, *transform, *props);
            }
        }
    }

    fn update(&mut self, state: &mut State, elapsed: f32) {
        if let Some(camera_controller) = &self.orbit_camera {
            camera_controller.update_camera(&mut state.camera, &state.input, elapsed);
        }
        self.scene.update(&state.camera, &state.resources);
    }

    fn render(&mut self, commands: &mut Vec<DrawCommand>) {
        self.scene.render(commands);
    }

    fn resize(&mut self, state: &mut State) {
        state.camera.aspect_ratio = state.size.width as f32 / state.size.height as f32;
    }
}

pub async fn run() {
    let game_state = GameState {
        orbit_camera: Some(OrbitCamera::new(1.5)),
        scene: Scene::new(),
    };
    Helia::new().run(Box::new(game_state)).await;
}

use scene::Scene;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn start() {
    run().await;
}

fn main() {
    pollster::block_on(run());
}
