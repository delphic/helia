use glam::*;
use helia::{
    camera::Camera,
    camera_controller::*,
    material::Material,
    mesh::Mesh,
    shader::{Instance, Vertex},
    *,
};
use winit::event::WindowEvent;

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
    camera_controller: Option<CameraController>,
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
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        };

        state.scene.camera = camera;

        // Makin' Textures
        let diffuse_bytes = include_bytes!("../../../assets/lena.png");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "lena.png").unwrap();
        let material = Material::new(diffuse_texture, &state.texture_bind_group_layout, &device);
        // ^^ arguably material should contain a link to the shader it executes (an id)
        // ^^ todo: remove need for texture_bind_group_layout from the call at this location

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

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let prefab = Prefab::new(mesh, material, instances, device);

        state.scene.prefabs.push(prefab);
    }

    fn update(&mut self, state: &mut State, elapsed: f32) {
        if let Some(camera_controller) = &self.camera_controller {
            camera_controller.update_camera(&mut state.scene.camera, elapsed);
        }
    }

    fn input(&mut self, state: &mut State, event: &winit::event::WindowEvent) -> bool {
        if let Some(camera_controller) = &mut self.camera_controller {
            camera_controller.process_events(event);
        }
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                state.scene.camera.clear_color = wgpu::Color {
                    r: position.x / state.size.width as f64,
                    g: 0.2,
                    b: position.y / state.size.height as f64,
                    a: 1.0,
                };
                true
            }
            _ => false,
        }
    }
}

pub async fn run() {
    let game_state = GameState {
        camera_controller: Some(CameraController::new(1.5)),
    };
    helia::run(Box::new(game_state)).await;
}

// Q: how does macroquad manage to make main async?
// TODO: Remove main.rs and create /examples
