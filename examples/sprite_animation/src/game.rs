use crate::aseprite::*;
use glam::*;
use helia::{camera::Camera, entity::*, mesh::Mesh, *};

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
pub struct GameState {
    sprite_data: AsepriteAnimation,
    time_in_frame: f32,
    current_frame: usize,
    lena: Option<EntityId>,
}

impl Game for GameState {
    fn init(&mut self, state: &mut State) {
        let sprite_bytes = include_bytes!("../assets/lena_shoot.png");

        let texture = helia::texture::Texture::from_bytes(
            &state.device,
            &state.queue,
            sprite_bytes,
            "lena_shoot",
        )
        .unwrap();
        let lena_material = helia::material::Material::new(state.shaders.sprite, texture, &state);
        let lena_material_id = state.resources.materials.insert(lena_material);

        let quad_mesh = Mesh::from_arrays(QUAD_POSITIONS, QUAD_UVS, QUAD_INDICES, &state.device);
        let mesh_id = state.resources.meshes.insert(quad_mesh);

        let camera = Camera {
            eye: (0.0, 0.0, 2.0).into(),
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
            projection: camera::Projection::Orthographic,
            size: 1.0,
        };

        state.scene.camera = camera;

        let (scale, offset) = self.calculate_scale_offset(self.current_frame);
        self.lena = Some(
            state.scene.add_entity(
                mesh_id,
                lena_material_id,
                InstancePropertiesBuilder::new()
                    .with_uv_offset_scale(offset, scale)
                    .build(),
            ),
        );
        // todo: change to InstanceProperties::builder() rather than importing the type
    }

    fn update(&mut self, state: &mut State, elapsed: f32) {
        self.time_in_frame += elapsed * 1000.0;
        let frame_duration = self.sprite_data.frames[self.current_frame].duration as f32;
        if self.time_in_frame > frame_duration {
            self.time_in_frame -= frame_duration;
            self.current_frame = (self.current_frame + 1) % self.sprite_data.frames.len();
            if let Some(entity_id) = self.lena {
                let (scale, offset) = self.calculate_scale_offset(self.current_frame);
                let lena = state.scene.get_entity_mut(entity_id);
                lena.properties.uv_scale = scale;
                lena.properties.uv_offset = offset;
            }
        }
    }

    fn input(&mut self, _state: &mut State, _event: &winit::event::WindowEvent) -> bool {
        false
    }

    fn resize(&mut self, state: &mut State) {
        state.scene.camera.aspect_ratio = state.size.width as f32 / state.size.height as f32;
    }
}

impl GameState {
    fn calculate_scale_offset(&mut self, index: usize) -> (Vec2, Vec2) {
        let frame = &self.sprite_data.frames[index].frame;
        let w = self.sprite_data.meta.size.w as f32;
        let h = self.sprite_data.meta.size.h as f32;
        let scale = Vec2::new(frame.w as f32 / w, frame.h as f32 / h);
        let offset = Vec2::new(frame.x as f32 / w, frame.y as f32 / h);
        (scale, offset)
    }
}

pub async fn run() {
    let game_state = GameState {
        current_frame: 0,
        time_in_frame: 0.0,
        lena: None,
        sprite_data: serde_json::from_str::<AsepriteAnimation>(include_str!(
            "../assets/lena_shoot.json"
        ))
        .unwrap(),
    };
    helia::run(Box::new(game_state)).await;
}
