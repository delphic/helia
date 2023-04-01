use glam::*;
use helia::{
    camera::Camera, camera_controller::*, entity::EntityId, material::Material, mesh::Mesh,
    shader::Vertex, texture::Texture, *,
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
    camera_controller: Option<CameraController>,
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
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        };

        state.scene.camera = camera;

        // Makin' Textures
        let texture_bytes = include_bytes!("../../../assets/lena.png");
        let texture = Texture::from_bytes(&device, &queue, texture_bytes, "lena").unwrap();
        let material = Material::new(state.shaders.sprite, texture, state);
        let material_id = state.resources.materials.insert(material);

        let mut vertices = Vec::new();
        for i in 0..CUBE_POSITIONS.len() {
            vertices.push(Vertex {
                position: CUBE_POSITIONS[i].to_array(),
                tex_coords: CUBE_UVS[i].to_array(),
            });
        }

        let mesh = Mesh::new(vertices.as_slice(), CUBE_INDICES, &device);
        let mesh_id = state.resources.meshes.insert(mesh);

        for i in 0..3 {
            let transform =
                glam::Mat4::from_rotation_translation(Quat::IDENTITY, -2.0 * (i as f32) * Vec3::Z);
            self.cubes.push(
                state
                    .scene
                    .add_entity(transform, mesh_id, material_id),
            );
        }
    }

    fn update(&mut self, state: &mut State, elapsed: f32) {
        self.time += elapsed; // todo: should be getting this from helia
        if let Some(camera_controller) = &self.camera_controller {
            camera_controller.update_camera(&mut state.scene.camera, elapsed);
        }

        for (i, cube) in self.cubes.iter().enumerate() {
            let entity = state.scene.get_entity_mut(*cube);
            let (scale, rotation, _) = entity.transform.to_scale_rotation_translation();
            let translation = Vec3::new((i as f32 + self.time).sin(), 0.0, -2.0 * i as f32);
            let rotation =
                Quat::from_euler(EulerRot::XYZ, 0.5 * elapsed, 0.4 * elapsed, 0.2 * elapsed)
                    * rotation;
            entity.transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
            // well that's horrible to work with, going to want some kind of Transform struct
            // exposing position / rotation / scale and build the matrix
        }
    }

    fn input(&mut self, _state: &mut State, event: &winit::event::WindowEvent) -> bool {
        if let Some(camera_controller) = &mut self.camera_controller {
            return camera_controller.process_events(event);
        }
        false
    }

    fn resize(&mut self, state: &mut State) {
        state.scene.camera.aspect_ratio = state.size.width as f32 / state.size.height as f32;
    }
}

pub async fn run() {
    let game_state = GameState {
        camera_controller: Some(CameraController::new(1.5)),
        cubes: Vec::new(),
        time: 0.0,
    };
    helia::run(Box::new(game_state)).await;
}

// Q: how does macroquad manage to make main async?
// A: TL:DR "macros"
